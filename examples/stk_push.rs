//! Initiate an M-Pesa Express STK Push (sandbox).
//!
//! Obtains an OAuth access token, then sends a Lipa na M-Pesa Online prompt.
//!
//! ```bash
//! export DARAJA_CONSUMER_KEY="your-consumer-key"
//! export DARAJA_CONSUMER_SECRET="your-consumer-secret"
//! export DARAJA_PASSKEY="your-passkey"
//! export DARAJA_CALLBACK_URL="https://your-domain.com/callback"
//! export DARAJA_PHONE_NUMBER="254700000000"
//! cargo run --example stk_push
//! ```

use daraja_sdk::mpesa::{self, MpesaExpress};

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

#[tokio::main]
async fn main() {
    let consumer_key = env("DARAJA_CONSUMER_KEY");
    let consumer_secret = env("DARAJA_CONSUMER_SECRET");
    let passkey = env("DARAJA_PASSKEY");
    let callback_url = env("DARAJA_CALLBACK_URL");
    let phone_number: u64 = env("DARAJA_PHONE_NUMBER")
        .parse()
        .expect("Failed to parse the phone number into a u64.");

    let token = mpesa::Client::with_credentials(&consumer_key, &consumer_secret)
        .generate_access_token()
        .await
        .expect("Failed to generate the Access token.");

    let response = MpesaExpress::new()
        .access_token(&token.access_token)
        .passkey(&passkey)
        .business_short_code(174379)
        .party_a(phone_number)
        .party_b(174379)
        .phone_number(phone_number)
        .amount(1)
        .account_reference("Order123")
        .tx_description("Payment")
        .call_back_url(&callback_url)
        .send_prompt()
        .await
        .expect("Failed to send the STK push request.");

    println!("{}", response.customer_message);
}
