//! Shared helpers for live sandbox integration tests.

use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;

use super::Client;

pub const TOKEN_CACHE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/last_token.txt");

/// Sandbox credentials loaded from `config.toml` at the crate root.
#[derive(Deserialize)]
pub struct TestConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
    pub callback_url: String,
    pub phone_number: u64,
}

impl TestConfig {
    pub fn load(is_production: bool) -> Self {
        let mut config_file = "config.toml";

        if is_production {
            config_file = "config-prod.toml";
        }

        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), config_file);
        let contents = std::fs::read_to_string(path)
            .expect("copy config.toml.example to config.toml and add sandbox credentials");
        toml::from_str(&contents).expect("config.toml is malformed")
    }
}

pub fn read_token_cache() -> Option<(String, SystemTime)> {
    let contents = fs::read_to_string(TOKEN_CACHE_PATH).ok()?;
    let mut parts = contents.splitn(2, ':');
    let access_token = parts.next()?.trim().to_string();
    if access_token.is_empty() {
        return None;
    }
    let expires_at_secs: u64 = parts.next()?.trim().parse().ok()?;
    let expires_at = UNIX_EPOCH.checked_add(Duration::from_secs(expires_at_secs))?;
    if expires_at <= SystemTime::now() {
        return None;
    }
    Some((access_token, expires_at))
}

pub fn write_token_cache(access_token: &str, expires_in: u64) {
    let expires_at = SystemTime::now()
        .checked_add(Duration::from_secs(expires_in))
        .expect("expires_in should fit in SystemTime");
    let expires_at_secs = expires_at
        .duration_since(UNIX_EPOCH)
        .expect("expiry should be after UNIX_EPOCH")
        .as_secs();
    fs::write(
        TOKEN_CACHE_PATH,
        format!("{access_token}:{expires_at_secs}"),
    )
    .expect("failed to write token cache");
}

pub fn assert_valid_access_token(access_token: &str) {
    assert!(!access_token.is_empty());
}

pub fn assert_valid_expires_in(expires_in: u64) {
    assert!(expires_in > 0);
    assert!(expires_in <= 3600);
}

/// Returns a cached sandbox access token when still valid, otherwise fetches a new one.
pub async fn get_access_token() -> String {
    if let Some((cached_token, expires_at)) = read_token_cache() {
        assert_valid_access_token(&cached_token);
        let remaining = expires_at
            .duration_since(SystemTime::now())
            .expect("cached token should not be expired");
        assert_valid_expires_in(remaining.as_secs());
        cached_token
    } else {
        let test_config = TestConfig::load(false);
        let client =
            Client::with_credentials(&test_config.consumer_key, &test_config.consumer_secret);
        let response = client
            .generate_access_token()
            .await
            .expect("failed to generate sandbox access token");
        let expires_in: u64 = response
            .expires_in
            .parse()
            .expect("expires_in should be a positive integer");
        write_token_cache(&response.access_token, expires_in);
        response.access_token
    }
}
