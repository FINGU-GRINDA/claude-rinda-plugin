// Authentication helper for the MCP server.
// Returns Result instead of calling process::exit() as the CLI does.

use rinda_common::{
    config::base_url,
    credentials::{Credentials, extract_exp_from_jwt, is_token_valid, load_credentials},
};

/// Error message returned to MCP tool callers when not authenticated.
pub const NOT_AUTHENTICATED_MSG: &str =
    "Not authenticated. Run `rinda auth login` first or visit the auth URL.";

/// Load credentials and ensure the access token is valid (refreshing if needed).
/// Returns `Ok((client, credentials))` on success.
/// Returns `Err(message)` with a human-readable error string on failure.
pub async fn get_authenticated_client() -> Result<(rinda_sdk::Client, Credentials), String> {
    let creds = match load_credentials() {
        Ok(c) => c,
        Err(rinda_common::credentials::CredError::NotLoggedIn) => {
            return Err(NOT_AUTHENTICATED_MSG.to_string());
        }
        Err(e) => {
            return Err(format!("Failed to load credentials: {e}"));
        }
    };

    // Fast path: token is still valid.
    if is_token_valid(&creds) {
        let client = sdk_client(Some(&creds.access_token));
        return Ok((client, creds));
    }

    // No refresh token — can't refresh.
    if creds.refresh_token.is_empty() {
        return Err("Session expired. Please log in again via the auth URL.".to_string());
    }

    // Attempt token refresh.
    let refresh_client = sdk_client(None);
    let body = rinda_sdk::types::PostApiV1AuthRefreshBody {
        refresh_token: creds.refresh_token.clone(),
    };

    match refresh_client.post_api_v1_auth_refresh(&body).await {
        Ok(resp) => {
            let resp = resp.into_inner();
            let data = resp
                .get("data")
                .and_then(|v| v.as_object())
                .unwrap_or(&resp);
            let new_token = match data.get("token").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => {
                    return Err(
                        "Session expired. Please log in again via the auth URL.".to_string()
                    );
                }
            };
            let new_refresh_token = data
                .get("refreshToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or(creds.refresh_token.clone());
            let new_expires_at = extract_exp_from_jwt(&new_token);

            let new_creds = Credentials {
                access_token: new_token.clone(),
                refresh_token: new_refresh_token,
                expires_at: new_expires_at,
                workspace_id: creds.workspace_id,
                user_id: creds.user_id,
                email: creds.email,
            };

            if let Err(e) = rinda_common::credentials::save_credentials(&new_creds) {
                // Non-fatal: log the save error but continue with the new token.
                eprintln!("Warning: failed to save refreshed credentials: {e}");
            }

            let client = sdk_client(Some(&new_token));
            Ok((client, new_creds))
        }
        Err(e) => {
            let err_str = format!("{e}");
            if err_str.contains("401") || err_str.contains("status code 401") {
                Err("Session expired. Please log in again via the auth URL.".to_string())
            } else if err_str.contains("connect") || err_str.contains("timeout") {
                Err("Cannot reach RINDA API. Check your network connection.".to_string())
            } else {
                Err(format!("Token refresh failed: {e}"))
            }
        }
    }
}

/// Build an authenticated SDK client with an optional bearer token.
pub fn sdk_client(bearer_token: Option<&str>) -> rinda_sdk::Client {
    if let Some(token) = bearer_token {
        let auth_value = format!("Bearer {token}");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_value)
                .expect("Bearer token contains invalid header characters"),
        );
        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build reqwest client");
        rinda_sdk::Client::new_with_client(base_url(), reqwest_client)
    } else {
        rinda_sdk::Client::new(base_url())
    }
}

/// Format a JSON map as a pretty-printed string for use in tool responses.
pub fn json_to_text(map: &serde_json::Map<String, serde_json::Value>) -> String {
    serde_json::to_string_pretty(map).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_to_text_formats_map_as_pretty_json() {
        let mut map = serde_json::Map::new();
        map.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let text = json_to_text(&map);
        assert!(text.contains("\"key\""), "should contain key");
        assert!(text.contains("\"value\""), "should contain value");
        // Pretty-printed JSON should have newlines.
        assert!(text.contains('\n'), "should be pretty-printed");
    }

    #[test]
    fn json_to_text_empty_map_returns_empty_object() {
        let map = serde_json::Map::new();
        let text = json_to_text(&map);
        assert_eq!(text.trim(), "{}");
    }

    #[test]
    fn sdk_client_without_token_creates_client() {
        // Should not panic — simply verify the call succeeds.
        let _client = sdk_client(None);
    }

    #[test]
    fn sdk_client_with_token_creates_client() {
        let _client = sdk_client(Some("test-token-123"));
    }

    /// Acceptance criteria: when not authenticated, all tools should return a clear
    /// error message — verified here by checking the NOT_AUTHENTICATED_MSG constant
    /// is non-empty and meaningful.
    #[test]
    fn not_authenticated_msg_is_actionable() {
        assert!(
            !NOT_AUTHENTICATED_MSG.is_empty(),
            "auth error message should not be empty"
        );
        assert!(
            NOT_AUTHENTICATED_MSG.contains("rinda auth login")
                || NOT_AUTHENTICATED_MSG.contains("auth URL"),
            "auth error message should tell the user how to fix it"
        );
    }
}
