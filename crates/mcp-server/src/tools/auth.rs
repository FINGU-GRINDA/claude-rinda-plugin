// Auth tool implementations: rinda_auth_status, rinda_auth_login.

use rinda_common::{config::base_url, credentials::Credentials};

use crate::auth;

/// Implementation for the rinda_auth_status tool.
/// Returns the current authentication status (email, workspace, token expiry).
pub async fn auth_status() -> String {
    if !Credentials::exists() {
        return serde_json::json!({
            "authenticated": false,
            "message": auth::NOT_AUTHENTICATED_MSG
        })
        .to_string();
    }

    match Credentials::load() {
        Ok(creds) => {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            let diff_ms = creds.expires_at - now_ms;
            let token_status = if diff_ms > 0 {
                let mins = diff_ms / 60_000;
                if mins >= 60 {
                    format!("expires in {} hour(s)", mins / 60)
                } else {
                    format!("expires in {mins} minute(s)")
                }
            } else {
                let ago_ms = now_ms - creds.expires_at;
                let mins = ago_ms / 60_000;
                if mins >= 60 {
                    format!("expired {} hour(s) ago", mins / 60)
                } else {
                    format!("expired {mins} minute(s) ago")
                }
            };

            serde_json::json!({
                "authenticated": true,
                "email": creds.email,
                "workspace_id": creds.workspace_id,
                "user_id": creds.user_id,
                "token_status": token_status,
                "expires_at_ms": creds.expires_at
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "authenticated": false,
            "error": format!("Error reading credentials: {e}")
        })
        .to_string(),
    }
}

/// Implementation for the rinda_auth_login tool.
/// Returns the browser login URL and instructions.
pub async fn auth_login() -> String {
    let url = format!("{}/cli-auth", base_url());
    serde_json::json!({
        "auth_url": url,
        "instructions": "Open the URL in your browser, complete Google login, then copy the refresh token and run: rinda auth token <TOKEN>"
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acceptance criteria: rinda_auth_login should return a URL and instructions
    /// without requiring credentials (works when not authenticated).
    #[tokio::test]
    async fn auth_login_returns_url_and_instructions() {
        let result = auth_login().await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");

        let url = parsed["auth_url"].as_str().expect("should have auth_url");
        assert!(
            url.contains("cli-auth"),
            "auth URL should contain 'cli-auth'"
        );
        assert!(url.starts_with("https://"), "auth URL should be HTTPS");

        let instructions = parsed["instructions"]
            .as_str()
            .expect("should have instructions");
        assert!(!instructions.is_empty(), "instructions should not be empty");
    }

    /// Acceptance criteria: rinda_auth_status should return authenticated=false with
    /// a clear message when no credentials file exists (not logged in).
    #[tokio::test]
    async fn auth_status_returns_not_authenticated_when_no_credentials() {
        // In the test environment there are typically no credentials.
        // If credentials happen to exist, skip this check (CI may have them).
        if Credentials::exists() {
            return;
        }
        let result = auth_status().await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");

        assert_eq!(
            parsed["authenticated"].as_bool(),
            Some(false),
            "should report not authenticated"
        );
        assert!(
            parsed["message"].as_str().is_some(),
            "should include a message field"
        );
    }

    /// Auth login response should be parseable as JSON regardless of environment.
    #[tokio::test]
    async fn auth_login_response_is_valid_json() {
        let result = auth_login().await;
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
        assert!(parsed.is_ok(), "auth_login should always return valid JSON");
    }
}
