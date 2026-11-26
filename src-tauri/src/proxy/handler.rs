//! 1 件ごとの記録処理。
//!
//! hudsucker は 1 つのリクエストとそのレスポンスに対して、**同じ** ハンドラーの複製を使う。
//! そこで `handle_request` で送った内容を `self.pending` に覚えておき、
//! `handle_response`（または `handle_error`）で返ってきた内容と組にして 1 件の記録にする。
//!
//! 流れ:
//! 1. `handle_request`: 宛先が観測対象か調べる。対象なら本文を読み切って覚え、相手へ渡す
//! 2. `handle_response`: 本文を読み切り、記録を作って `sink` に渡し、ブラウザへ返す
//! 3. `should_intercept_connect`: HTTPS（CONNECT）を開いて中を見るか。いまは開かずに素通し

use std::net::SocketAddr;
use std::time::Instant;

use http_body_util::BodyExt;
use hudsucker::hyper::header::{CONTENT_TYPE, HOST};
use hudsucker::hyper::{HeaderMap, Method, Request, Response, StatusCode, Uri};
use hudsucker::{Body, HttpContext, HttpHandler, RequestOrResponse, decode_response};

use super::{ExchangeSink, SharedHosts};
use crate::models::{BodyCapture, Exchange, ExchangeKind, HttpHeader, host_matches, now_ms};

/// 送った内容（レスポンスが来るまで覚えておく分）。
/// ハンドラーは `Clone` が必要なので、これも `Clone` にしておく
#[derive(Clone)]
struct Pending {
    started: Instant,
    exchange: Exchange,
}

#[derive(Clone)]
pub struct ObserveHandler {
    hosts: SharedHosts,
    sink: ExchangeSink,
    /// 関所自身のアドレス（自分宛ての通信を記録しないために使う）
    own_addr: SocketAddr,
    /// 記録中のリクエスト。観測対象でなければ None。
    /// hudsucker はリクエストごとにハンドラーを複製するので、
    /// ほかのリクエストの記録と混ざることはない
    pending: Option<Pending>,
}

impl ObserveHandler {
    pub fn new(hosts: SharedHosts, sink: ExchangeSink, own_addr: SocketAddr) -> Self {
        Self {
            hosts,
            sink,
            own_addr,
            pending: None,
        }
    }

    /// 宛先（`host:port`）が登録した観測対象ホストか
    fn is_target(&self, authority: &str) -> bool {
        if authority == self.own_addr.to_string() {
            return false;
        }
        self.hosts
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .any(|h| host_matches(h, authority))
    }
}

impl HttpHandler for ObserveHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        self.pending = None;

        // CONNECT（HTTPS のトンネルを張る依頼）そのものは記録しない。
        // 中を開くかどうかは should_intercept_connect で決める
        if req.method() == Method::CONNECT {
            return req.into();
        }
        let Some(authority) = request_authority(&req) else {
            return req.into();
        };
        // 観測対象でなければ、中身を読まずにそのまま渡す（F21）
        if !self.is_target(&authority) {
            return req.into();
        }

        let started_at = now_ms();
        let started = Instant::now();
        let (parts, body) = req.into_parts();
        let url = display_url(&parts.uri);

        // 本文を最後まで読む。相手に渡すために、読んだ本文は元の形のまま詰め直す
        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                // ブラウザからの本文を読めなかった。記録だけ残して 400 を返す
                let mut ex = new_exchange(started_at, parts.method.as_str(), url, &parts.headers);
                ex.error = Some(format!("ブラウザからの本文を読めませんでした（{e}）"));
                (self.sink)(ex);
                return plain_response(StatusCode::BAD_REQUEST, "api-lens: request body error")
                    .into();
            }
        };

        let mut ex = new_exchange(started_at, parts.method.as_str(), url, &parts.headers);
        let mut capture = BodyCapture::new();
        capture.push(&bytes);
        ex.body_truncated = capture.truncated();
        ex.request_body = capture.into_text();

        self.pending = Some(Pending {
            started,
            exchange: ex,
        });
        Request::from_parts(parts, Body::from(bytes)).into()
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        let Some(Pending {
            started,
            exchange: mut ex,
        }) = self.pending.take()
        else {
            return res; // 観測対象ではない
        };

        let (parts, body) = res.into_parts();
        ex.status = Some(parts.status.as_u16());
        ex.response_headers = header_list(&parts.headers);

        // Server-Sent Events のような「終わらない」レスポンスは読み切れないので、
        // 本文は記録せずにそのまま流す（読み切るまで待つとブラウザに何も届かなくなる）
        if is_streaming(&parts.headers) {
            ex.duration_ms = started.elapsed().as_millis() as u64;
            ex.response_body = "（ストリーミング応答のため本文は記録していません）".into();
            (self.sink)(ex);
            return Response::from_parts(parts, body);
        }

        let raw = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                ex.duration_ms = started.elapsed().as_millis() as u64;
                ex.error = Some(format!("サーバーからの本文を読めませんでした（{e}）"));
                (self.sink)(ex);
                return plain_response(StatusCode::BAD_GATEWAY, "api-lens: response body error");
            }
        };
        ex.duration_ms = started.elapsed().as_millis() as u64;

        // 表示用に、gzip などの圧縮を解いた本文を作る。
        // ブラウザへは「受け取ったままの」本文を返す（見るだけで、書き換えないため）
        let mut capture = BodyCapture::new();
        capture.push(&decoded_body(&parts.headers, raw.clone()).await);
        ex.size = capture.total();
        ex.body_truncated |= capture.truncated();
        ex.response_body = capture.into_text();
        (self.sink)(ex);

        Response::from_parts(parts, Body::from(raw))
    }

    async fn handle_error(
        &mut self,
        _ctx: &HttpContext,
        err: hudsucker::hyper_util::client::legacy::Error,
    ) -> Response<Body> {
        if let Some(Pending {
            started,
            exchange: mut ex,
        }) = self.pending.take()
        {
            ex.duration_ms = started.elapsed().as_millis() as u64;
            ex.error = Some(format!(
                "相手のサーバーに届きませんでした（{}）",
                error_chain(&err)
            ));
            (self.sink)(ex);
        }
        plain_response(StatusCode::BAD_GATEWAY, "api-lens: upstream error")
    }

    /// HTTPS の通信を開いて中を見るか。いまは開かない（中身を見ずにそのまま素通しする）
    async fn should_intercept_connect(&mut self, _ctx: &HttpContext, _req: &Request<Body>) -> bool {
        false
    }
}

// ----------------------------------------------------------------------
// 小さな道具
// ----------------------------------------------------------------------

fn new_exchange(started_at: i64, method: &str, url: String, headers: &HeaderMap) -> Exchange {
    Exchange {
        id: 0,
        started_at,
        kind: ExchangeKind::Observe,
        method: method.to_string(),
        url,
        request_headers: header_list(headers),
        request_body: String::new(),
        status: None,
        response_headers: Vec::new(),
        response_body: String::new(),
        duration_ms: 0,
        size: 0,
        body_truncated: false,
        error: None,
    }
}

/// リクエストの宛先（`host:port`）。
/// プロキシ宛てのリクエストは `GET http://localhost:3000/api` のように URL 全体が来るので
/// そこから取る。なければ Host ヘッダーを見る
fn request_authority(req: &Request<Body>) -> Option<String> {
    if let Some(a) = req.uri().authority() {
        return Some(a.as_str().to_lowercase());
    }
    req.headers()
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase())
}

/// 画面に出す URL。既定のポート（http:80 / https:443）は省く
fn display_url(uri: &Uri) -> String {
    let scheme = uri.scheme_str().unwrap_or("http");
    let host = uri.host().unwrap_or("");
    let path = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let default_port = if scheme == "https" { 443 } else { 80 };
    match uri.port_u16() {
        Some(port) if port != default_port => format!("{scheme}://{host}:{port}{path}"),
        _ => format!("{scheme}://{host}{path}"),
    }
}

fn header_list(headers: &HeaderMap) -> Vec<HttpHeader> {
    headers
        .iter()
        .map(|(k, v)| {
            HttpHeader::new(
                k.as_str(),
                String::from_utf8_lossy(v.as_bytes()).into_owned(),
            )
        })
        .collect()
}

fn is_streaming(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("text/event-stream"))
}

/// Content-Encoding（gzip / br / deflate / zstd）を解いた本文。解けなければ元のまま
async fn decoded_body(headers: &HeaderMap, raw: hudsucker::hyper::body::Bytes) -> Vec<u8> {
    let mut res = Response::new(Body::from(raw.clone()));
    *res.headers_mut() = headers.clone();
    match decode_response(res) {
        Ok(decoded) => match decoded.into_body().collect().await {
            Ok(c) => c.to_bytes().to_vec(),
            Err(_) => raw.to_vec(),
        },
        Err(_) => raw.to_vec(),
    }
}

fn plain_response(status: StatusCode, text: &'static str) -> Response<Body> {
    let mut res = Response::new(Body::from(text));
    *res.status_mut() = status;
    res
}

/// エラーの根本原因までつなげた説明（例: "client error (Connect) / Connection refused"）
fn error_chain(err: &dyn std::error::Error) -> String {
    let mut s = err.to_string();
    let mut source = err.source();
    while let Some(e) = source {
        s.push_str(&format!(" / {e}"));
        source = e.source();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_url_hides_default_port() {
        let u: Uri = "https://api.example.com:443/v1?x=1".parse().unwrap();
        assert_eq!(display_url(&u), "https://api.example.com/v1?x=1");
        let u: Uri = "http://localhost:3000/api".parse().unwrap();
        assert_eq!(display_url(&u), "http://localhost:3000/api");
    }
}
