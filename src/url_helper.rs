use crate::types::DarajaEnvironment;

/// Daraja supports two modes. Same endpoints different subdomains.
///
/// # Arguments
///
/// * `app_mode` - This can be live or sandbox.
/// * `path` - The endpoint being called.
pub fn url_helper(environment: DarajaEnvironment, path: &str) -> String {
    let url_prefix = match environment {
        DarajaEnvironment::Sandbox => "sandbox",
        DarajaEnvironment::Live => "live",
    };

    format!("https://{}.safaricom.co.ke/{}", url_prefix, path)
}
