// file used to define stream commands

use crate::auth::{create_token, generate_user_id};
use crate::config::Config;
use crate::stream_chat::StreamChatClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

// Store active user sessions
pub struct AppState {
    pub users: Mutex<HashMap<String, String>>, // username -> user_id
    pub config: Config,
}

// Response types
#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub struct StreamTokenRequest {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub channel_id: String,
    pub message: String,
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    pub channel_id: String,
    pub channel_name: String,
    pub members: Vec<String>,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct InitChatResponse {
    pub api_key: String,
    pub channel_id: String,
}

#[derive(Deserialize)]
pub struct InitChatRequest {
    pub user_id: String,
    pub username: String,
}

// Get Stream Chat API key for the frontend
#[tauri::command]
pub fn get_stream_api_key(app_state: State<'_, AppState>) -> Result<String, String> {
    Ok(app_state.config.stream_api_key.clone())
}

// Authenticate user and generate Stream Chat token
#[tauri::command]
pub async fn authenticate_user(
    app_state: State<'_, AppState>,
    request: AuthRequest,
) -> Result<AuthResponse, String> {
    println!("trying to auth user: {}", request.username.trim());

    let username = request.username.trim();

    if username.is_empty() {
        return Err("Username cannot be empty".into());
    }

    let mut users = app_state.users.lock().unwrap();

    // Get or create user_id for this username
    let user_id = match users.get(username) {
        Some(id) => id.clone(),
        None => {
            let new_id = generate_user_id(username);
            users.insert(username.to_string(), new_id.clone());
            new_id
        }
    };

    // Generate Stream Chat token
    let token = create_token(&user_id, &app_state.config.stream_api_secret)
        .map_err(|e| format!("Failed to create token: {}", e))?;

    Ok(AuthResponse { user_id, token })
}

// Get Stream Chat token for a user
#[tauri::command]
pub async fn stream_token(
    app_state: State<'_, AppState>,
    request: StreamTokenRequest,
) -> Result<String, String> {
    create_token(&request.user_id, &app_state.config.stream_api_secret)
        .map_err(|e| format!("Failed to create token: {}", e))
}

// Send a message to a channel through the backend
#[tauri::command]
pub async fn send_message(
    app_state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<(), String> {
    let client = StreamChatClient::new(
        &app_state.config.stream_api_key,
        &app_state.config.stream_api_secret,
    );

    client
        .send_message(&request.channel_id, &request.user_id, &request.message)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))
}

// Create a new channel
#[tauri::command]
pub async fn create_channel(
    app_state: State<'_, AppState>,
    request: CreateChannelRequest,
) -> Result<(), String> {
    let client = StreamChatClient::new(
        &app_state.config.stream_api_key,
        &app_state.config.stream_api_secret,
    );

    client
        .create_channel(
            &request.channel_id,
            &request.channel_name,
            &request.members,
            &request.user_id,
        )
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))
}

#[tauri::command]
pub async fn initialize_chat(
    app_state: State<'_, AppState>,
    request: InitChatRequest,
) -> Result<InitChatResponse, String> {
    // Default channel ID - this could be made more sophisticated
    // by finding the user's most recent channel or preferred channel
    let default_channel_id = "general".to_string();

    // Return only what the client needs to establish a connection
    Ok(InitChatResponse {
        api_key: app_state.config.stream_api_key.clone(),
        channel_id: default_channel_id,
    })
}
