use std::path::PathBuf;

/// Returns the path to the rinda configuration directory (~/.rinda).
pub fn rinda_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".rinda"))
}

/// Returns the path to the credentials file (~/.rinda/credentials.json).
pub fn credentials_path() -> Option<PathBuf> {
    rinda_config_dir().map(|dir| dir.join("credentials.json"))
}

/// Base URL for the Rinda API (host only, no path prefix).
pub const BASE_URL: &str = "https://alpha.rinda.ai";
