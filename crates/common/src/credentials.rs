// Credentials file read/write (~/.rinda/credentials.json).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::config::{credentials_path, rinda_config_dir};
use crate::error::{Result, RindaError};

/// Credentials stored in ~/.rinda/credentials.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
    /// Milliseconds since UNIX epoch when the access token expires.
    pub expires_at: i64,
    pub workspace_id: String,
    pub user_id: String,
    pub email: String,
}

impl Credentials {
    /// Load credentials from ~/.rinda/credentials.json.
    pub fn load() -> Result<Credentials> {
        let path = credentials_path()
            .ok_or_else(|| RindaError::Config("Cannot determine home directory".into()))?;
        load_credentials_from(&path)
            .map_err(|e| RindaError::Config(format!("Failed to load credentials: {e}")))
    }

    /// Save credentials to ~/.rinda/credentials.json with permissions 0o600 (Unix).
    pub fn save(&self) -> Result<()> {
        let dir = rinda_config_dir()
            .ok_or_else(|| RindaError::Config("Cannot determine home directory".into()))?;
        fs::create_dir_all(&dir)?;

        let path = dir.join("credentials.json");
        save_credentials_to(self, &path)
            .map_err(|e| RindaError::Config(format!("Failed to save credentials: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Delete the credentials file. Ignores NotFound errors (idempotent).
    pub fn delete() -> Result<()> {
        let path = credentials_path()
            .ok_or_else(|| RindaError::Config("Cannot determine home directory".into()))?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(RindaError::Io(e)),
        }
    }

    /// Returns true if the credentials file exists on disk.
    pub fn exists() -> bool {
        credentials_path().map(|p| p.exists()).unwrap_or(false)
    }
}

/// Error type for credential operations used by ensure-valid flow.
#[derive(Debug, thiserror::Error)]
pub enum CredError {
    #[error("Not logged in. Run: rinda auth url")]
    NotLoggedIn,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Configuration error: no home directory found")]
    NoHomeDir,
}

pub type CredResult<T> = std::result::Result<T, CredError>;

/// Load credentials from a specific path.
/// Acquires a shared (read) lock via a `.lock` file while reading.
pub fn load_credentials_from(path: &Path) -> CredResult<Credentials> {
    let lock_path = path.with_extension("lock");

    let lock_file = match File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CredError::NotLoggedIn);
        }
        Err(e) => return Err(CredError::Io(e)),
    };
    fs2::FileExt::lock_shared(&lock_file)?;

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = fs2::FileExt::unlock(&lock_file);
            return Err(CredError::NotLoggedIn);
        }
        Err(e) => {
            let _ = fs2::FileExt::unlock(&lock_file);
            return Err(CredError::Io(e));
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    fs2::FileExt::unlock(&lock_file)?;

    let creds: Credentials = serde_json::from_str(&contents)?;
    Ok(creds)
}

/// Save credentials to a specific path atomically.
/// Writes to a temp file in the same directory and renames to avoid partial writes.
/// Acquires an exclusive lock via a `.lock` file while writing.
pub fn save_credentials_to(creds: &Credentials, path: &Path) -> CredResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let lock_path = path.with_extension("lock");

    let lock_file = File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;
    fs2::FileExt::lock_exclusive(&lock_file)?;

    let json = serde_json::to_string_pretty(creds)?;

    let tmp_path = path.with_extension("tmp");
    {
        let mut tmp_file = File::create(&tmp_path)?;
        tmp_file.write_all(json.as_bytes())?;
        tmp_file.flush()?;
    }
    fs::rename(&tmp_path, path)?;

    fs2::FileExt::unlock(&lock_file)?;

    Ok(())
}

/// Load credentials from ~/.rinda/credentials.json (for ensure-valid flow).
pub fn load_credentials() -> CredResult<Credentials> {
    load_credentials_from(&cred_path()?)
}

/// Save credentials to ~/.rinda/credentials.json atomically (for ensure-valid flow).
pub fn save_credentials(creds: &Credentials) -> CredResult<()> {
    save_credentials_to(creds, &cred_path()?)
}

/// Returns true if the token in `creds` is valid for at least 5 more minutes.
/// Uses millisecond timestamp comparison for speed.
pub fn is_token_valid(creds: &Credentials) -> bool {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let buffer_ms: i64 = 5 * 60 * 1000; // 5 minutes
    creds.expires_at - now_ms > buffer_ms
}

/// Decode the `exp` claim from a JWT token (without signature verification).
/// Returns milliseconds since UNIX epoch.
/// Falls back to `now + 1 hour` if decoding fails.
pub fn extract_exp_from_jwt(token: &str) -> i64 {
    try_extract_exp_from_jwt(token).unwrap_or_else(|| {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        now_ms + 3_600_000 // 1 hour in ms
    })
}

fn try_extract_exp_from_jwt(token: &str) -> Option<i64> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() < 2 {
        return None;
    }

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine.decode(parts[1]).ok()?;
    let payload: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    // JWT `exp` is in seconds; convert to milliseconds.
    let exp_secs = payload.get("exp")?.as_i64()?;
    Some(exp_secs * 1000)
}

fn cred_path() -> CredResult<PathBuf> {
    credentials_path().ok_or(CredError::NoHomeDir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_credentials() -> Credentials {
        Credentials {
            access_token: "tok_abc".into(),
            refresh_token: "ref_xyz".into(),
            expires_at: 9_999_999_999_000,
            workspace_id: "ws-1".into(),
            user_id: "user-1".into(),
            email: "test@example.com".into(),
        }
    }

    #[test]
    fn credentials_serialize_camel_case() {
        let creds = sample_credentials();
        let json = serde_json::to_value(&creds).expect("serialization failed");
        assert!(json.get("accessToken").is_some(), "accessToken key missing");
        assert!(
            json.get("refreshToken").is_some(),
            "refreshToken key missing"
        );
        assert!(json.get("expiresAt").is_some(), "expiresAt key missing");
        assert!(json.get("workspaceId").is_some(), "workspaceId key missing");
        assert!(json.get("userId").is_some(), "userId key missing");
        assert!(json.get("email").is_some(), "email key missing");
        assert!(json.get("access_token").is_none());
        assert!(json.get("refresh_token").is_none());
    }

    #[test]
    fn credentials_round_trip_json() {
        let original = sample_credentials();
        let json = serde_json::to_string(&original).expect("serialization failed");
        let decoded: Credentials = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(decoded.access_token, original.access_token);
        assert_eq!(decoded.refresh_token, original.refresh_token);
        assert_eq!(decoded.expires_at, original.expires_at);
        assert_eq!(decoded.workspace_id, original.workspace_id);
        assert_eq!(decoded.user_id, original.user_id);
        assert_eq!(decoded.email, original.email);
    }

    #[test]
    fn credentials_deserialize_from_camel_case_json() {
        let json = r#"{
            "accessToken": "a",
            "refreshToken": "b",
            "expiresAt": 1000,
            "workspaceId": "ws",
            "userId": "u",
            "email": "a@b.com"
        }"#;
        let creds: Credentials = serde_json::from_str(json).expect("deserialization failed");
        assert_eq!(creds.access_token, "a");
        assert_eq!(creds.refresh_token, "b");
        assert_eq!(creds.expires_at, 1000);
        assert_eq!(creds.workspace_id, "ws");
        assert_eq!(creds.user_id, "u");
        assert_eq!(creds.email, "a@b.com");
    }

    #[test]
    fn delete_is_idempotent_when_file_missing() {
        let path = std::path::PathBuf::from("/tmp/rinda_test_nonexistent_credentials.json");
        let _ = std::fs::remove_file(&path);
        let result: std::io::Result<()> = match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        };
        assert!(
            result.is_ok(),
            "expected Ok for missing file, got {result:?}"
        );
    }

    #[test]
    fn test_is_token_valid_future() {
        let mut creds = sample_credentials();
        // Set expires_at to 1 hour from now.
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        creds.expires_at = now_ms + 3_600_000;
        assert!(is_token_valid(&creds));
    }

    #[test]
    fn test_is_token_valid_expired() {
        let mut creds = sample_credentials();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        creds.expires_at = now_ms - 3_600_000;
        assert!(!is_token_valid(&creds));
    }

    #[test]
    fn test_is_token_valid_within_buffer() {
        let mut creds = sample_credentials();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        // 3 minutes from now (within 5-minute buffer).
        creds.expires_at = now_ms + 180_000;
        assert!(!is_token_valid(&creds));
    }

    #[test]
    fn test_is_token_valid_just_outside_buffer() {
        let mut creds = sample_credentials();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        // 6 minutes from now (just outside 5-minute buffer).
        creds.expires_at = now_ms + 360_000;
        assert!(is_token_valid(&creds));
    }

    #[test]
    fn test_extract_exp_from_jwt_known_token() {
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"none"}"#);
        let payload = engine.encode(r#"{"exp":2000000000}"#);
        let token = format!("{}.{}.sig", header, payload);

        let exp_ms = extract_exp_from_jwt(&token);
        assert_eq!(exp_ms, 2_000_000_000_000); // 2000000000 seconds * 1000
    }

    #[test]
    fn test_extract_exp_from_jwt_invalid_falls_back() {
        let exp_ms = extract_exp_from_jwt("not.a.jwt");
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let diff = exp_ms - now_ms;
        // Should be approximately 1 hour (3_600_000 ms).
        assert!(diff > 3_500_000);
        assert!(diff < 3_700_000);
    }

    #[test]
    fn test_save_and_load_credentials_roundtrip() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let path = dir.path().join("credentials.json");

        let creds = sample_credentials();
        save_credentials_to(&creds, &path).expect("save failed");
        let loaded = load_credentials_from(&path).expect("load failed");

        assert_eq!(loaded.access_token, creds.access_token);
        assert_eq!(loaded.refresh_token, creds.refresh_token);
        assert_eq!(loaded.expires_at, creds.expires_at);
        assert_eq!(loaded.workspace_id, creds.workspace_id);
        assert_eq!(loaded.email, creds.email);
    }

    #[test]
    fn test_load_credentials_from_not_found() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let path = dir.path().join("nonexistent.json");

        let result = load_credentials_from(&path);
        assert!(matches!(result, Err(CredError::NotLoggedIn)));
    }
}
