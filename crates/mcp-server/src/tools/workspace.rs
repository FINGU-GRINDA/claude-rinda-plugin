// Workspace tool implementations.

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// List workspaces the authenticated user belongs to.
pub async fn workspace_list(auth: &AuthContext) -> String {
    let client = sdk_client(Some(&auth.access_token));
    match client.get_api_v1_workspaces_user().await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("workspace list failed: {e}") }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_auth(token: &str) -> AuthContext {
        AuthContext {
            access_token: token.to_string(),
            workspace_id: "ws-test".to_string(),
            user_id: "user-test".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    /// Acceptance criteria: rinda_workspace_list should call GET /api/v1/workspaces/user
    /// and always return valid JSON — either a response object or an error object.
    #[tokio::test]
    async fn workspace_list_always_returns_valid_json() {
        let auth = make_auth("invalid.token.value");
        let result = workspace_list(&auth).await;

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("workspace_list should always return valid JSON");

        assert!(
            parsed.is_object(),
            "workspace_list should return a JSON object, got: {result}"
        );
    }

    /// Acceptance criteria: when the API call fails, the error response must
    /// contain an "error" key with a "workspace list failed:" prefix.
    #[tokio::test]
    async fn workspace_list_error_message_has_correct_prefix() {
        let auth = make_auth("sometoken");
        let result = workspace_list(&auth).await;

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should be valid JSON");

        if let Some(err_msg) = parsed.get("error").and_then(|v| v.as_str()) {
            assert!(
                err_msg.starts_with("workspace list failed:"),
                "error message should start with 'workspace list failed:', got: {err_msg}"
            );
        }
    }
}
