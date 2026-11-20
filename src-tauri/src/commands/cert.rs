//! HTTPS 観測用の認証局証明書のコマンド（F13）
//!
//! アプリは証明書をキーチェーンに勝手に登録しない。場所を教え、Finder で開くところまで。

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use super::{AppState, CmdResult, to_message};
use crate::proxy::cert::{CA_COMMON_NAME, is_trusted};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaInfo {
    /// 証明書（公開してよい方）のファイルの場所。キーチェーンに登録するのはこれ
    pub cert_path: String,
    /// キーチェーンアクセスで探すときの名前
    pub common_name: String,
    /// 信頼済みとして登録されているか（macOS の `security verify-cert` で調べた目安）。
    /// 調べられないときは null
    pub trusted: Option<bool>,
}

#[tauri::command]
pub async fn ca_info(state: State<'_, AppState>) -> CmdResult<CaInfo> {
    let path = &state.proxy.ca_files().cert_path;
    Ok(CaInfo {
        cert_path: path.display().to_string(),
        common_name: CA_COMMON_NAME.to_string(),
        // 外部コマンドを待つので、async の実行環境をふさがないよう別スレッドで動かす
        trusted: tauri::async_runtime::spawn_blocking({
            let path = path.clone();
            move || is_trusted(&path)
        })
        .await
        .map_err(to_message)?,
    })
}

/// 証明書のファイルを Finder で表示する（ダブルクリックでキーチェーンアクセスに追加できる）
#[tauri::command]
pub async fn reveal_ca_cert(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    app.opener()
        .reveal_item_in_dir(&state.proxy.ca_files().cert_path)
        .map_err(to_message)
}
