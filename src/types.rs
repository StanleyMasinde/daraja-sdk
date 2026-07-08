//! Shared types for building Daraja API requests.
//!
//! Each API client in this crate implements [`DarajaApi`] and selects its target
//! environment independently. Call [`.production()`](DarajaApi::production) on a
//! builder to target the live Daraja API; otherwise requests use the sandbox.

/// The Daraja API environment a client targets.
///
/// Defaults to [`Sandbox`](Self::Sandbox) so new builders are safe for development
/// without extra configuration.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum DarajaEnvironment {
    /// The Daraja sandbox (`sandbox.safaricom.co.ke`).
    #[default]
    Sandbox,
    /// The Daraja production API (`api.safaricom.co.ke`).
    Live,
}

/// Common behavior for Daraja API endpoint builders.
///
/// Implementors supply the endpoint path and current [`DarajaEnvironment`].
/// [`get_url`](Self::get_url) combines them into a full request URL.
///
/// Each endpoint builder exposes [`.production()`](Self::production) so callers
/// can opt into the live environment without sharing configuration across APIs.
pub trait DarajaApi {
    /// Returns the URL path snippet for this endpoint (no leading slash).
    ///
    /// Example: `"oauth/v1/generate?grant_type=client_credentials"`.
    fn path(&self) -> &'static str;

    /// Returns the environment this builder will target.
    fn environment(&self) -> DarajaEnvironment;

    /// Targets the Daraja production API (`api.safaricom.co.ke`).
    ///
    /// Call this on a builder before sending a request when using production
    /// credentials and passkeys. Each API builder must be configured
    /// independently.
    fn production(mut self) -> Self
    where
        Self: Sized,
    {
        self.set_environment(DarajaEnvironment::Live);
        self
    }

    /// Sets the environment used by this builder.
    fn set_environment(&mut self, environment: DarajaEnvironment);

    /// Returns the full URL for this endpoint in the builder's current environment.
    ///
    /// Sandbox requests use `https://sandbox.safaricom.co.ke/...`; production
    /// requests use `https://api.safaricom.co.ke/...`.
    fn get_url(&self) -> String {
        let url_prefix = match self.environment() {
            DarajaEnvironment::Sandbox => "sandbox",
            DarajaEnvironment::Live => "api",
        };

        format!("https://{}.safaricom.co.ke/{}", url_prefix, self.path())
    }
}

#[cfg(test)]
mod test {
    use crate::types::{DarajaApi, DarajaEnvironment};

    #[test]
    fn test_daraja_api_trait() {
        struct ReversalAPI {
            environment: DarajaEnvironment,
        }

        impl DarajaApi for ReversalAPI {
            fn path(&self) -> &'static str {
                "mpesa/reversal/v1/request"
            }

            fn environment(&self) -> DarajaEnvironment {
                self.environment
            }

            fn set_environment(&mut self, environment: DarajaEnvironment) {
                self.environment = environment;
            }
        }

        let sandbox_api = ReversalAPI {
            environment: DarajaEnvironment::Sandbox,
        };
        assert_eq!(
            sandbox_api.get_url(),
            "https://sandbox.safaricom.co.ke/mpesa/reversal/v1/request"
        );

        let live_api = sandbox_api.production();
        assert_eq!(live_api.environment(), DarajaEnvironment::Live);
        assert_eq!(
            live_api.get_url(),
            "https://api.safaricom.co.ke/mpesa/reversal/v1/request"
        );
    }
}
