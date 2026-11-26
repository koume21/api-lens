//! 送信役。送信画面で組み立てたリクエストを reqwest で送る（F1〜F8）。
//!
//! Tauri や画面のことは知らない（`commands` から呼ばれるだけ）。
//! ブラウザの fetch ではなく Rust 側で送るので、CORS の制限を受けず、
//! `Cookie` や `Host` などブラウザが付けさせてくれないヘッダーも自由に付けられる。

use std::time::{Duration, Instant};

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};

use crate::models::{BodyCapture, Exchange, ExchangeKind, HttpHeader, RequestInput, now_ms};

/// 1 回の送信で待つ最長時間。超えたらタイムアウトとして失敗扱いにする
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// 送信役本体。中の `reqwest::Client` は接続を使い回すので、アプリ全体で 1 つだけ作る。
pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let inner = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            // OS のプロキシ設定を使わない。観測のために OS 全体のプロキシを
            // 自分の関所に向けていると、送信モードの通信まで関所を通ってしまうため
            .no_proxy()
            .build()?;
        Ok(Self { inner })
    }

    /// リクエストを送り、結果を `Exchange` にして返す。
    ///
    /// 失敗しても `Err` にはせず、`error` に理由を入れた `Exchange` を返す。
    /// 失敗した通信も「結果」として画面に出したり履歴に残したりしたいため。
    pub async fn send(&self, input: &RequestInput) -> Exchange {
        let mut exchange = Exchange {
            id: 0,
            started_at: now_ms(),
            kind: ExchangeKind::Send,
            method: input.method.trim().to_uppercase(),
            url: input.url.trim().to_string(),
            request_headers: Vec::new(),
            request_body: input.body.clone(),
            status: None,
            response_headers: Vec::new(),
            response_body: String::new(),
            duration_ms: 0,
            size: 0,
            body_truncated: false,
            error: None,
        };

        // --- 1. 入力を reqwest の形に直す（ここで失敗したら送らずに終わる） ---
        let method = match Method::from_bytes(exchange.method.as_bytes()) {
            Ok(m) => m,
            Err(_) => {
                exchange.error = Some(format!("メソッドが正しくありません: {}", exchange.method));
                return exchange;
            }
        };
        let (headers, sent_headers) = match build_headers(input) {
            Ok(v) => v,
            Err(msg) => {
                exchange.error = Some(msg);
                return exchange;
            }
        };
        // 実際に付けたヘッダー（自動で足した Content-Type を含む）を記録する
        exchange.request_headers = sent_headers;

        let mut builder = self.inner.request(method, &exchange.url).headers(headers);
        if has_body(input) {
            builder = builder.body(input.body.clone());
        }

        // --- 2. 送って、本文を最後まで受け取る ---
        // 時間は「送る直前」から「本文を受け取り終わった時点」まで測る。
        // ヘッダーが届いた時点で止めると、大きな本文のダウンロード時間が入らないため。
        let started = Instant::now();
        let result = async {
            let mut res = builder.send().await?;
            let status = res.status().as_u16();
            let headers = header_list(res.headers());

            // chunk() で少しずつ受け取り、10MB を超えた分は捨てる（N3）
            let mut body = BodyCapture::new();
            while let Some(chunk) = res.chunk().await? {
                body.push(&chunk);
            }
            Ok::<_, reqwest::Error>((status, headers, body))
        }
        .await;
        exchange.duration_ms = started.elapsed().as_millis() as u64;

        // --- 3. 結果を詰める ---
        match result {
            Ok((status, headers, body)) => {
                exchange.status = Some(status);
                exchange.response_headers = headers;
                exchange.size = body.total();
                exchange.body_truncated = body.truncated();
                exchange.response_body = body.into_text();
            }
            Err(e) => exchange.error = Some(describe_error(&e)),
        }
        exchange
    }
}

/// 本文が入力されていれば付ける。
/// GET に本文を付けるのは珍しいが HTTP として禁止ではないので、メソッドでは止めない
fn has_body(input: &RequestInput) -> bool {
    !input.body.trim().is_empty()
}

/// 入力されたヘッダーを `HeaderMap` にする。
///
/// - 名前が空の行は「入力途中」とみなして無視する
/// - 本文があるのに Content-Type が指定されていなければ `application/json` を足す（F4 は JSON 本文）
///
/// 戻り値は（reqwest に渡すもの, 記録用の一覧）
fn build_headers(input: &RequestInput) -> Result<(HeaderMap, Vec<HttpHeader>), String> {
    let mut map = HeaderMap::new();
    let mut list = Vec::new();

    for h in &input.headers {
        let name = h.name.trim();
        if name.is_empty() {
            continue;
        }
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| format!("ヘッダー名が正しくありません: {name}"))?;
        let header_value = HeaderValue::from_str(h.value.trim())
            .map_err(|_| format!("ヘッダー「{name}」の値に使えない文字が含まれています"))?;
        // insert ではなく append にして、同じ名前を複数付けられるようにする
        map.append(header_name, header_value);
        list.push(HttpHeader::new(name, h.value.trim()));
    }

    if has_body(input) && !map.contains_key(CONTENT_TYPE) {
        map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        list.push(HttpHeader::new("content-type", "application/json"));
    }

    Ok((map, list))
}

/// `HeaderMap` を画面に出せる一覧にする（値が UTF-8 でなければ lossy に読む）
pub fn header_list(map: &HeaderMap) -> Vec<HttpHeader> {
    map.iter()
        .map(|(name, value)| {
            HttpHeader::new(
                name.as_str(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect()
}

/// reqwest のエラーを、何が起きたか分かる日本語にする（エラーの種類分け）。
///
/// reqwest の Display は一番外側の説明しか出さないので、`source()` をたどって
/// 根本原因（例: "Connection refused"）も後ろに付ける。
fn describe_error(e: &reqwest::Error) -> String {
    let kind = if e.is_timeout() {
        format!("タイムアウトしました（{} 秒）", REQUEST_TIMEOUT.as_secs())
    } else if e.is_builder() {
        "リクエストを組み立てられませんでした。URL の形式（http:// から始まるか）を確認してください"
            .into()
    } else if e.is_connect() {
        "接続できませんでした。サーバーが起動しているか、URL が正しいか確認してください".into()
    } else if e.is_redirect() {
        "リダイレクトが多すぎます".into()
    } else if e.is_body() || e.is_decode() {
        "レスポンス本文の受信に失敗しました".into()
    } else {
        "送信に失敗しました".into()
    };

    let mut detail = e.to_string();
    let mut source = std::error::Error::source(e);
    while let Some(s) = source {
        detail.push_str(&format!(" / {s}"));
        source = s.source();
    }
    format!("{kind}（{detail}）")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// 1 回だけ決まったレスポンスを返す小さな HTTP サーバーを立てる。
    /// 受け取ったリクエストの生データを返すので、ヘッダーや本文が届いたか確かめられる。
    fn one_shot_server(response: &'static str) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/test", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });
        (url, handle)
    }

    fn input(method: &str, url: &str, body: &str) -> RequestInput {
        RequestInput {
            method: method.into(),
            url: url.into(),
            headers: vec![
                HttpHeader::new("X-Test", "abc"),
                // 名前が空の行は無視される
                HttpHeader::new("", "ignored"),
            ],
            body: body.into(),
        }
    }

    #[tokio::test]
    async fn sends_post_with_headers_and_json_body() {
        let (url, server) = one_shot_server(
            "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\n{\"ok\":true}",
        );
        let client = HttpClient::new().unwrap();
        let ex = client.send(&input("post", &url, r#"{"name":"a"}"#)).await;

        let raw = server.join().unwrap().to_lowercase();
        assert!(raw.starts_with("post /test"));
        assert!(raw.contains("x-test: abc"));
        assert!(raw.contains("content-type: application/json"));
        assert!(raw.contains(r#"{"name":"a"}"#));

        assert_eq!(ex.error, None);
        assert_eq!(ex.method, "POST");
        assert_eq!(ex.status, Some(201));
        assert_eq!(ex.response_body, r#"{"ok":true}"#);
        assert_eq!(ex.size, 11);
        assert!(!ex.body_truncated);
    }

    #[tokio::test]
    async fn connection_error_is_reported_in_exchange() {
        // 一度開いてすぐ閉じたポートには誰もいないので、接続に失敗する
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let client = HttpClient::new().unwrap();
        let ex = client
            .send(&input("GET", &format!("http://127.0.0.1:{port}/"), ""))
            .await;
        assert_eq!(ex.status, None);
        assert!(ex.error.unwrap().starts_with("接続できませんでした"));
    }

    #[tokio::test]
    async fn invalid_method_is_rejected_before_sending() {
        let client = HttpClient::new().unwrap();
        let ex = client.send(&input("GE T", "http://127.0.0.1/", "")).await;
        assert!(ex.error.unwrap().contains("メソッド"));
    }
}
