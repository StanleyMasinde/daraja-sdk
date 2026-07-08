/// The current app mode.
/// Sandbox is for testing while live for your live application.
/// This defaults to Sandbox for safety. You have to explitly set it to live.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum DarajaEnvironment {
    #[default]
    Sandbox,
    Live,
}

pub trait DarajaApi {
    /// Returns the specific URL path snippet for the API resource.
    fn path(&self) -> &'static str;

    /// Returns the current environment.
    fn environment(&self) -> DarajaEnvironment;

    /// Get the full URL for the particlar endpoint while respecting the
    /// environment.
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
        struct ReversalAPI {}
        impl DarajaApi for ReversalAPI {
            fn path(&self) -> &'static str {
                "/api/v1/revese"
            }

            fn environment(&self) -> super::DarajaEnvironment {
                DarajaEnvironment::Live
            }
        }

        let reverse_api = ReversalAPI {};
        assert_eq!(reverse_api.environment(), DarajaEnvironment::Live);
        assert_eq!(
            reverse_api.get_url(),
            format!("https://api.safaricom.co.ke/{}", reverse_api.path())
        )
    }
}
