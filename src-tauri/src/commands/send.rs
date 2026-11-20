//! 送信モードのコマンド

use tauri::State;

use super::{AppState, CmdResult, to_message};
use crate::models::{Exchange, ExchangeKind, RequestInput};

/// 送信履歴として画面に出す件数
const SEND_HISTORY_LIMIT: i64 = 100;

/// 入力されたリクエストを送り、結果を通信の記録に残してから返す（F1〜F8）。
///
/// JS からは `invoke("send_request", { input })` で呼ぶ。
/// 送信の失敗（タイムアウトなど）は `Exchange.error` に入るので、ここでは `Err` にならない。
/// `Err` になるのは記録（DB への保存）に失敗したときだけ。
#[tauri::command]
pub async fn send_request(state: State<'_, AppState>, input: RequestInput) -> CmdResult<Exchange> {
    let mut exchange = state.client.send(&input).await;
    exchange.id = state.store.insert_exchange(&exchange).map_err(to_message)?;
    Ok(exchange)
}

/// 送信履歴（送信モードで送った通信の記録）を新しい順に返す
#[tauri::command]
pub async fn list_send_history(state: State<'_, AppState>) -> CmdResult<Vec<Exchange>> {
    state
        .store
        .list_exchanges(Some(ExchangeKind::Send), SEND_HISTORY_LIMIT)
        .map_err(to_message)
}
