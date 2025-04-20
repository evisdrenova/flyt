// backend stream module that defines most functions rlated to users and channels
use crate::auth::StreamChatClient;
use crate::config::Config;
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
#[derive(Debug, Serialize)]
pub struct ChannelData {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub client_config: ClientConfig,
}

// Parse channel data from Stream API response
fn parse_channel_data(value: &serde_json::Value) -> Vec<ChannelData> {
    let mut channels = Vec::new();

    // Parse channels from response
    if let Some(channels_array) = value.get("channels").and_then(|v| v.as_array()) {
        for channel in channels_array {
            if let (Some(id), Some(channel_type), Some(name)) = (
                channel.get("cid").and_then(|v| v.as_str()),
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
                    name: name.to_string(),
                    members,
                });
            }
        }
    }

    channels
}

// =========== Command Handlers ===========

// Combined login and initialize function
#[tauri::command]
pub async fn login_and_initialize(
    state: State<'_, AppState>,
    request: AuthRequest,
) -> Result<LoginResponse, String> {
    let username = request.username.trim();
    if username.is_empty() {
        return Err("Username cannot be empty".into());
    }

    // Initialize Stream client
    let mut client = StreamChatClient::initialize(
        &state.config.stream_api_key,
        &state.config.stream_api_secret,
    )
    .map_err(|e| format!("Failed to initialize Stream client: {}", e))?;

    // Get user ID
    let user_id = {
        let mut users = state.users.lock().unwrap();
        client.get_or_create_user_id(&mut users, username)
    };

    // Create user token
    let user_token = client
        .create_user_token(&user_id)
        .map_err(|e| format!("Failed to create token: {}", e))?;

    // Create server token for API calls
    let server_token = client
        .create_server_token()
        .map_err(|e| format!("Failed to create server token: {}", e))?;

    // Set the server token for API calls
    client.auth_token = server_token;

    // Get channels for user
    let channels_result = client
        .get_user_channels(&user_id)
        .await
        .map_err(|e| format!("Failed to get user channels: {}", e))?;

    println!("get user channels: {:?}", channels_result);

    // Parse channels from result
    let mut channels = parse_channel_data(&channels_result);

    // If no channels exist, create a default one
    if channels.is_empty() {
        match client
            .create_channel("general", &user_id.clone(), &user_id)
            .await
        {
            Ok(_) => {
                channels.push(ChannelData {
                    name: "General".to_string(),
                    members: vec![user_id.clone()],
                });
            }
            Err(e) => {
                println!("Error creating default channel: {}", e);
            }
        }
    }

    // Create client config to return to frontend
    let client_config = ClientConfig {
        api_key: state.config.stream_api_key.clone(),
        user_token: user_token,
        channels,
    };

    let lg = LoginResponse {
        user_id,
        client_config,
    };

    println!("the lg: {:?}", lg);

    Ok(lg)
}

// //Create a new channel
// #[tauri::command]
// pub async fn create_channel(
//     state: State<'_, AppState>,
//     request: CreateChannelRequest,
// ) -> Result<ChannelData, String> {
//     // Initialize Stream Chat client
//     let client = StreamChatClient::new(
//         &state.config.stream_api_key,
//         &state.config.stream_api_secret,
//     )
//     .map_err(|e| format!("Failed to initialize Stream client: {}", e))?;

//     // Create the channel
//     client
//         .create_channel(
//             &request.channel_id,
//             &request.channel_name,
//             &request.members,
//             &request.user_id,
//         )
//         .await
//         .map_err(|e| format!("Failed to create channel: {}", e))?;

//     // Return the new channel data
//     Ok(ChannelData {
//         id: request.channel_id,
//         type_: "team".to_string(),
//         name: request.channel_name,
//         members: request.members,
//     })
// }
