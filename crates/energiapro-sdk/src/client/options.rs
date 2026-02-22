use std::time::Duration;

/// Configuration options for the EnergiaPro API client.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// Base URL for requests to the EnergiaPro API.
    pub base_url: String,

    /// Timeout for requests to the EnergiaPro API.
    pub timeout: Duration,
}

/// Default options for the EnergiaPro API client.
impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            base_url: "https://web2.holdigaz.ch/espace-client-api/api".to_owned(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl ClientOptions {
    /// Set a custom base URL for requests to the EnergiaPro API.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set a custom timeout for requests to the EnergiaPro API.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
