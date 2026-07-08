//! M-Pesa client for the [Safaricom Daraja](https://developer.safaricom.co.ke/) API.
//!
//! Start with [`Client`] — construct it via [`Client::with_credentials`] with borrowed
//! credentials, then call [`Client::generate_access_token`] to obtain an OAuth access token
//! for subsequent API requests.
//!
//! For Lipa Na M-Pesa Online (STK Push), use [`MpesaExpress`] after obtaining an access token.
//! Daraja delivers transaction results to your callback URL — handle those in your application.
//!
//! ## Environments
//!
//! Both [`Client`] and [`MpesaExpress`] default to the
//! [sandbox](crate::DarajaEnvironment::Sandbox). Call [`.production()`](crate::DarajaApi::production)
//! on each builder before sending a request to target the live Daraja API. Configure each
//! endpoint independently — if you use OAuth and STK Push together, call `.production()` on
//! both builders when using production credentials.

mod client;
mod express;

#[cfg(test)]
pub(crate) mod test_support;

#[doc(inline)]
pub use crate::types::{DarajaApi, DarajaEnvironment};
#[doc(inline)]
pub use client::{Client, GenerateAccessTokenResponse};
#[doc(inline)]
pub use express::{ExpressError, MpesaExpress, StkPushResponse, TransactionType};
