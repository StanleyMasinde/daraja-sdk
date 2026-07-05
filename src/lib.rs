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
//! use daraja_sdk::Mpesa;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), reqwest::Error> {
//!     let mpesa = Mpesa::with_credentials(
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

use reqwest::Error;
use serde::Deserialize;

const OAUTH_URL: &str =
    "https://sandbox.safaricom.co.ke/oauth/v1/generate?grant_type=client_credentials";

/// The response from Daraja when you request an *access_token*.
///
/// Returned by [`Mpesa::generate_access_token`].
#[derive(Deserialize, Debug)]
pub struct GenerateAccessTokenResponse {
    /// Access token to access the APIs.
    ///
    /// Example: `"c9SQxWWhmdVRlyh0zh8gZDTkubVF"`
    pub access_token: String,

    /// Token expiry time in seconds, as returned by the API.
    ///
    /// Example: `"3599"`
    pub expires_in: String,
}

/// An M-Pesa SDK client.
///
/// `Mpesa` holds the configuration state needed to build requests against the Daraja M-Pesa API.
/// Construct it with [`Mpesa::new`] for defaults, or the recommended [`Mpesa::with_credentials`]
/// to supply a consumer key and secret up front.
pub struct Mpesa {
    /// The consumer key of your app in Daraja.
    pub consumer_key: String,

    /// The consumer secret of your app in Daraja.
    pub consumer_secret: String,
}

impl Default for Mpesa {
    fn default() -> Self {
        Self::new()
    }
}

impl Mpesa {
    /// Creates a new `Mpesa` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use daraja_sdk::Mpesa;
    ///
    /// let mpesa_sdk = Mpesa::new();
    /// ```
    pub fn new() -> Self {
        Self {
            consumer_key: String::new(),
            consumer_secret: String::new(),
        }
    }

    /// Creates an `Mpesa` preconfigured with consumer_key and consumer_secret.
    ///
    /// # Examples
    ///
    /// ```
    /// use daraja_sdk::Mpesa;
    ///
    /// let mpesa_sdk = Mpesa::with_credentials(
    ///     "consumer_key".to_string(),
    ///     "consumer_secret".to_string(),
    /// );
    /// ```
    pub fn with_credentials(consumer_key: String, consumer_secret: String) -> Self {
        Self {
            consumer_key,
            consumer_secret,
        }
    }

    /// Generates the access token needed to authenticate requests to the Daraja API.
    ///
    /// This method consumes `self`. The token expires in 1 hour; persist the result and call this
    /// again after it expires.
    ///
    /// # Errors
    ///
    /// Returns a [`reqwest::Error`] if the request fails or the response body cannot be
    /// deserialized into a [`GenerateAccessTokenResponse`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use daraja_sdk::Mpesa;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), reqwest::Error> {
    ///     let mpesa = Mpesa::with_credentials(
    ///         "consumer_key".to_string(),
    ///         "consumer_secret".to_string(),
    ///     );
    ///
    ///     let token = mpesa.generate_access_token().await?;
    ///     println!("{}", token.access_token);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn generate_access_token(self) -> Result<GenerateAccessTokenResponse, Error> {
        let http_client = reqwest::Client::new();
        http_client
            .get(OAUTH_URL)
            .basic_auth(self.consumer_key, Some(self.consumer_secret))
            .send()
            .await?
            .error_for_status()?
            .json::<GenerateAccessTokenResponse>()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct TestConfig {
        consumer_key: String,
        consumer_secret: String,
    }

    impl TestConfig {
        fn load() -> Self {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");
            let contents = std::fs::read_to_string(path)
                .expect("copy config.toml.example to config.toml and add sandbox credentials");
            toml::from_str(&contents).expect("config.toml is malformed")
        }
    }

    #[test]
    fn new_creates_client_with_empty_credentials() {
        let mpesa = Mpesa::new();
        assert!(mpesa.consumer_key.is_empty());
        assert!(mpesa.consumer_secret.is_empty());
    }

    #[test]
    fn with_credentials_sets_consumer_key_and_secret() {
        let mpesa = Mpesa::with_credentials("test-key".into(), "test-secret".into());
        assert_eq!(mpesa.consumer_key, "test-key");
        assert_eq!(mpesa.consumer_secret, "test-secret");
    }

    #[tokio::test]
    async fn generate_access_token_fails_with_invalid_credentials() {
        let mpesa = Mpesa::with_credentials("invalid-key".into(), "invalid-secret".into());
        let err = mpesa
            .generate_access_token()
            .await
            .expect_err("expected request to fail with invalid credentials");

        assert_eq!(err.status(), Some(reqwest::StatusCode::BAD_REQUEST));
    }

    #[tokio::test]
    async fn generate_access_token_returns_valid_response() {
        let test_config = TestConfig::load();

        let mpesa = Mpesa::with_credentials(test_config.consumer_key, test_config.consumer_secret);
        let response = mpesa.generate_access_token().await.unwrap();

        assert!(!response.access_token.is_empty());

        let expires_in: u64 = response
            .expires_in
            .parse()
            .expect("expires_in should be a positive integer");
        assert!(expires_in > 0);
        assert!(expires_in <= 3600);
    }
}
