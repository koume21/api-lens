//! 保存したリクエストのコマンド（F9・F10）

use tauri::State;

use super::{AppState, CmdResult, to_message};
use crate::models::{RequestInput, SavedRequest};

/// 名前の前後の空白を落とし、空なら断る
fn clean_name(name: &str) -> CmdResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("名前を入力してください".into());
    }
    Ok(name.to_string())
}

/// 名前を付けて新しく保存する
#[tauri::command]
pub async fn save_request(
    state: State<'_, AppState>,
    name: String,
    request: RequestInput,
) -> CmdResult<SavedRequest> {
    let name = clean_name(&name)?;
    state
        .store
        .insert_saved_request(&name, &request)
        .map_err(to_message)
}

/// 保存済みのものを上書きする
#[tauri::command]
pub async fn update_saved_request(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    request: RequestInput,
) -> CmdResult<SavedRequest> {
    let name = clean_name(&name)?;
    state
        .store
        .update_saved_request(id, &name, &request)
        .map_err(to_message)
}

#[tauri::command]
pub async fn list_saved_requests(state: State<'_, AppState>) -> CmdResult<Vec<SavedRequest>> {
    state.store.list_saved_requests().map_err(to_message)
}

#[tauri::command]
pub async fn delete_saved_request(state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    state.store.delete_saved_request(id).map_err(to_message)
}
