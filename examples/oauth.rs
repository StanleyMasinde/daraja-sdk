//! Generate a Daraja OAuth access token (sandbox).
//!
//! ```bash
//! export DARAJA_CONSUMER_KEY="your-consumer-key"
//! export DARAJA_CONSUMER_SECRET="your-consumer-secret"
//! cargo run --example oauth
//! ```

use daraja_sdk::mpesa;

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let consumer_key = env("DARAJA_CONSUMER_KEY");
    let consumer_secret = env("DARAJA_CONSUMER_SECRET");

    let client = mpesa::Client::with_credentials(&consumer_key, &consumer_secret);

    let token = client.generate_access_token().await?;
    println!("access_token: {}", token.access_token);
    println!("expires_in: {}s", token.expires_in);

    Ok(())
}
