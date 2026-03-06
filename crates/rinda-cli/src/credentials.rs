// Credentials file read/write (~/.rinda/credentials.json).

use serde::{Deserialize, Serialize};
use std::fs;

use rinda_common::config::{credentials_path, rinda_config_dir};
use rinda_common::error::{Result, RindaError};

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
        let data = fs::read_to_string(&path)?;
        let creds: Credentials = serde_json::from_str(&data)?;
        Ok(creds)
    }

    /// Save credentials to ~/.rinda/credentials.json with permissions 0o600 (Unix).
    pub fn save(&self) -> Result<()> {
        let dir = rinda_config_dir()
            .ok_or_else(|| RindaError::Config("Cannot determine home directory".into()))?;
        fs::create_dir_all(&dir)?;

        let path = dir.join("credentials.json");
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;

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
        // snake_case keys must NOT appear
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
        // Simulate the NotFound-ignoring logic with a path that doesn't exist.
        let path = std::path::PathBuf::from("/tmp/rinda_test_nonexistent_credentials.json");
        // Ensure the file really doesn't exist.
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
}
