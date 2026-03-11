use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Returns the path to the rinda configuration directory (~/.rinda).
pub fn rinda_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".rinda"))
}

/// Returns the path to the credentials file (~/.rinda/credentials.json).
pub fn credentials_path() -> Option<PathBuf> {
    rinda_config_dir().map(|dir| dir.join("credentials.json"))
}

/// Returns the path to the config file (~/.rinda/config.json).
pub fn config_path() -> Option<PathBuf> {
    rinda_config_dir().map(|dir| dir.join("config.json"))
}

/// Persistent CLI configuration stored at ~/.rinda/config.json.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RindaConfig {
    /// "alpha" or "beta" (default: "beta")
    #[serde(default = "default_env")]
    pub env: String,
}

fn default_env() -> String {
    "beta".to_string()
}

impl RindaConfig {
    /// Load config from disk, returning defaults if file doesn't exist.
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save config to disk.
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = config_path() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot determine config directory",
            ));
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, json)
    }
}

/// Resolve the base URL from config.
/// Priority: RINDA_API_BASE_URL env var > config file > default (https://app.rinda.ai)
/// "alpha" → https://alpha.rinda.ai, "beta" (default) → https://app.rinda.ai
pub fn base_url() -> &'static str {
    static URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    URL.get_or_init(|| {
        // Environment variable takes precedence over config file
        if let Ok(url) = std::env::var("RINDA_API_BASE_URL")
            && !url.is_empty()
        {
            return url;
        }
        let config = RindaConfig::load();
        match config.env.as_str() {
            "alpha" => "https://alpha.rinda.ai".to_string(),
            _ => "https://app.rinda.ai".to_string(),
        }
    })
}
