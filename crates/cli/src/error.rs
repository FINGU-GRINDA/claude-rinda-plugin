use thiserror::Error;

#[derive(Debug, Error)]
pub enum RindaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    #[allow(dead_code)]
    Auth(String),

    #[error("API error: {0}")]
    #[allow(dead_code)]
    Api(String),
}

pub type Result<T> = std::result::Result<T, RindaError>;
