use config::Config;

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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            stream::authenticate_user,
            stream::stream_token,
            stream::send_message,
            stream::create_channel,
            stream::get_stream_api_key,
            stream::initialize_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
