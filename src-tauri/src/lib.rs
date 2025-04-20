use std::{collections::HashMap, sync::Mutex};

use config::Config;
use stream::AppState;
use tauri::Manager;

mod auth;
mod config;
mod stream;
mod stream_chat;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    #[cfg(debug_assertions)]
    config.display_debug_info();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            users: Mutex::new(HashMap::new()),
            config,
        })
        .invoke_handler(tauri::generate_handler![
            stream::send_message,
            stream::create_channel,
            stream::login_and_initialize
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
