//! 受付。画面（`invoke`）からの依頼を受けて、送信役・関所・保存係に振り分ける。
//!
//! 画面とやり取りするのはこのモジュールだけ。ここで Tauri の型（`State` など）を
//! 受け取り、`client` / `proxy` / `store` には Tauri に依存しない形で渡す。

pub mod send;

use crate::client::HttpClient;

/// アプリ全体で共有する部品。`lib.rs` で `.manage()` に渡し、
/// 各コマンドは `tauri::State<AppState>` で受け取る。
pub struct AppState {
    pub client: HttpClient,
}
