use serde::Deserialize;

use crate::types::{DarajaApi, DarajaEnvironment};

/// The response from Daraja when you request an *access_token*.
///
/// Returned by [`Client::generate_access_token`].
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
/// `Client` holds the configuration state needed to build requests against the Daraja M-Pesa API.
/// It borrows the consumer key and secret rather than owning them — credentials must remain
/// valid for as long as the `Client` exists.
///
/// Construct it with [`Client::new`] for defaults, or the recommended [`Client::with_credentials`]
/// to supply a consumer key and secret up front.
pub struct Client<'a> {
    /// The consumer key of your app in Daraja.
    pub consumer_key: &'a str,

    /// The consumer secret of your app in Daraja.
    pub consumer_secret: &'a str,

    /// The current environment.
    pub environment: DarajaEnvironment,
}

impl Default for Client<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl DarajaApi for Client<'_> {
    fn path(&self) -> &'static str {
        "oauth/v1/generate?grant_type=client_credentials"
    }

    fn environment(&self) -> DarajaEnvironment {
        self.environment
    }

    fn production(mut self) -> Self {
        self.environment = DarajaEnvironment::Live;
        self
    }
}

impl<'a> Client<'a> {
    /// Creates a new `Client` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use daraja_sdk::mpesa;
    ///
    /// let client = mpesa::Client::new();
    /// ```
    pub fn new() -> Self {
        Self {
            consumer_key: "",
            consumer_secret: "",
            environment: DarajaEnvironment::default(),
        }
    }

    /// Creates a `Client` preconfigured with consumer_key and consumer_secret.
    ///
    /// Credentials are borrowed, not copied. They must remain valid for as long as
    /// the returned `Client` is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use daraja_sdk::mpesa;
    ///
    /// let client = mpesa::Client::with_credentials("consumer_key", "consumer_secret");
    /// ```
    pub fn with_credentials(consumer_key: &'a str, consumer_secret: &'a str) -> Self {
        Self {
            consumer_key,
            consumer_secret,
            environment: DarajaEnvironment::default(),
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
    /// use daraja_sdk::mpesa;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), reqwest::Error> {
    ///     let client = mpesa::Client::with_credentials("consumer_key", "consumer_secret");
    ///
    ///     let token = client.generate_access_token().await?;
    ///     println!("{}", token.access_token);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn generate_access_token(
        self,
    ) -> Result<GenerateAccessTokenResponse, reqwest::Error> {
        let http_client = reqwest::Client::new();
        http_client
            .get(self.get_url())
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
    use crate::mpesa::test_support::{
        TestConfig, assert_valid_access_token, assert_valid_expires_in, read_token_cache,
        write_token_cache,
    };

    #[test]
    fn new_creates_client_with_empty_credentials() {
        let client = Client::new();
        assert!(client.consumer_key.is_empty());
        assert!(client.consumer_secret.is_empty());
    }

    #[test]
    fn with_credentials_sets_consumer_key_and_secret() {
        let client = Client::with_credentials("test-key".into(), "test-secret".into());
        assert_eq!(client.consumer_key, "test-key");
        assert_eq!(client.consumer_secret, "test-secret");
    }

    #[tokio::test]
    async fn generate_access_token_fails_with_invalid_credentials() {
        let client = Client::with_credentials("invalid-key".into(), "invalid-secret".into());
        let err = client
            .generate_access_token()
            .await
            .expect_err("expected request to fail with invalid credentials");

        assert_eq!(err.status(), Some(reqwest::StatusCode::BAD_REQUEST));
    }

    #[tokio::test]
    async fn generate_access_token_returns_valid_response() {
        let test_config = TestConfig::load();

        let client =
            Client::with_credentials(&test_config.consumer_key, &test_config.consumer_secret);

        // Reuse a cached token when still valid to reduce live API calls.
        if let Some((cached_token, expires_at)) = read_token_cache() {
            assert_valid_access_token(&cached_token);
            let remaining = expires_at
                .duration_since(std::time::SystemTime::now())
                .expect("cached token should not be expired");
            assert_valid_expires_in(remaining.as_secs());
            return;
        }

        let response = client.generate_access_token().await.unwrap();

        assert_valid_access_token(&response.access_token);

        let expires_in: u64 = response
            .expires_in
            .parse()
            .expect("expires_in should be a positive integer");
        assert_valid_expires_in(expires_in);

        write_token_cache(&response.access_token, expires_in);
    }
}
