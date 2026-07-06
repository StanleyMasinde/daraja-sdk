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
//! ## Quick start
//!
//! ```no_run
//! use daraja_sdk::mpesa;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), reqwest::Error> {
//!     let mpesa = mpesa::Mpesa::with_credentials(
//!         "your-consumer-key".into(),
//!         "your-consumer-secret".into(),
//!     );
//!
//!     let token = mpesa.generate_access_token().await?;
//!     println!("{}", token.access_token);
//!
//!     Ok(())
//! }
//! ```

pub mod mpesa;
