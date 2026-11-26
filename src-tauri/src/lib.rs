//! Tauri の準備とコマンドの登録。
//!
//! 部品の役割:
//! - `commands`: 画面からの依頼を受ける受付（画面とやり取りするのはここだけ）
//! - `client`:   送信役（reqwest）
//! - `proxy`:    関所（hudsucker）
//! - `store`:    保存係（SQLite）
//! - `models`:   データの形

mod client;
mod commands;
mod models;
mod proxy;
mod store;

use tauri::Manager;

use commands::AppState;

/// DB ファイルの名前。置き場所は OS ごとのアプリ用データフォルダ
/// （macOS なら ~/Library/Application Support/com.koumei.api-lens/）
const DB_FILE: &str = "api-lens.db";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // setup はウィンドウを出す前に 1 回だけ呼ばれる。
        // データフォルダの場所は App（`app.path()`）からしか分からないので、ここで部品を作る
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let state = AppState {
                client: client::HttpClient::new()?,
                store: store::Store::open(&data_dir.join(DB_FILE))?,
                proxy: proxy::ProxyController::new(),
            };
            // 前回までに登録した観測対象ホストを関所に知らせておく
            state.sync_proxy_hosts()?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send::send_request,
            commands::send::list_send_history,
            commands::saved::save_request,
            commands::saved::update_saved_request,
            commands::saved::list_saved_requests,
            commands::saved::delete_saved_request,
            commands::hosts::list_hosts,
            commands::hosts::add_host,
            commands::hosts::delete_host,
            commands::observe::start_observe,
            commands::observe::stop_observe,
            commands::observe::observe_status,
            commands::observe::list_observed,
            commands::observe::get_exchange,
            commands::observe::clear_observed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
