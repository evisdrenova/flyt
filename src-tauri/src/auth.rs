// handles all auth functions
use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

// Generate a deterministic but unique user ID from username
pub fn generate_user_id(username: &str) -> String {
    // Using a namespace UUID to generate deterministic UUIDs based on username
    // This ensures the same username always gets the same ID
    let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
    let user_id = Uuid::new_v5(&namespace, username.as_bytes());
    user_id.to_string()
}

// Create a JWT token for Stream Chat
pub fn create_token(user_id: &str, api_secret: &str) -> Result<String> {
    // Create a HMAC-SHA256 key from the API secret
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(api_secret.as_bytes()).map_err(|_| anyhow!("Invalid key"))?;

    // Prepare claims for the JWT token
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(14))
        .expect("valid timestamp")
        .timestamp();

    let mut claims = HashMap::new();
    claims.insert("user_id", user_id);
    claims.insert("exp", &expiration.to_string());

    // Create and sign JWT
    let token_str = claims
        .sign_with_key(&key)
        .map_err(|e| anyhow!("Signing error: {}", e))?;

    Ok(token_str)
}

// Verify a JWT token
pub fn verify_token(token: &str, api_secret: &str) -> Result<HashMap<String, String>> {
    // Create a HMAC-SHA256 key from the API secret
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(api_secret.as_bytes()).map_err(|_| anyhow!("Invalid key"))?;

    // Verify the token and get claims
    let claims: HashMap<String, String> = token
        .verify_with_key(&key)
        .map_err(|e| anyhow!("Token verification error: {}", e))?;

    // Check if token is expired
    if let Some(exp_str) = claims.get("exp") {
        let exp = exp_str
            .parse::<i64>()
            .map_err(|_| anyhow!("Invalid expiration timestamp"))?;

        let now = chrono::Utc::now().timestamp();
        if now > exp {
            return Err(anyhow!("Token expired"));
        }
    }

    Ok(claims)
}
