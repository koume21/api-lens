//! 観測対象ホストのコマンド（F21）

use tauri::State;

use super::{AppState, CmdResult, to_message};
use crate::models::{ObservedHost, normalize_host};

#[tauri::command]
pub async fn list_hosts(state: State<'_, AppState>) -> CmdResult<Vec<ObservedHost>> {
    state.store.list_hosts().map_err(to_message)
}

/// 表示名とホストを登録する。ホストは URL を貼り付けても `localhost:3000` の形に整えて保存する
#[tauri::command]
pub async fn add_host(
    state: State<'_, AppState>,
    label: String,
    host: String,
) -> CmdResult<ObservedHost> {
    let label = label.trim();
    if label.is_empty() {
        return Err("表示名を入力してください".into());
    }
    let host = normalize_host(&host)
        .ok_or("ホストの形式が正しくありません（例: localhost:3000、api.example.com）")?;
    let added = state.store.insert_host(label, &host).map_err(to_message)?;
    // 観測中でもすぐ効くよう、関所の一覧も入れ替える
    state.sync_proxy_hosts()?;
    Ok(added)
}

#[tauri::command]
pub async fn delete_host(state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    state.store.delete_host(id).map_err(to_message)?;
    state.sync_proxy_hosts()
}
