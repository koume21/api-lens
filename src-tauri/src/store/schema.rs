//! 表の作成と更新（マイグレーション）。
//!
//! SQLite の `PRAGMA user_version`（DB ファイルに 1 つだけ持てる整数）を
//! 「いまの DB がどこまで更新済みか」の印として使う。
//! 起動のたびに、まだ当てていない MIGRATIONS を順番に当てていく。
//!
//! 表の形を変えたいときは、既存の要素を書き換えずに **末尾に追加** する。
//! （すでに配布した DB は古い番号のまま残っているので、過去の手順を変えると壊れる）

use rusqlite::Connection;

/// 1 要素 = 1 バージョン分の SQL。添字 + 1 がそのバージョン番号
const MIGRATIONS: &[&str] = &[
    // v1: 表1（通信の記録）と表2（保存したリクエスト）
    "
    CREATE TABLE exchanges (
        id               INTEGER PRIMARY KEY AUTOINCREMENT,
        started_at       INTEGER NOT NULL,          -- UNIX 時刻（ミリ秒）
        kind             TEXT    NOT NULL,          -- 'observe' / 'send'
        method           TEXT    NOT NULL,
        url              TEXT    NOT NULL,
        request_headers  TEXT    NOT NULL,          -- [{name, value}] の JSON
        request_body     TEXT    NOT NULL,
        status           INTEGER,                   -- レスポンスがなければ NULL
        response_headers TEXT    NOT NULL,
        response_body    TEXT    NOT NULL,
        duration_ms      INTEGER NOT NULL,
        size             INTEGER NOT NULL,          -- レスポンス本文の元のサイズ（バイト）
        body_truncated   INTEGER NOT NULL,          -- 本文省略フラグ（0 / 1）
        error            TEXT
    );
    CREATE INDEX idx_exchanges_kind ON exchanges(kind, id);

    CREATE TABLE saved_requests (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        name       TEXT    NOT NULL,
        method     TEXT    NOT NULL,
        url        TEXT    NOT NULL,
        headers    TEXT    NOT NULL,                -- [{name, value}] の JSON
        body       TEXT    NOT NULL,
        created_at INTEGER NOT NULL
    );
    ",
];

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    // rusqlite は usize を直接読み書きできないので i64 で受ける
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let version = (i + 1) as i64;
        // 1 バージョン分は「全部当たる」か「何も当たらない」かのどちらかにする
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}
