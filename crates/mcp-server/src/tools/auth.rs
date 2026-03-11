// Auth tool implementations: rinda_auth_status, rinda_auth_login.

use crate::auth;

/// Implementation for the rinda_auth_status tool.
/// Returns the current authentication status extracted from the HTTP Bearer token.
/// If no Bearer token is present, returns an unauthenticated response.
pub async fn auth_status(parts: Option<&http::request::Parts>) -> String {
    let parts = match parts {
        Some(p) => p,
        None => {
            return serde_json::json!({
                "authenticated": false,
                "message": auth::NOT_AUTHENTICATED_MSG
            })
            .to_string();
        }
    };

    match auth::extract_auth_from_parts(parts) {
        Ok(ctx) => {
            // Decode expiry from the JWT for informational purposes.
            use base64::Engine as _;
            let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
            let token_status = if let Some(payload_part) = ctx.access_token.split('.').nth(1) {
                if let Ok(decoded) = engine.decode(payload_part) {
                    if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as i64)
                            .unwrap_or(0);
                        if let Some(exp_secs) = payload.get("exp").and_then(|v| v.as_i64()) {
                            let exp_ms = exp_secs * 1000;
                            let diff_ms = exp_ms - now_ms;
                            if diff_ms > 0 {
                                let mins = diff_ms / 60_000;
                                if mins >= 60 {
                                    format!("expires in {} hour(s)", mins / 60)
                                } else {
                                    format!("expires in {mins} minute(s)")
                                }
                            } else {
                                let ago_ms = now_ms - exp_ms;
                                let mins = ago_ms / 60_000;
                                if mins >= 60 {
                                    format!("expired {} hour(s) ago", mins / 60)
                                } else {
                                    format!("expired {mins} minute(s) ago")
                                }
                            }
                        } else {
                            "unknown expiry".to_string()
                        }
                    } else {
                        "unknown expiry".to_string()
                    }
                } else {
                    "unknown expiry".to_string()
                }
            } else {
                "unknown expiry".to_string()
            };

            serde_json::json!({
                "authenticated": true,
                "email": ctx.email,
                "workspace_id": ctx.workspace_id,
                "user_id": ctx.user_id,
                "token_status": token_status
            })
            .to_string()
        }
        Err(_) => serde_json::json!({
            "authenticated": false,
            "message": auth::NOT_AUTHENTICATED_MSG
        })
        .to_string(),
    }
}

/// Implementation for the rinda_auth_login tool.
/// Returns instructions for authenticating via the MCP OAuth 2.0 flow.
/// Authentication is handled automatically by the MCP client (e.g. Claude Desktop)
/// through the server's OAuth endpoints.
pub async fn auth_login() -> String {
    let server_url =
        std::env::var("MCP_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    serde_json::json!({
        "auth_url": format!("{}/oauth/authorize", server_url),
        "instructions": "Authentication is handled automatically by your MCP client via OAuth 2.0. If you are not authenticated, your client should redirect you to sign in with Google. You can also check your auth status using the rinda_auth_status tool."
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
            url.contains("/oauth/authorize"),
            "auth URL should contain '/oauth/authorize'"
        );

        let instructions = parsed["instructions"]
            .as_str()
            .expect("should have instructions");
        assert!(!instructions.is_empty(), "instructions should not be empty");
    }

    /// Acceptance criteria: rinda_auth_status should return authenticated=false with
    /// a clear message when no parts/token are provided.
    #[tokio::test]
    async fn auth_status_returns_not_authenticated_when_no_parts() {
        let result = auth_status(None).await;
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

    /// Acceptance criteria: rinda_auth_status should return authenticated=true when
    /// a valid Bearer token is present in the request parts.
    #[tokio::test]
    async fn auth_status_returns_authenticated_when_valid_bearer_token() {
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"none"}"#);
        let payload = serde_json::json!({
            "workspaceId": "ws-test",
            "userId": "user-test",
            "email": "test@example.com",
            "exp": 9999999999i64
        });
        let payload_enc = engine.encode(payload.to_string());
        let token = format!("{}.{}.sig", header, payload_enc);

        let (parts, _) = http::Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap()
            .into_parts();

        let result = auth_status(Some(&parts)).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");

        assert_eq!(
            parsed["authenticated"].as_bool(),
            Some(true),
            "should report authenticated"
        );
        assert_eq!(parsed["email"].as_str(), Some("test@example.com"));
        assert_eq!(parsed["workspace_id"].as_str(), Some("ws-test"));
    }

    /// Auth login response should be parseable as JSON regardless of environment.
    #[tokio::test]
    async fn auth_login_response_is_valid_json() {
        let result = auth_login().await;
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
        assert!(parsed.is_ok(), "auth_login should always return valid JSON");
    }

    /// Acceptance criteria from issue #82: auth_status should work from Bearer token
    /// without reading ~/.rinda/credentials.json.
    #[tokio::test]
    async fn auth_status_does_not_require_credentials_file() {
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"none"}"#);
        let payload = engine
            .encode(r#"{"workspaceId":"ws","userId":"u","email":"a@b.com","exp":9999999999}"#);
        let token = format!("{}.{}.sig", header, payload);

        let (parts, _) = http::Request::builder()
            .header("Authorization", format!("Bearer {token}"))
            .body(())
            .unwrap()
            .into_parts();

        // auth_status should succeed without touching the filesystem
        let result = auth_status(Some(&parts)).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["authenticated"].as_bool(), Some(true));
    }
}
