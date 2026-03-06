use reqwest::{RequestBuilder, Response};
use serde::de::DeserializeOwned;

use rinda_common::config::BASE_URL;

use crate::error::{Result, SdkError};

/// HTTP client wrapper for the Rinda API.
pub struct RindaClient {
    inner: reqwest::Client,
    base_url: String,
    access_token: Option<String>,
}

impl RindaClient {
    /// Creates a new `RindaClient` with the default base URL.
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
            base_url: BASE_URL.to_string(),
            access_token: None,
        }
    }

    /// Creates a new `RindaClient` with a custom base URL.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            inner: reqwest::Client::new(),
            base_url: base_url.into(),
            access_token: None,
        }
    }

    /// Sets the access token and returns `&mut Self` for chaining.
    pub fn with_access_token(&mut self, token: impl Into<String>) -> &mut Self {
        self.access_token = Some(token.into());
        self
    }

    /// Sets the access token.
    pub fn set_access_token(&mut self, token: impl Into<String>) {
        self.access_token = Some(token.into());
    }

    /// Returns a reference to the underlying reqwest client.
    pub fn http(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Returns the base URL of the API.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the current access token, if set.
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    // ---- private HTTP helpers ----

    fn apply_auth(&self, rb: RequestBuilder) -> RequestBuilder {
        if let Some(token) = &self.access_token {
            rb.bearer_auth(token)
        } else {
            rb
        }
    }

    pub(crate) fn get(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.inner.get(url))
    }

    pub(crate) fn post(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.inner.post(url))
    }

    pub(crate) fn put(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.inner.put(url))
    }

    pub(crate) fn patch(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.inner.patch(url))
    }

    pub(crate) fn delete(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.inner.delete(url))
    }

    /// Deserializes a successful response or converts an error response into `SdkError::Api`.
    pub(crate) async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T> {
        let status = response.status();
        if status.is_success() {
            let body = response.json::<T>().await?;
            Ok(body)
        } else {
            let status_code = status.as_u16();
            let message = response.text().await.unwrap_or_else(|_| status.to_string());
            Err(SdkError::Api {
                status: status_code,
                message,
            })
        }
    }

    /// Like `handle_response` but discards the body on success (returns `()`).
    pub(crate) async fn handle_response_empty(response: Response) -> Result<()> {
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let status_code = status.as_u16();
            let message = response.text().await.unwrap_or_else(|_| status.to_string());
            Err(SdkError::Api {
                status: status_code,
                message,
            })
        }
    }
}

impl Default for RindaClient {
    fn default() -> Self {
        Self::new()
    }
}
