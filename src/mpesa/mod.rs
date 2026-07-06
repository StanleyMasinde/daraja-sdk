//! M-Pesa client for the [Safaricom Daraja](https://developer.safaricom.co.ke/) API.
//!
//! Start with [`Client`] — construct it via [`Client::with_credentials`], then call
//! [`Client::generate_access_token`] to obtain an OAuth access token for subsequent API requests.
//!
//! For Lipa Na M-Pesa Online (STK Push), use [`MpesaExpress`] after obtaining an access token.
//! Daraja delivers transaction results to your callback URL — handle those in your application.

mod client;
mod express;

#[cfg(test)]
pub(crate) mod test_support;

#[doc(inline)]
pub use client::{Client, GenerateAccessTokenResponse};
#[doc(inline)]
pub use express::{ExpressError, MpesaExpress, StkPushResponse, TransactionType};
