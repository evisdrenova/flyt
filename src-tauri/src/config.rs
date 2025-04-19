// src-tauri/src/config.rs
use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub stream_api_key: String,
    pub stream_api_secret: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load from .env file if it exists
        dotenv().ok();

        // Get environment variables
        let stream_api_key = env::var("STREAM_API_KEY")
            .map_err(|_| anyhow!("STREAM_API_KEY environment variable not set"))?;

        let stream_api_secret = env::var("STREAM_API_SECRET")
            .map_err(|_| anyhow!("STREAM_API_SECRET environment variable not set"))?;

        // Make sure they're not empty
        if stream_api_key.is_empty() {
            return Err(anyhow!("STREAM_API_KEY cannot be empty"));
        }

        if stream_api_secret.is_empty() {
            return Err(anyhow!("STREAM_API_SECRET cannot be empty"));
        }

        Ok(Config {
            stream_api_key,
            stream_api_secret,
        })
    }

    #[cfg(debug_assertions)]
    pub fn display_debug_info(&self) {
        eprintln!("Debug: Configuration loaded");
        eprintln!(
            "Debug: Stream API Key length: {}",
            self.stream_api_key.len()
        );
        eprintln!(
            "Debug: Stream API Secret length: {}",
            self.stream_api_secret.len()
        );
    }
}
