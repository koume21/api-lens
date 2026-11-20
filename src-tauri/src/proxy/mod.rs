//! 関所（プロキシ）。ブラウザからの通信を受けて、登録したホスト宛てのものだけを記録し、
//! 本来の相手へそのまま渡す（hudsucker を使う）。
//!
//! - 観測は「見るだけ」。中身の書き換え・遮断はしない
//! - 待ち受けは 127.0.0.1 だけ。ほかの PC からは使えない（N6）
//! - 記録した通信は `ExchangeSink`（呼び出し側が渡すコールバック）に渡す。
//!   DB への保存や画面への通知は呼び出し側（commands）の仕事で、ここは Tauri を知らない
//!
//! ファイル:
//! - `handler.rs`: 1 件ごとの記録処理（hudsucker の `HttpHandler`）
//! - `cert.rs`:    認証局（CA）証明書の生成

mod cert;
mod handler;

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, RwLock};

use hudsucker::Proxy;
use hudsucker::rustls::crypto::aws_lc_rs;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::models::Exchange;
use handler::ObserveHandler;

/// 記録した通信の受け取り手。関所は 1 件記録するたびにこれを呼ぶ
pub type ExchangeSink = Arc<dyn Fn(Exchange) + Send + Sync>;

/// 観測対象ホストの一覧（`localhost:3000` など、整えた形）。
/// 観測中でも登録・削除がすぐ効くよう、関所と受付で共有して読み書きする
pub type SharedHosts = Arc<RwLock<Vec<String>>>;

#[derive(Debug)]
pub enum ProxyError {
    AlreadyRunning,
    Bind(std::io::Error),
    Cert(String),
    Build(hudsucker::Error),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::AlreadyRunning => write!(f, "すでに観測中です"),
            ProxyError::Bind(e) => write!(
                f,
                "ポートを開けませんでした。ほかのアプリが使っていないか確認してください（{e}）"
            ),
            ProxyError::Cert(e) => write!(f, "証明書を用意できませんでした（{e}）"),
            ProxyError::Build(e) => write!(f, "プロキシを起動できませんでした（{e}）"),
        }
    }
}

impl std::error::Error for ProxyError {}

/// 動いている関所の情報。止めるときに使う
struct Running {
    addr: SocketAddr,
    /// これに値を送ると、関所が新しい接続の受け付けをやめて止まる
    stop: oneshot::Sender<()>,
}

/// 関所の起動・停止を受け持つ。アプリ全体で 1 つ。
pub struct ProxyController {
    hosts: SharedHosts,
    running: Mutex<Option<Running>>,
}

impl Default for ProxyController {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyController {
    pub fn new() -> Self {
        Self {
            hosts: Arc::new(RwLock::new(Vec::new())),
            running: Mutex::new(None),
        }
    }

    /// 観測対象ホストを入れ替える（登録・削除のたびに受付が呼ぶ）。観測中でもすぐ効く
    pub fn set_hosts(&self, hosts: Vec<String>) {
        *self.hosts.write().unwrap_or_else(|e| e.into_inner()) = hosts;
    }

    /// 観測中なら待ち受けアドレス、止まっていれば None
    pub fn status(&self) -> Option<SocketAddr> {
        self.lock_running().as_ref().map(|r| r.addr)
    }

    /// 関所を起動する（F11）。`127.0.0.1:<port>` で待ち受け、実際のアドレスを返す。
    ///
    /// tokio の実行環境の中から呼ぶこと（Tauri の async コマンドの中なら OK）。
    pub async fn start(&self, port: u16, sink: ExchangeSink) -> Result<SocketAddr, ProxyError> {
        if self.status().is_some() {
            return Err(ProxyError::AlreadyRunning);
        }

        // 先に自分でポートを開いておく。hudsucker に任せると、ポートが使用中でも
        // 起動後の別タスクの中でしか失敗が分からず、画面にエラーを返せないため。
        // 127.0.0.1（ループバック）だけで待ち受けるので、ほかの PC からはつながらない（N6）
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .await
            .map_err(ProxyError::Bind)?;
        let addr = listener.local_addr().map_err(ProxyError::Bind)?;

        // HTTPS の中身を開くための認証局。いまは HTTPS を開かない（素通しする）ので、
        // 起動のたびにメモリ上で作る使い捨てのもので足りる
        let ca = cert::ephemeral_authority().map_err(ProxyError::Cert)?;

        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let proxy = Proxy::builder()
            .with_listener(listener)
            .with_ca(ca)
            // 本来の相手（API サーバー）へ渡すときの TLS（rustls）
            .with_rustls_connector(aws_lc_rs::default_provider())
            .with_http_handler(ObserveHandler::new(self.hosts.clone(), sink, addr))
            .with_graceful_shutdown(async {
                // stop_tx が送られる（または捨てられる）まで待つ
                let _ = stop_rx.await;
            })
            .build()
            .map_err(ProxyError::Build)?;

        // 関所は別タスクで動かし続ける。止まるのは stop が送られたとき
        tokio::spawn(async move {
            if let Err(e) = proxy.start().await {
                // 通信の中身は出さない（秘密情報が含まれ得るため）。エラーの種類だけ
                eprintln!("[proxy] 停止しました: {e}");
            }
        });

        *self.lock_running() = Some(Running {
            addr,
            stop: stop_tx,
        });
        Ok(addr)
    }

    /// 関所を止める（F11）。止まっていれば何もしない
    pub fn stop(&self) {
        if let Some(running) = self.lock_running().take() {
            let _ = running.stop.send(());
        }
    }

    fn lock_running(&self) -> std::sync::MutexGuard<'_, Option<Running>> {
        self.running.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener as StdListener;
    use std::time::Duration;

    /// 決まったレスポンスを返し続ける小さな HTTP サーバー（API サーバーの代わり）
    fn upstream(body: &'static str) -> SocketAddr {
        let listener = StdListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let res = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(res.as_bytes());
            }
        });
        addr
    }

    /// 関所を通して GET する（ブラウザの代わり）
    async fn get_via_proxy(proxy: SocketAddr, url: &str) -> String {
        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::http(format!("http://{proxy}")).unwrap())
            .build()
            .unwrap();
        client.get(url).send().await.unwrap().text().await.unwrap()
    }

    #[tokio::test]
    async fn records_only_registered_hosts() {
        let target = upstream(r#"{"user":"taro"}"#);
        let other = upstream("other");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Exchange>();
        let sink: ExchangeSink = Arc::new(move |ex| {
            let _ = tx.send(ex);
        });

        let controller = ProxyController::new();
        controller.set_hosts(vec![target.to_string()]);
        let addr = controller.start(0, sink).await.unwrap();
        assert!(addr.ip().is_loopback());

        // 対象外のホスト: ブラウザには普通に届くが、記録はされない
        assert_eq!(
            get_via_proxy(addr, &format!("http://{other}/x")).await,
            "other"
        );
        // 対象のホスト: 届いて、記録もされる
        let url = format!("http://{target}/api/users?id=1");
        assert_eq!(get_via_proxy(addr, &url).await, r#"{"user":"taro"}"#);

        let ex = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("1 秒以内に記録される（N4）")
            .unwrap();
        assert_eq!(ex.url, url);
        assert_eq!(ex.method, "GET");
        assert_eq!(ex.status, Some(200));
        assert_eq!(ex.response_body, r#"{"user":"taro"}"#);
        assert_eq!(ex.kind, crate::models::ExchangeKind::Observe);
        // 対象外の分は記録されていない
        assert!(rx.try_recv().is_err());

        controller.stop();
        assert!(controller.status().is_none());
    }
}
