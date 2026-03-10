use std::path::PathBuf;

/// Returns the path to the rinda configuration directory (~/.rinda).
pub fn rinda_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".rinda"))
}

/// Returns the path to the credentials file (~/.rinda/credentials.json).
pub fn credentials_path() -> Option<PathBuf> {
    rinda_config_dir().map(|dir| dir.join("credentials.json"))
}

/// Base URL for the Rinda API.
/// Override with RINDA_BASE_URL env var (e.g. "https://alpha.rinda.ai" for testing).
pub fn base_url() -> &'static str {
    static URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    URL.get_or_init(|| {
        std::env::var("RINDA_BASE_URL").unwrap_or_else(|_| "https://app.rinda.ai".to_string())
    })
}
