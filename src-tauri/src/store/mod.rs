//! 保存係。SQLite（rusqlite）に通信の記録・保存したリクエストを残す。
//!
//! Tauri や画面のことは知らない（DB ファイルの場所は `lib.rs` が決めて渡す）。
//! 保存先は PC の中（アプリのデータフォルダ）だけで、外には送らない（N5）。
//!
//! 表（`docs/05-design.md` の DB 設計）:
//! - `exchanges`:      表1 通信の記録（観測と送信の両方。最新 1,000 件まで）
//! - `saved_requests`: 表2 保存したリクエスト
//! - `observed_hosts`: 表3 観測対象ホスト
//!
//! ヘッダーは名前と値のペアの配列を JSON 文字列にして 1 列に入れる。

mod schema;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::models::{
    Exchange, ExchangeKind, HttpHeader, ObservedHost, RequestInput, SavedRequest, now_ms,
};

/// 通信の記録を残す最大件数（N2）
pub const MAX_EXCHANGES: i64 = 1_000;

/// 保存係でのエラー。画面には文字列にして返す
#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    NotFound,
    /// 同じものがすでにある（UNIQUE 制約に当たった）
    Duplicate,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "データベースの操作に失敗しました: {e}"),
            StoreError::Json(e) => write!(f, "保存データの読み書きに失敗しました: {e}"),
            StoreError::NotFound => write!(f, "対象のデータが見つかりません"),
            StoreError::Duplicate => write!(f, "同じものがすでに登録されています"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sqlite(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

pub type StoreResult<T> = Result<T, StoreError>;

/// 保存係本体。
///
/// `rusqlite::Connection` は複数スレッドから同時に使えない（`Sync` でない）ので、
/// `Mutex` で包んで 1 度に 1 つの処理だけが触るようにする。
/// 1 回の操作は短いので、ロック待ちが問題になることはない想定。
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// DB ファイルを開く（なければ作る）。表の作成・更新（マイグレーション）もここで行う
    pub fn open(path: &Path) -> StoreResult<Self> {
        Self::init(Connection::open(path)?)
    }

    /// テスト用：メモリ上の使い捨て DB
    #[cfg(test)]
    pub fn open_in_memory() -> StoreResult<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> StoreResult<Self> {
        schema::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// ロックを取る。別スレッドがロック中にパニックして「毒入り」になっていても、
    /// DB 自体は壊れていないので中身を取り出して使い続ける
    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ------------------------------------------------------------------
    // 表1：通信の記録
    // ------------------------------------------------------------------

    /// 通信 1 件を記録し、振られた番号を返す。
    /// 記録したあと、1,000 件を超えた古いものを消す（N2）
    pub fn insert_exchange(&self, ex: &Exchange) -> StoreResult<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO exchanges (
                started_at, kind, method, url,
                request_headers, request_body,
                status, response_headers, response_body,
                duration_ms, size, body_truncated, error
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                ex.started_at,
                kind_to_str(ex.kind),
                ex.method,
                ex.url,
                serde_json::to_string(&ex.request_headers)?,
                ex.request_body,
                ex.status,
                serde_json::to_string(&ex.response_headers)?,
                ex.response_body,
                ex.duration_ms as i64,
                ex.size as i64,
                ex.body_truncated,
                ex.error,
            ],
        )?;
        let id = conn.last_insert_rowid();

        // 新しい順に 1,000 件目より古い（番号が小さい）ものを消す。
        // id は増える一方なので「id の大きさ = 新しさ」として扱える
        conn.execute(
            "DELETE FROM exchanges WHERE id <= (
                SELECT id FROM exchanges ORDER BY id DESC LIMIT 1 OFFSET ?1
             )",
            params![MAX_EXCHANGES],
        )?;
        Ok(id)
    }

    /// 通信の記録を新しい順に取り出す。`kind` を指定するとその種類だけ
    pub fn list_exchanges(
        &self,
        kind: Option<ExchangeKind>,
        limit: i64,
    ) -> StoreResult<Vec<Exchange>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!(
            "SELECT {EXCHANGE_COLUMNS} FROM exchanges
             WHERE (?1 IS NULL OR kind = ?1)
             ORDER BY id DESC LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![kind.map(kind_to_str), limit], row_to_exchange)?;
        collect_rows(rows)
    }

    // ------------------------------------------------------------------
    // 表2：保存したリクエスト
    // ------------------------------------------------------------------

    pub fn insert_saved_request(
        &self,
        name: &str,
        req: &RequestInput,
    ) -> StoreResult<SavedRequest> {
        let conn = self.conn();
        let created_at = now_ms();
        conn.execute(
            "INSERT INTO saved_requests (name, method, url, headers, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                name,
                req.method,
                req.url,
                serde_json::to_string(&req.headers)?,
                req.body,
                created_at
            ],
        )?;
        Ok(SavedRequest {
            id: conn.last_insert_rowid(),
            name: name.to_string(),
            request: req.clone(),
            created_at,
        })
    }

    /// 保存済みのリクエストを上書きする（作った日時は変えない）
    pub fn update_saved_request(
        &self,
        id: i64,
        name: &str,
        req: &RequestInput,
    ) -> StoreResult<SavedRequest> {
        let conn = self.conn();
        let changed = conn.execute(
            "UPDATE saved_requests SET name = ?2, method = ?3, url = ?4, headers = ?5, body = ?6
             WHERE id = ?1",
            params![
                id,
                name,
                req.method,
                req.url,
                serde_json::to_string(&req.headers)?,
                req.body
            ],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound);
        }
        let created_at: i64 = conn
            .query_row(
                "SELECT created_at FROM saved_requests WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(StoreError::NotFound)?;
        Ok(SavedRequest {
            id,
            name: name.to_string(),
            request: req.clone(),
            created_at,
        })
    }

    /// 保存したリクエストを名前順に取り出す
    pub fn list_saved_requests(&self) -> StoreResult<Vec<SavedRequest>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, method, url, headers, body, created_at
             FROM saved_requests ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?;
        let mut list = Vec::new();
        for row in rows {
            let (id, name, method, url, headers, body, created_at) = row?;
            list.push(SavedRequest {
                id,
                name,
                request: RequestInput {
                    method,
                    url,
                    headers: serde_json::from_str(&headers)?,
                    body,
                },
                created_at,
            });
        }
        Ok(list)
    }

    pub fn delete_saved_request(&self, id: i64) -> StoreResult<()> {
        self.conn()
            .execute("DELETE FROM saved_requests WHERE id = ?1", [id])?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // 表3：観測対象ホスト
    // ------------------------------------------------------------------

    /// 観測対象ホストを登録する。`host` は `models::normalize_host` で整えてから渡すこと
    pub fn insert_host(&self, label: &str, host: &str) -> StoreResult<ObservedHost> {
        let conn = self.conn();
        let created_at = now_ms();
        let result = conn.execute(
            "INSERT INTO observed_hosts (label, host, created_at) VALUES (?1, ?2, ?3)",
            params![label, host, created_at],
        );
        match result {
            Ok(_) => {}
            // UNIQUE 制約違反は「もう登録済み」として分かりやすいエラーにする
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                return Err(StoreError::Duplicate);
            }
            Err(e) => return Err(e.into()),
        }
        Ok(ObservedHost {
            id: conn.last_insert_rowid(),
            label: label.to_string(),
            host: host.to_string(),
            created_at,
        })
    }

    /// 観測対象ホストを登録順に取り出す
    pub fn list_hosts(&self) -> StoreResult<Vec<ObservedHost>> {
        let conn = self.conn();
        let mut stmt =
            conn.prepare("SELECT id, label, host, created_at FROM observed_hosts ORDER BY id")?;
        let rows = stmt.query_map([], |r| {
            Ok(ObservedHost {
                id: r.get(0)?,
                label: r.get(1)?,
                host: r.get(2)?,
                created_at: r.get(3)?,
            })
        })?;
        collect_rows(rows)
    }

    pub fn delete_host(&self, id: i64) -> StoreResult<()> {
        self.conn()
            .execute("DELETE FROM observed_hosts WHERE id = ?1", [id])?;
        Ok(())
    }
}

// ----------------------------------------------------------------------
// 行 ⇔ 構造体の変換
// ----------------------------------------------------------------------

/// `row_to_exchange` が読む列の並び。SELECT 文はこの定数を使って列の順番をそろえる
const EXCHANGE_COLUMNS: &str = "id, started_at, kind, method, url,
    request_headers, request_body, status, response_headers, response_body,
    duration_ms, size, body_truncated, error";

fn kind_to_str(kind: ExchangeKind) -> &'static str {
    match kind {
        ExchangeKind::Observe => "observe",
        ExchangeKind::Send => "send",
    }
}

/// JSON 文字列の列を読む。`query_map` の中では `rusqlite::Error` しか返せないので、
/// serde_json のエラーを rusqlite の「変換失敗」エラーに包む
fn json_column(row: &Row, idx: usize) -> rusqlite::Result<Vec<HttpHeader>> {
    let text: String = row.get(idx)?;
    serde_json::from_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn row_to_exchange(row: &Row) -> rusqlite::Result<Exchange> {
    let kind: String = row.get(2)?;
    Ok(Exchange {
        id: row.get(0)?,
        started_at: row.get(1)?,
        kind: if kind == "observe" {
            ExchangeKind::Observe
        } else {
            ExchangeKind::Send
        },
        method: row.get(3)?,
        url: row.get(4)?,
        request_headers: json_column(row, 5)?,
        request_body: row.get(6)?,
        status: row.get(7)?,
        response_headers: json_column(row, 8)?,
        response_body: row.get(9)?,
        duration_ms: row.get::<_, i64>(10)? as u64,
        size: row.get::<_, i64>(11)? as u64,
        body_truncated: row.get(12)?,
        error: row.get(13)?,
    })
}

fn collect_rows<T>(rows: impl Iterator<Item = rusqlite::Result<T>>) -> StoreResult<Vec<T>> {
    rows.collect::<rusqlite::Result<Vec<T>>>()
        .map_err(StoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_exchange(url: &str) -> Exchange {
        Exchange {
            id: 0,
            started_at: now_ms(),
            kind: ExchangeKind::Send,
            method: "GET".into(),
            url: url.into(),
            request_headers: vec![HttpHeader::new("X-A", "1")],
            request_body: String::new(),
            status: Some(200),
            response_headers: vec![HttpHeader::new("content-type", "application/json")],
            response_body: "{}".into(),
            duration_ms: 12,
            size: 2,
            body_truncated: false,
            error: None,
        }
    }

    fn sample_request() -> RequestInput {
        RequestInput {
            method: "POST".into(),
            url: "http://localhost:3000/api".into(),
            headers: vec![HttpHeader::new("Authorization", "Bearer t")],
            body: r#"{"a":1}"#.into(),
        }
    }

    #[test]
    fn exchange_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let id = store
            .insert_exchange(&sample_exchange("http://a/"))
            .unwrap();
        let list = store.list_exchanges(None, 10).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].request_headers, vec![HttpHeader::new("X-A", "1")]);
        assert_eq!(list[0].status, Some(200));
        // 種類での絞り込み
        assert!(
            store
                .list_exchanges(Some(ExchangeKind::Observe), 10)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn keeps_only_latest_exchanges() {
        let store = Store::open_in_memory().unwrap();
        for i in 0..(MAX_EXCHANGES + 5) {
            store
                .insert_exchange(&sample_exchange(&format!("http://a/{i}")))
                .unwrap();
        }
        let list = store.list_exchanges(None, MAX_EXCHANGES * 2).unwrap();
        assert_eq!(list.len() as i64, MAX_EXCHANGES);
        // 一番古い 5 件（0〜4）が消え、最新が先頭
        assert_eq!(list[0].url, format!("http://a/{}", MAX_EXCHANGES + 4));
        assert_eq!(list.last().unwrap().url, "http://a/5");
    }

    #[test]
    fn saved_request_crud() {
        let store = Store::open_in_memory().unwrap();
        let saved = store
            .insert_saved_request("ユーザー作成", &sample_request())
            .unwrap();

        let list = store.list_saved_requests().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].request.headers, sample_request().headers);

        let mut changed = sample_request();
        changed.url = "http://localhost:3000/v2".into();
        let updated = store
            .update_saved_request(saved.id, "改名", &changed)
            .unwrap();
        assert_eq!(updated.created_at, saved.created_at);
        assert_eq!(
            store.list_saved_requests().unwrap()[0].request.url,
            "http://localhost:3000/v2"
        );

        store.delete_saved_request(saved.id).unwrap();
        assert!(store.list_saved_requests().unwrap().is_empty());
        assert!(matches!(
            store.update_saved_request(saved.id, "x", &changed),
            Err(StoreError::NotFound)
        ));
    }

    #[test]
    fn hosts_crud_and_duplicate() {
        let store = Store::open_in_memory().unwrap();
        let h = store.insert_host("ローカル", "localhost:3000").unwrap();
        assert!(matches!(
            store.insert_host("もう一度", "localhost:3000"),
            Err(StoreError::Duplicate)
        ));
        store.insert_host("本番", "api.example.com").unwrap();
        let list = store.list_hosts().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].label, "ローカル");
        store.delete_host(h.id).unwrap();
        assert_eq!(store.list_hosts().unwrap().len(), 1);
    }

    #[test]
    fn reopening_file_keeps_data() {
        // F10 の完了条件「再起動しても残っている」をファイル DB で確かめる
        let dir = std::env::temp_dir().join(format!("api-lens-test-{}", now_ms()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        {
            let store = Store::open(&path).unwrap();
            store.insert_saved_request("a", &sample_request()).unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert_eq!(store.list_saved_requests().unwrap().len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
