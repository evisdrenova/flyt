// src-tauri/src/stream_chat.rs
use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde_json::{json, Value};

// Stream Chat API client
pub struct StreamChatClient {
    api_key: String,
    api_secret: String,
    client: Client,
    base_url: String,
}

impl StreamChatClient {
    pub fn new(api_key: &str, api_secret: &str) -> Self {
        let client = Client::new();
        Self {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            client,
            base_url: "https://chat.stream-io-api.com".to_string(),
        }
    }

    // Create auth headers for Stream Chat API
    fn create_headers(&self) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Stream-Auth-Type", "jwt".parse().unwrap());

        let token = crate::auth::create_token("server", &self.api_secret)?;
        headers.insert("Authorization", format!("{}", token).parse().unwrap());

        Ok(headers)
    }

    // Send a message to a channel
    pub async fn send_message(&self, channel_id: &str, user_id: &str, text: &str) -> Result<()> {
        let headers = self.create_headers()?;

        let payload = json!({
            "message": {
                "text": text,
                "user_id": user_id
            }
        });

        let response = self
            .client
            .post(format!(
                "{}/channels/team/{}/message",
                self.base_url, channel_id
            ))
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("API error: {}", error_body));
        }

        Ok(())
    }

    // Create a new channel
    pub async fn create_channel(
        &self,
        channel_id: &str,
        channel_name: &str,
        members: &[String],
        created_by_id: &str,
    ) -> Result<()> {
        let headers = self.create_headers()?;

        let payload = json!({
            "created_by_id": created_by_id,
            "name": channel_name,
            "members": members
        });

        let response = self
            .client
            .post(format!("{}/channels/team/{}", self.base_url, channel_id))
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("API error: {}", error_body));
        }

        Ok(())
    }

    // Get all channels for a user
    pub async fn get_user_channels(&self, user_id: &str) -> Result<Value> {
        let headers = self.create_headers()?;

        let response = self
            .client
            .get(format!(
                "{}/channels?user_id={}&type=team",
                self.base_url, user_id
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("API error: {}", error_body));
        }

        let result = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse API response: {}", e))?;

        Ok(result)
    }

    // Get channel messages
    pub async fn get_messages(&self, channel_id: &str) -> Result<Value> {
        let headers = self.create_headers()?;

        let response = self
            .client
            .get(format!(
                "{}/channels/team/{}/messages",
                self.base_url, channel_id
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("API error: {}", error_body));
        }

        let result = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse API response: {}", e))?;

        Ok(result)
    }
}
