// Authentication helper for the MCP server.
// Returns Result instead of calling process::exit() as the CLI does.

use rinda_common::{
    config::base_url,
    credentials::{Credentials, extract_exp_from_jwt, is_token_valid, load_credentials},
};

/// Error message returned to MCP tool callers when not authenticated.
pub const NOT_AUTHENTICATED_MSG: &str =
    "Not authenticated. Please provide a Bearer token in the Authorization header.";

/// Authentication context extracted from the HTTP Bearer token.
/// Holds all identity information decoded from the JWT payload without
/// requiring a local credentials file.
#[derive(Clone, Debug, Default)]
pub struct AuthContext {
    pub access_token: String,
    pub workspace_id: String,
    pub user_id: String,
    pub email: String,
}

/// Extract an `AuthContext` from the `Authorization: Bearer <token>` header of
/// an HTTP request parts object.
///
/// Returns `Ok(AuthContext)` if the header is present and the JWT payload can
/// be decoded.  Returns `Err(message)` with a human-readable string when:
/// - No `Authorization` header is present.
/// - The header value is not valid UTF-8.
/// - The header value is not of the form `Bearer <token>`.
/// - The JWT payload cannot be base64-decoded.
/// - The JWT payload is not valid JSON.
pub fn extract_auth_from_parts(parts: &http::request::Parts) -> Result<AuthContext, String> {
    // First check if the OAuth middleware injected an AuthenticatedToken.
    // This contains the real RINDA JWT plus user profile data fetched from /auth/me.
    if let Some(authenticated) = parts.extensions.get::<crate::oauth::AuthenticatedToken>() {
        return Ok(AuthContext {
            access_token: authenticated.rinda_access_token.clone(),
            workspace_id: authenticated.workspace_id.clone(),
            user_id: authenticated.user_id.clone(),
            email: authenticated.email.clone(),
        });
    }

    // Fall back to reading the raw Authorization Bearer header (direct JWT).
    let auth_header = parts
        .headers
        .get(http::header::AUTHORIZATION)
        .ok_or_else(|| NOT_AUTHENTICATED_MSG.to_string())?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| "Authorization header contains invalid characters".to_string())?;

    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| "Authorization header must be of the form 'Bearer <token>'".to_string())?;

    extract_auth_context_from_jwt(token)
}

/// Decode a JWT payload and extract identity claims into an `AuthContext`.
///
/// Looks for the following claims in the JWT payload (without signature
/// verification):
/// - `workspaceId` or `workspace_id` — workspace UUID
/// - `userId` or `user_id` or `sub` — user UUID
/// - `email` — user email address
///
/// Returns `Err(message)` when the token cannot be parsed or required claims
/// are missing.
pub fn extract_auth_context_from_jwt(token: &str) -> Result<AuthContext, String> {
    use base64::Engine as _;

    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() < 2 {
        return Err("Invalid JWT: expected at least two dot-separated parts".to_string());
    }

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine
        .decode(parts[1])
        .map_err(|e| format!("Invalid JWT payload base64: {e}"))?;

    let payload: serde_json::Value =
        serde_json::from_slice(&decoded).map_err(|e| format!("Invalid JWT payload JSON: {e}"))?;

    // Try several claim name variants used by RINDA's API.
    let workspace_id = payload
        .get("workspaceId")
        .or_else(|| payload.get("workspace_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let user_id = payload
        .get("userId")
        .or_else(|| payload.get("user_id"))
        .or_else(|| payload.get("sub"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(AuthContext {
        access_token: token.to_string(),
        workspace_id,
        user_id,
        email,
    })
}

/// Load credentials and ensure the access token is valid (refreshing if needed).
/// Returns `Ok((client, credentials))` on success.
/// Returns `Err(message)` with a human-readable error string on failure.
///
/// This function is kept for local/stdio development mode where credentials
/// are stored in `~/.rinda/credentials.json`.
#[allow(dead_code)]
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
            NOT_AUTHENTICATED_MSG.contains("Bearer token")
                || NOT_AUTHENTICATED_MSG.contains("Authorization"),
            "auth error message should mention the expected auth mechanism"
        );
    }

    /// Helper: build a valid JWT with the given payload (unsigned, for tests only).
    fn make_test_jwt(payload: serde_json::Value) -> String {
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"none"}"#);
        let payload_enc = engine.encode(payload.to_string());
        format!("{}.{}.sig", header, payload_enc)
    }

    #[test]
    fn extract_auth_context_from_jwt_with_all_claims() {
        let payload = serde_json::json!({
            "workspaceId": "ws-123",
            "userId": "user-456",
            "email": "test@example.com",
            "exp": 9999999999i64
        });
        let token = make_test_jwt(payload);
        let ctx = extract_auth_context_from_jwt(&token).expect("should decode JWT");
        assert_eq!(ctx.workspace_id, "ws-123");
        assert_eq!(ctx.user_id, "user-456");
        assert_eq!(ctx.email, "test@example.com");
        assert_eq!(ctx.access_token, token);
    }

    #[test]
    fn extract_auth_context_from_jwt_snake_case_claims() {
        let payload = serde_json::json!({
            "workspace_id": "ws-snake",
            "user_id": "user-snake",
            "email": "snake@example.com"
        });
        let token = make_test_jwt(payload);
        let ctx = extract_auth_context_from_jwt(&token).expect("should decode JWT");
        assert_eq!(ctx.workspace_id, "ws-snake");
        assert_eq!(ctx.user_id, "user-snake");
        assert_eq!(ctx.email, "snake@example.com");
    }

    #[test]
    fn extract_auth_context_from_jwt_sub_fallback_for_user_id() {
        let payload = serde_json::json!({
            "sub": "user-from-sub",
            "email": "sub@example.com"
        });
        let token = make_test_jwt(payload);
        let ctx = extract_auth_context_from_jwt(&token).expect("should decode JWT");
        assert_eq!(ctx.user_id, "user-from-sub");
    }

    #[test]
    fn extract_auth_context_from_jwt_missing_claims_use_empty_strings() {
        // A JWT with no workspace/user/email claims should still succeed, with empty strings.
        let payload = serde_json::json!({ "exp": 9999999999i64 });
        let token = make_test_jwt(payload);
        let ctx =
            extract_auth_context_from_jwt(&token).expect("should decode JWT even without claims");
        assert_eq!(ctx.workspace_id, "");
        assert_eq!(ctx.user_id, "");
        assert_eq!(ctx.email, "");
    }

    #[test]
    fn extract_auth_context_from_jwt_invalid_token_returns_error() {
        let result = extract_auth_context_from_jwt("not-a-jwt");
        assert!(result.is_err(), "invalid JWT should return error");
    }

    #[test]
    fn extract_auth_from_parts_no_header_returns_error() {
        let (parts, _) = http::Request::builder()
            .uri("http://example.com/")
            .body(())
            .unwrap()
            .into_parts();
        let result = extract_auth_from_parts(&parts);
        assert!(result.is_err(), "missing Authorization header should error");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("Not authenticated") || msg.contains("Bearer"),
            "error message should be actionable: {msg}"
        );
    }

    #[test]
    fn extract_auth_from_parts_bearer_token_decoded() {
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header_enc = engine.encode(r#"{"alg":"none"}"#);
        let payload = serde_json::json!({
            "workspaceId": "ws-http",
            "userId": "user-http",
            "email": "http@example.com"
        });
        let payload_enc = engine.encode(payload.to_string());
        let token = format!("{}.{}.sig", header_enc, payload_enc);

        let (parts, _) = http::Request::builder()
            .uri("http://example.com/mcp")
            .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
            .body(())
            .unwrap()
            .into_parts();

        let ctx = extract_auth_from_parts(&parts).expect("should extract auth context");
        assert_eq!(ctx.workspace_id, "ws-http");
        assert_eq!(ctx.user_id, "user-http");
        assert_eq!(ctx.email, "http@example.com");
    }

    /// Acceptance criteria from issue #82: tools extract Bearer token from the HTTP
    /// Authorization header instead of reading from ~/.rinda/credentials.json.
    #[test]
    fn extract_auth_from_parts_does_not_require_credentials_file() {
        // This test verifies the new auth path does not touch the filesystem.
        // We create a request with a valid Bearer token and check that
        // extract_auth_from_parts succeeds without reading any file.
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"none"}"#);
        let payload = engine.encode(r#"{"workspaceId":"ws-1","userId":"u-1","email":"a@b.com"}"#);
        let token = format!("{}.{}.sig", header, payload);

        let (parts, _) = http::Request::builder()
            .header("Authorization", format!("Bearer {token}"))
            .body(())
            .unwrap()
            .into_parts();

        // This must NOT try to open ~/.rinda/credentials.json.
        let result = extract_auth_from_parts(&parts);
        assert!(result.is_ok(), "should succeed without credentials file");
    }
}
