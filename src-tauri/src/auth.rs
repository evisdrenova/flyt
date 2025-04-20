// defines a stream
use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, env};
use uuid::Uuid;

const DEFAULT_BASE_URL: &str = "https://chat.stream-io-api.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(6);

// Stream Chat API client
pub struct StreamChatClient {
    api_key: String,
    api_secret: String,
    base_url: String,
    client: Client,
    auth_token: String,
}

// Response is a common response type from Stream API
#[derive(Debug, Deserialize)]
pub struct Response {
    pub duration: String,
    pub message: Option<String>,
    pub more_info: Option<String>,
}

impl StreamChatClient {
    // Create a new client
    pub fn initialize(api_key: &str, api_secret: &str) -> Result<Self> {
        if api_key.is_empty() || api_secret.is_empty() {
            return Err(anyhow!("API key or secret is empty"));
        }

        // Set up HTTP client
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

    // Create a JWT token for authentication
    pub fn create_user_token(&self, user_id: &str) -> Result<String> {
        if user_id.is_empty() {
            return Err(anyhow!("User ID is empty"));
        }

        let mut claims = HashMap::new();
        claims.insert("user_id".to_string(), user_id.to_string());

        // Add current time as issued time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Time error: {}", e))?
            .as_secs();

        claims.insert("iat".to_string(), now.to_string());

        // Default expiration: 14 days
        let expiration = now + (14 * 24 * 60 * 60);
        claims.insert("exp".to_string(), expiration.to_string());

        // Create token
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

        // Add current time as issued at time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Time error: {}", e))?
            .as_secs();

        claims.insert("iat".to_string(), now.to_string());

        // Create and sign the token
        let key: Hmac<Sha256> =
            Hmac::new_from_slice(self.api_secret.as_bytes()).map_err(|_| anyhow!("Invalid key"))?;

        let token = claims
            .sign_with_key(&key)
            .map_err(|e| anyhow!("Signing error: {}", e))?;

        Ok(token)
    }

    fn generate_user_id(&self, username: &str) -> String {
        // Using a namespace UUID to generate deterministic UUIDs based on username
        // This ensures the same username always gets the same ID
        let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
        let user_id = uuid::Uuid::new_v5(&namespace, username.as_bytes());
        user_id.to_string()
    }

    // Verify webhook signature
    pub fn verify_webhook(body: &[u8], signature: &[u8], api_secret: &str) -> bool {
        let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes()).unwrap();
        mac.update(body);

        let expected_signature = hex::encode(mac.finalize().into_bytes());
        let signature_str = std::str::from_utf8(signature).unwrap_or("");

        expected_signature == signature_str
    }
}

// // Create default headers for API requests
// fn create_headers(&self) -> Result<HeaderMap> {
//     let mut headers = HeaderMap::new();

//     // Add common headers
//     headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
//     headers.insert("Stream-Auth-Type", HeaderValue::from_static("jwt"));
//     headers.insert(
//         "X-Stream-Client",
//         HeaderValue::from_static("stream-chat-rust-client-1.0.0"),
//     );

//     // Add API key header
//     let api_key_value = HeaderValue::from_str(&self.api_key)?;
//     headers.insert("X-Stream-API-Key", api_key_value);

//     // Add auth token
//     let auth_value = HeaderValue::from_str(&format!("Bearer {}", self.auth_token))?;
//     headers.insert(AUTHORIZATION, auth_value);

//     Ok(headers)
// }

// // Execute a request and parse the response
// async fn execute_request<T: for<'de> Deserialize<'de>>(
//     &self,
//     method: reqwest::Method,
//     path: &str,
//     body: Option<Value>,
//     query: Option<Vec<(String, String)>>,
// ) -> Result<T> {
//     let url = format!("{}{}", self.base_url, path);

//     let headers = self.create_headers()?;

//     let mut request_builder = self.client.request(method, &url).headers(headers);

//     // Add query parameters if provided
//     if let Some(params) = query {
//         for (key, value) in params {
//             request_builder = request_builder.query(&[(key, value)]);
//         }
//     }

//     // Add body if provided
//     if let Some(json_body) = body {
//         request_builder = request_builder.json(&json_body);
//     }

//     // Execute the request
//     let response = request_builder.send().await?;

//     // Handle error status codes
//     if !response.status().is_success() {
//         let status = response.status();
//         let error_text = response.text().await?;
//         return Err(anyhow!(
//             "API request failed with status {}: {}",
//             status,
//             error_text
//         ));
//     }

//     // Parse response body
//     let result = response.json::<T>().await?;

//     Ok(result)
// }

// // Get all channels for a user
// pub async fn get_user_channels(&self, user_id: &str) -> Result<Value> {
//     let query = vec![
//         ("user_id".to_string(), user_id.to_string()),
//         ("presence".to_string(), "true".to_string()),
//         ("state".to_string(), "true".to_string()),
//     ];

//     self.execute_request(reqwest::Method::GET, "/channels", None, Some(query))
//         .await
// }

// // Create a new channel
// pub async fn create_channel(
//     &self,
//     channel_id: &str,
//     channel_name: &str,
//     members: &[String],
//     created_by_id: &str,
// ) -> Result<Value> {
//     let path = format!("/channels/team/{}", channel_id);

//     let payload = json!({
//         "created_by_id": created_by_id,
//         "name": channel_name,
//         "members": members
//     });

//     self.execute_request(reqwest::Method::POST, &path, Some(payload), None)
//         .await
// }

// // Send a message to a channel
// pub async fn send_message(&self, channel_id: &str, user_id: &str, text: &str) -> Result<Value> {
//     let path = format!("/channels/team/{}/message", channel_id);

//     let payload = json!({
//         "message": {
//             "text": text,
//             "user_id": user_id
//         }
//     });

//     self.execute_request(reqwest::Method::POST, &path, Some(payload), None)
//         .await
// }
