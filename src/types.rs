/// The current app mode.
/// Sandbox is for testing while live for your live application.
/// This defaults to Sandbox for safety. You have to explitly set it to live.
#[derive(Debug, Default, PartialEq)]
pub enum DarajaEnvironment {
    #[default]
    Sandbox,
    Live,
}

pub trait DarajaApi {
    /// Returns the exact absolute URL for this specific API product
    /// depending on whether it is running in Sandbox or Production.
    fn get_url(&self, env: DarajaEnvironment) -> &'static str;

    /// Returns the specific URL path snippet for the API resource.
    fn path(&self) -> &'static str;
}
