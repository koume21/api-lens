//! 受付。画面（`invoke`）からの依頼を受けて、送信役・関所・保存係に振り分ける。
//!
//! 画面とやり取りするのはこのモジュールだけ。ここで Tauri の型（`State` など）を
//! 受け取り、`client` / `proxy` / `store` には Tauri に依存しない形で渡す。
//!
//! コマンドはすべて `async fn` にしている。Tauri では async でないコマンドは
//! メインスレッドで動くため、DB 操作などで少しでも待つと画面が固まってしまう。

pub mod hosts;
pub mod saved;
pub mod send;

use crate::client::HttpClient;
use crate::store::Store;

/// アプリ全体で共有する部品。`lib.rs` で `.manage()` に渡し、
/// 各コマンドは `tauri::State<AppState>` で受け取る。
pub struct AppState {
    pub client: HttpClient,
    pub store: Store,
}

/// コマンドの戻り値。JS 側では失敗が例外（`invoke` の Promise の reject）になり、
/// `catch` で受け取るとこの文字列が入っている
pub type CmdResult<T> = Result<T, String>;

/// エラーを画面に返す文字列にする（`map_err(to_message)` の形で使う）
pub fn to_message(e: impl std::fmt::Display) -> String {
    e.to_string()
}
