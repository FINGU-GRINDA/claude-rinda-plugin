use thiserror::Error;

use rinda_common::error::RindaError;

/// Errors that can occur when using the Rinda SDK.
#[derive(Debug, Error)]
pub enum SdkError {
    /// A network or transport-level error from reqwest.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API returned a non-2xx status code.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// A common rinda error (config, auth, io).
    #[error("Rinda error: {0}")]
    Common(#[from] RindaError),

    /// A JSON deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience result type for SDK operations.
pub type Result<T> = std::result::Result<T, SdkError>;
