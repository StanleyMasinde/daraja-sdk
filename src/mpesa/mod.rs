//! M-Pesa client for the [Safaricom Daraja](https://developer.safaricom.co.ke/) API.
//!
//! Start with [`Client`] — construct it via [`Client::with_credentials`], then call
//! [`Client::generate_access_token`] to obtain an OAuth access token for subsequent API requests.

mod client;

#[doc(inline)]
pub use client::{Client, GenerateAccessTokenResponse};
