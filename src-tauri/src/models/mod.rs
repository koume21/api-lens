//! データの形（通信1件・送信の入力など）。
//!
//! 画面（`src/types/index.ts`）と 1 対 1 で対応させている。
//! 画面とやり取りする構造体は `#[serde(rename_all = "camelCase")]` を付けて、
//! JS 側で `requestHeaders` のような camelCase で扱えるようにする。
//! （Tauri が自動で camelCase に変換するのはコマンドの「引数名」だけで、
//!   構造体の中のフィールド名は変換されないため）

use serde::{Deserialize, Serialize};

/// 本文として保持する最大バイト数（N3: 10MB を超えたら先頭だけを残す）
pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// ヘッダー 1 つ（名前と値のペア）。
///
/// `HashMap` にしないのは、HTTP では同じ名前のヘッダーが複数回出てよく
/// （例: `Set-Cookie`）、並び順も見せたいため。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// 送信画面で入力されたリクエスト（F1〜F4）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestInput {
    /// GET / POST / PUT / PATCH / DELETE
    pub method: String,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    /// JSON 本文。空文字なら本文なしで送る
    pub body: String,
}

/// 名前を付けて保存したリクエスト（F9・F10）。DB の「表2：保存したリクエスト」に対応する。
///
/// 「リクエストの定義」（何を送るか）だけを持ち、送った結果は持たない。
/// 結果は送るたびに変わるので、通信の記録（`Exchange`）の方に残す。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedRequest {
    pub id: i64,
    pub name: String,
    pub request: RequestInput,
    pub created_at: i64,
}

/// 観測対象ホスト（F21・F22）。DB の「表3：観測対象ホスト」に対応する。
/// 関所はここに登録したホストへの通信だけを記録し、それ以外は中身を開かずに素通しする。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservedHost {
    pub id: i64,
    /// 表示名（例: ローカル、本番）
    pub label: String,
    /// ホスト（例: `localhost:3000`、`api.example.com`）。`normalize_host` で整えた形で持つ
    pub host: String,
    pub created_at: i64,
}

/// 入力されたホストを比べやすい形に整える。
///
/// - 前後の空白を落とし、小文字にする（ホスト名は大文字小文字を区別しない）
/// - `http://` などのスキームと、`/` 以降のパスを取り除く
///   （URL をそのまま貼り付けても登録できるように）
///
/// 空になったり空白を含んだりするなら None
pub fn normalize_host(input: &str) -> Option<String> {
    let mut s = input.trim().to_lowercase();
    if let Some(pos) = s.find("://") {
        s = s[pos + 3..].to_string();
    }
    if let Some(pos) = s.find(['/', '?', '#']) {
        s.truncate(pos);
    }
    if s.is_empty() || s.contains(char::is_whitespace) {
        return None;
    }
    Some(s)
}

/// 通信の種類。観測（ブラウザ → 関所）か、送信（送信画面 → 送信役）か
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExchangeKind {
    Observe,
    Send,
}

/// 通信 1 件（リクエストとレスポンスの組）。DB の「表1：通信の記録」に対応する。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    /// DB の番号。まだ保存していないときは 0
    pub id: i64,
    /// 通信を始めた日時（UNIX 時刻・ミリ秒）。JS の `new Date(ms)` にそのまま渡せる
    pub started_at: i64,
    pub kind: ExchangeKind,
    pub method: String,
    pub url: String,
    pub request_headers: Vec<HttpHeader>,
    pub request_body: String,
    /// ステータスコード。接続できなかったなど、レスポンスがないときは None
    pub status: Option<u16>,
    pub response_headers: Vec<HttpHeader>,
    pub response_body: String,
    /// リクエストを送ってからレスポンス本文を受け取り終わるまで（ミリ秒）
    pub duration_ms: u64,
    /// レスポンス本文の実際のサイズ（バイト）。省略したときも元のサイズを入れる
    pub size: u64,
    /// 本文が 10MB を超えて先頭だけを残したか（N3）
    pub body_truncated: bool,
    /// 失敗したときの理由（タイムアウト・接続失敗など）
    pub error: Option<String>,
}

/// 本文を貯めながら、10MB を超えた分は捨てて「全体のサイズ」だけ数える入れ物。
///
/// 送信役（client）と関所（proxy）の両方で使う。
/// 本文を全部メモリに持ってから切ると、巨大なレスポンスでメモリを使い切るおそれがあるため、
/// 受け取りながら上限で止める。
#[derive(Debug, Default)]
pub struct BodyCapture {
    buf: Vec<u8>,
    total: u64,
}

impl BodyCapture {
    pub fn new() -> Self {
        Self::default()
    }

    /// 受け取った断片（チャンク）を追加する
    pub fn push(&mut self, chunk: &[u8]) {
        self.total += chunk.len() as u64;
        let room = MAX_BODY_BYTES.saturating_sub(self.buf.len());
        let take = room.min(chunk.len());
        self.buf.extend_from_slice(&chunk[..take]);
    }

    /// これまでに受け取った全体のサイズ（捨てた分も含む）
    pub fn total(&self) -> u64 {
        self.total
    }

    /// 上限を超えて捨てた分があるか
    pub fn truncated(&self) -> bool {
        self.total > self.buf.len() as u64
    }

    /// 画面に出す文字列にする。
    ///
    /// UTF-8 として読めないバイト（画像など）は「�」に置き換わる。
    /// 見るだけのアプリなので、バイナリを正確に残すことはしない（制約として README に書く）。
    pub fn into_text(self) -> String {
        String::from_utf8_lossy(&self.buf).into_owned()
    }
}

/// 今の時刻（UNIX 時刻・ミリ秒）
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_host_strips_scheme_and_path() {
        assert_eq!(
            normalize_host(" http://LocalHost:3000/api?x=1 "),
            Some("localhost:3000".into())
        );
        assert_eq!(
            normalize_host("https://api.example.com"),
            Some("api.example.com".into())
        );
        assert_eq!(
            normalize_host("api.example.com/"),
            Some("api.example.com".into())
        );
        assert_eq!(normalize_host("   "), None);
        assert_eq!(normalize_host("a b"), None);
    }

    #[test]
    fn body_capture_keeps_small_body() {
        let mut c = BodyCapture::new();
        c.push(b"hello ");
        c.push(b"world");
        assert_eq!(c.total(), 11);
        assert!(!c.truncated());
        assert_eq!(c.into_text(), "hello world");
    }

    #[test]
    fn body_capture_truncates_over_limit() {
        let mut c = BodyCapture::new();
        // 上限ちょうど + 1 バイトを 2 回に分けて入れる
        c.push(&vec![b'a'; MAX_BODY_BYTES - 1]);
        c.push(b"bc");
        assert_eq!(c.total(), MAX_BODY_BYTES as u64 + 1);
        assert!(c.truncated());
        let text = c.into_text();
        assert_eq!(text.len(), MAX_BODY_BYTES);
        assert!(text.ends_with('b'));
    }
}
