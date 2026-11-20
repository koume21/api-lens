//! Tauri の準備とコマンドの登録。
//!
//! 部品の役割:
//! - `commands`: 画面からの依頼を受ける受付（画面とやり取りするのはここだけ）
//! - `client`:   送信役（reqwest）
//! - `models`:   データの形

mod client;
mod commands;
mod models;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = client::HttpClient::new().expect("HTTP クライアントを作れませんでした");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { client })
        .invoke_handler(tauri::generate_handler![commands::send::send_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
