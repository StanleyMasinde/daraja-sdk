//! # Daraja SDK
//!
//! A memory-safe Rust SDK for [Safaricom Daraja](https://developer.safaricom.co.ke/) (M-Pesa API 3.0),
//! focused on correctness and type safety.
//!
//! **Experimental:** This crate is an early experiment. The API is unstable, coverage is limited,
//! and it is not ready for production use.
//!
//! Requests currently target the Daraja sandbox environment.
//!
//! ## OAuth
//!
//! ```no_run
//! use daraja_sdk::mpesa;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), reqwest::Error> {
//!     let client = mpesa::Client::with_credentials("your-consumer-key", "your-consumer-secret");
//!     let token = client.generate_access_token().await?;
//!     println!("{}", token.access_token);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## M-Pesa Express (STK Push)
//!
//! Obtain an access token first (see OAuth above), then:
//!
//! ```no_run
//! use daraja_sdk::mpesa::{ExpressError, MpesaExpress};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ExpressError> {
//!     let response = MpesaExpress::new()
//!         .access_token("your-access-token")
//!         .passkey("your-lipa-na-mpesa-passkey")
//!         .business_short_code(174379)
//!         .party_a(254700000000)
//!         .party_b(174379)
//!         .phone_number(254700000000)
//!         .amount(1)
//!         .account_reference("Order123")
//!         .tx_description("Payment")
//!         .call_back_url("https://your-domain.com/callback")
//!         .send_prompt()
//!         .await?;
//!
//!     println!("{}", response.customer_message);
//!     Ok(())
//! }
//! ```
//!
//! Transaction results are posted to your callback URL. Handle those in your application;
//! this crate only initiates the STK Push prompt.

pub mod mpesa;
pub mod types;
mod url_helper;
