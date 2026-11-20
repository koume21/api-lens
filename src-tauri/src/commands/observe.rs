//! 観測モードのコマンド（F11・F12・F16・F19・F20）と、関所 → 画面へのイベント

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use super::{AppState, CmdResult, to_message};
use crate::models::{Exchange, ExchangeKind, ExchangeSummary};
use crate::proxy::ExchangeSink;
use crate::store::MAX_EXCHANGES;

/// 関所が通信を 1 件記録したときに画面へ送るイベントの名前。
/// 画面側（src/api/observe.ts）でも同じ名前で受け取る
pub const EXCHANGE_OBSERVED_EVENT: &str = "exchange-observed";

/// 観測の状態（ツールバーの「観測中（127.0.0.1:8080）」の表示に使う）
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObserveStatus {
    pub running: bool,
    /// 観測中なら待ち受けアドレス（例: 127.0.0.1:8080）
    pub addr: Option<String>,
}

fn status_of(state: &AppState) -> ObserveStatus {
    let addr = state.proxy.status();
    ObserveStatus {
        running: addr.is_some(),
        addr: addr.map(|a| a.to_string()),
    }
}

/// 関所が記録した通信の受け取り手を作る。
///
/// 1. 保存係に記録して番号をもらう（F19: 次回起動時も見られる）
/// 2. 画面にイベントで「新しい通信」を知らせる（F12、N4: 1 秒以内に一覧に出る）
///
/// 関所は Tauri を知らないので、Tauri の機能（`emit` や `State`）を使う部分はここで包んで渡す。
fn make_sink(app: AppHandle) -> ExchangeSink {
    Arc::new(move |mut ex: Exchange| {
        let state = app.state::<AppState>();
        match state.store.insert_exchange(&ex) {
            Ok(id) => ex.id = id,
            // 通信の中身（URL やヘッダー）はログに出さない（秘密情報が含まれ得るため）
            Err(e) => eprintln!("[observe] 記録に失敗しました: {e}"),
        }
        // 一覧に必要な要約だけを送る（本文まで送ると重い）
        if let Err(e) = app.emit(EXCHANGE_OBSERVED_EVENT, ExchangeSummary::from(&ex)) {
            eprintln!("[observe] 画面への通知に失敗しました: {e}");
        }
    })
}

/// 観測を開始する（F11）。`127.0.0.1:<port>` で関所が待ち受けを始める
#[tauri::command]
pub async fn start_observe(
    app: AppHandle,
    state: State<'_, AppState>,
    port: u16,
) -> CmdResult<ObserveStatus> {
    state
        .proxy
        .start(port, make_sink(app.clone()))
        .await
        .map_err(to_message)?;
    Ok(status_of(&state))
}

/// 観測を停止する（F11）
#[tauri::command]
pub async fn stop_observe(state: State<'_, AppState>) -> CmdResult<ObserveStatus> {
    state.proxy.stop();
    Ok(status_of(&state))
}

#[tauri::command]
pub async fn observe_status(state: State<'_, AppState>) -> CmdResult<ObserveStatus> {
    Ok(status_of(&state))
}

/// 観測した通信の一覧（要約・新しい順・最大 1,000 件）。起動時に前回までの分を読むのに使う（F19）
#[tauri::command]
pub async fn list_observed(state: State<'_, AppState>) -> CmdResult<Vec<ExchangeSummary>> {
    state
        .store
        .list_exchange_summaries(Some(ExchangeKind::Observe), MAX_EXCHANGES)
        .map_err(to_message)
}

/// 通信 1 件の詳細（送った内容と返ってきた内容）（F16）
#[tauri::command]
pub async fn get_exchange(state: State<'_, AppState>, id: i64) -> CmdResult<Exchange> {
    state.store.get_exchange(id).map_err(to_message)
}

/// 観測した通信の履歴をまとめて削除する（F20）。送信モードの履歴は残す
#[tauri::command]
pub async fn clear_observed(state: State<'_, AppState>) -> CmdResult<()> {
    state
        .store
        .delete_exchanges(ExchangeKind::Observe)
        .map(|_| ())
        .map_err(to_message)
}
