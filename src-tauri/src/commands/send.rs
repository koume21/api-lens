//! 送信モードのコマンド

use tauri::State;

use super::AppState;
use crate::models::{Exchange, RequestInput};

/// 入力されたリクエストを送り、結果を返す（F1〜F8）。
///
/// JS からは `invoke("send_request", { input })` で呼ぶ。
/// 送信の失敗（タイムアウトなど）は `Exchange.error` に入るので、ここでは `Err` にならない。
/// async コマンドで `State` のような借用を受け取るときは、戻り値を `Result` にする必要がある（Tauri の決まり）。
#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    input: RequestInput,
) -> Result<Exchange, String> {
    Ok(state.client.send(&input).await)
}
