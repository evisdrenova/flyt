// src/stream.rs
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

// =========== Type Definitions ===========
#[derive(Serialize)]
pub struct ChannelData {
    pub id: String,
    pub type_: String, // 'type' is a keyword in Rust
    #[serde(rename = "type")] // Rename to "type" in JSON output
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Serialize)]
pub struct ClientConfig {
    pub api_key: String,
    pub user_token: String,
    pub channels: Vec<ChannelData>,
}

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
pub struct LoginResponse {
    pub user_id: String,
    pub client_config: ClientConfig,
}

// =========== Helper Functions ===========

// Get user ID from username, creating a new one if needed
fn get_or_create_user_id(users: &mut HashMap<String, String>, username: &str) -> String {
    match users.get(username) {
        Some(id) => id.clone(),
        None => {
            let new_id = generate_user_id(username);
            users.insert(username.to_string(), new_id.clone());
            new_id
        }
    }
}

// Get channels for a user, creating a default if none found
async fn get_user_channels(client: &StreamChatClient, user_id: &str) -> Vec<ChannelData> {
    match client.get_user_channels(user_id).await {
        Ok(channels_response) => {
            let mut channels = Vec::new();

            // Parse channels from response
            if let Some(channels_array) = channels_response.get("channels") {
                if let Some(channels_array) = channels_array.as_array() {
                    for channel in channels_array {
                        if let (Some(id), Some(channel_type), Some(name)) = (
                            channel.get("id").and_then(|v| v.as_str()),
                            channel.get("type").and_then(|v| v.as_str()),
                            channel.get("name").and_then(|v| v.as_str()),
                        ) {
                            // Extract members
                            let members = if let Some(members_obj) = channel.get("members") {
                                if let Some(members_arr) = members_obj.as_array() {
                                    members_arr
                                        .iter()
                                        .filter_map(|m| {
                                            m.get("user_id")
                                                .and_then(|id| id.as_str())
                                                .map(String::from)
                                        })
                                        .collect()
                                } else {
                                    Vec::new()
                                }
                            } else {
                                Vec::new()
                            };

                            channels.push(ChannelData {
                                id: id.to_string(),
                                type_: channel_type.to_string(),
                                name: name.to_string(),
                                members,
                            });
                        }
                    }
                }
            }

            // If no channels found, create a default channel
            if channels.is_empty() {
                // Create a general channel if none exists
                let general_id = "general";
                let channel_name = "General";

                match client
                    .create_channel(general_id, channel_name, &[user_id.to_string()], user_id)
                    .await
                {
                    Ok(_) => {
                        channels.push(ChannelData {
                            id: general_id.to_string(),
                            type_: "team".to_string(),
                            name: channel_name.to_string(),
                            members: vec![user_id.to_string()],
                        });
                    }
                    Err(e) => {
                        println!("Error creating default channel: {}", e);
                        // Still add it to the list so the UI can try to connect
                        channels.push(ChannelData {
                            id: general_id.to_string(),
                            type_: "team".to_string(),
                            name: channel_name.to_string(),
                            members: vec![user_id.to_string()],
                        });
                    }
                }
            }

            channels
        }
        Err(e) => {
            println!("Error fetching channels: {}", e);
            // Provide a default channel
            vec![ChannelData {
                id: "general".to_string(),
                type_: "team".to_string(),
                name: "General".to_string(),
                members: vec![user_id.to_string()],
            }]
        }
    }
}

// =========== Command Handlers ===========

// Combined login and initialize function
#[tauri::command]
pub async fn login_and_initialize(
    state: State<'_, AppState>,
    request: AuthRequest,
) -> Result<LoginResponse, String> {
    println!(
        "Login and initializing for user: {}",
        request.username.trim()
    );

    let username = request.username.trim();
    if username.is_empty() {
        return Err("Username cannot be empty".into());
    }

    // IMPORTANT: Release the mutex guard before any async operations
    let user_id = {
        let mut users = state.users.lock().unwrap();
        get_or_create_user_id(&mut users, username)
    };

    // Generate token
    let token = create_token(&user_id, &state.config.stream_api_secret)
        .map_err(|e| format!("Failed to create token: {}", e))?;

    // Get channels for user
    let client = StreamChatClient::new(
        &state.config.stream_api_key,
        &state.config.stream_api_secret,
    );

    let channels = get_user_channels(&client, &user_id).await;

    // Create client config
    let client_config = ClientConfig {
        api_key: state.config.stream_api_key.clone(),
        user_token: token,
        channels,
    };

    Ok(LoginResponse {
        user_id,
        client_config,
    })
}

// Create a new channel
#[tauri::command]
pub async fn create_channel(
    state: State<'_, AppState>,
    request: CreateChannelRequest,
) -> Result<ChannelData, String> {
    let client = StreamChatClient::new(
        &state.config.stream_api_key,
        &state.config.stream_api_secret,
    );

    client
        .create_channel(
            &request.channel_id,
            &request.channel_name,
            &request.members,
            &request.user_id,
        )
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))?;

    // Return the new channel data
    Ok(ChannelData {
        id: request.channel_id,
        type_: "team".to_string(),
        name: request.channel_name,
        members: request.members,
    })
}

// Send a message to a channel
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<(), String> {
    let client = StreamChatClient::new(
        &state.config.stream_api_key,
        &state.config.stream_api_secret,
    );

    client
        .send_message(&request.channel_id, &request.user_id, &request.message)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))
}
