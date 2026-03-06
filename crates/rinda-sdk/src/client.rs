/// HTTP client wrapper for the Rinda API.
pub struct RindaClient {
    inner: reqwest::Client,
    base_url: String,
}

impl RindaClient {
    /// Creates a new `RindaClient` with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            inner: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Returns a reference to the underlying reqwest client.
    pub fn http(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Returns the base URL of the API.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
