// defines a stream
use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, env};
use tauri::http::{HeaderMap, HeaderValue};
use uuid::Uuid;

const DEFAULT_BASE_URL: &str = "https://chat.stream-io-api.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(6);

// Stream Chat API client
pub struct StreamChatClient {
    api_key: String,
    api_secret: String,
    base_url: String,
    client: Client,
    pub auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelData {
    created_by_id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelMember {
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateChannelRequest {
    data: ChannelData,
    members: ChannelMember,
}

impl StreamChatClient {
    // Create a new client
    pub fn initialize(api_key: &str, api_secret: &str) -> Result<Self> {
        if api_key.is_empty() || api_secret.is_empty() {
            return Err(anyhow!("API key or secret is empty"));
        }

        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .pool_idle_timeout(Duration::from_secs(59))
            .pool_max_idle_per_host(5)
            .build()?;

        Ok(Self {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            client,
            auth_token: String::new(), // Empty initially
        })
    }

    // Get user ID from username, creating a new one if needed
    pub fn get_or_create_user_id(
        &self,
        users: &mut HashMap<String, String>,
        username: &str,
    ) -> String {
        match users.get(username) {
            Some(id) => id.clone(),
            None => {
                let new_id = self.generate_user_id(username);
                users.insert(username.to_string(), new_id.clone());
                new_id
            }
        }
    }

    fn generate_user_id(&self, username: &str) -> String {
        // Using a namespace UUID to generate deterministic UUIDs based on username
        // This ensures the same username always gets the same ID
        let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
        let user_id = uuid::Uuid::new_v5(&namespace, username.as_bytes());
        user_id.to_string()
    }

    // Create a user token taht we can use on the front end to chat
    pub fn create_user_token(&self, user_id: &str) -> Result<String> {
        if user_id.is_empty() {
            return Err(anyhow!("User ID is empty"));
        }

        let mut claims = HashMap::new();
        claims.insert("user_id".to_string(), user_id.to_string());

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Time error: {}", e))?
            .as_secs();

        claims.insert("iat".to_string(), now.to_string());

        // 14 days
        let expiration = now + (14 * 24 * 60 * 60);
        claims.insert("exp".to_string(), expiration.to_string());

        let key: Hmac<Sha256> =
            Hmac::new_from_slice(self.api_secret.as_bytes()).map_err(|_| anyhow!("Invalid key"))?;

        let token = claims
            .sign_with_key(&key)
            .map_err(|e| anyhow!("Signing error: {}", e))?;

        Ok(token)
    }

    pub fn create_server_token(&self) -> Result<String> {
        let mut claims = HashMap::new();
        claims.insert("server".to_string(), "true".to_string());

        let key: Hmac<Sha256> =
            Hmac::new_from_slice(self.api_secret.as_bytes()).map_err(|_| anyhow!("Invalid key"))?;

        let token = claims
            .sign_with_key(&key)
            .map_err(|e| anyhow!("Signing error: {}", e))?;

        Ok(token)
    }

    // Create a new channel
    pub async fn create_channel(
        &self,
        channel_name: &str,
        member: &str,
        created_by_id: &str,
    ) -> Result<Value> {
        let path = format!("/channels/messaging/{}/query", channel_name);

        // Create the payload using the struct
        let payload = CreateChannelRequest {
            data: ChannelData {
                created_by_id: created_by_id.to_string(),
                name: channel_name.to_string(),
            },
            members: ChannelMember {
                user_id: member.to_string(),
            },
        };

        // Convert the struct to a JSON value
        let payload_json = serde_json::to_value(payload)?;

        self.execute_request(reqwest::Method::POST, &path, Some(payload_json), None)
            .await
    }

    // Get all channels for a user
    pub async fn get_user_channels(&self, user_id: &str) -> Result<Value> {
        let query = vec![
            ("user_id".to_string(), user_id.to_string()),
            ("presence".to_string(), "true".to_string()),
            ("state".to_string(), "true".to_string()),
        ];

        self.execute_request(reqwest::Method::POST, "/channels", None, Some(query))
            .await
    }

    // Create default headers for API requests
    fn create_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add common headers
        headers.insert("Content-type", HeaderValue::from_static("application/json"));
        headers.insert("Stream-Auth-Type", HeaderValue::from_static("jwt"));

        // Add API key header
        let api_key_value = HeaderValue::from_str(&self.api_key)?;
        headers.insert("api_key", api_key_value);

        // Add auth token
        let auth_value = HeaderValue::from_str(&self.auth_token)?;
        headers.insert("Authorization", auth_value);

        Ok(headers)
    }

    // Execute a request and parse the response
    async fn execute_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
        query: Option<Vec<(String, String)>>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);

        let headers = self.create_headers()?;

        let mut request_builder = self.client.request(method, &url).headers(headers);

        // Add query parameters if provided
        if let Some(params) = query {
            for (key, value) in params {
                request_builder = request_builder.query(&[(key, value)]);
            }
        }

        // Add body if provided
        if let Some(json_body) = body {
            request_builder = request_builder.json(&json_body);
        }

        // Execute the request
        let response = request_builder.send().await?;

        // Handle error status codes
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!(
                "API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        // Parse response body
        let result = response.json::<T>().await?;

        Ok(result)
    }
}
