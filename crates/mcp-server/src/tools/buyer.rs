// Buyer (lead discovery) tool implementations.

use uuid::Uuid;

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// Start an async buyer search. Returns session_id and initial status.
pub async fn buyer_search(
    auth: &AuthContext,
    industry: Option<String>,
    countries: Option<String>,
    buyer_type: Option<String>,
    min_revenue: Option<f64>,
    limit: Option<u32>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let workspace_id = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in token. Please re-authenticate."
            })
            .to_string();
        }
    };

    // Build the query from available filters.
    let mut parts: Vec<String> = Vec::new();
    if let Some(ind) = industry {
        parts.push(ind);
    }
    if let Some(countries) = countries {
        parts.push(format!("countries:{countries}"));
    }
    if let Some(bt) = buyer_type {
        parts.push(format!("type:{bt}"));
    }
    if let Some(rev) = min_revenue {
        parts.push(format!("min_revenue:{rev}"));
    }
    if let Some(lim) = limit {
        parts.push(format!("limit:{lim}"));
    }
    let query = if parts.is_empty() {
        "buyer search".to_string()
    } else {
        parts.join(" ")
    };

    let body = rinda_sdk::types::PostApiV1LeadDiscoverySearchBody {
        query,
        workspace_id,
        crawl_timeout_seconds: None,
        locale: None,
        session_id: None,
        use_auto_timeout: true,
    };

    match client.post_api_v1_lead_discovery_search(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer search failed: {e}") }).to_string(),
    }
}

/// Poll the status of an async buyer search session.
pub async fn buyer_status(auth: &AuthContext, session_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match session_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid session_id — must be a valid UUID" })
                .to_string();
        }
    };

    match client
        .get_api_v1_lead_discovery_db_sessions_by_session_id(&uuid)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer status failed: {e}") }).to_string(),
    }
}

/// Get the results of a completed buyer search session.
pub async fn buyer_results(auth: &AuthContext, session_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match session_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid session_id — must be a valid UUID" })
                .to_string();
        }
    };

    match client
        .get_api_v1_lead_discovery_db_sessions_by_session_id_results(&uuid)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer results failed: {e}") }).to_string(),
    }
}

/// Save selected leads from a discovery session.
pub async fn buyer_select(
    auth: &AuthContext,
    session_id: String,
    recommendation_id: String,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let workspace_id = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in token. Please re-authenticate."
            })
            .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1LeadDiscoverySelectBody {
        session_id,
        selected_recommendation_id: recommendation_id,
        workspace_id,
    };

    match client.post_api_v1_lead_discovery_select(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer select failed: {e}") }).to_string(),
    }
}

/// Enrich a buyer/lead with additional data.
pub async fn buyer_enrich(auth: &AuthContext, buyer_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let body = rinda_sdk::types::PostApiV1LeadDiscoveryEnrichBody {
        website_url: buyer_id,
        workspace_id: auth.workspace_id.clone(),
    };

    match client.post_api_v1_lead_discovery_enrich(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer enrich failed: {e}") }).to_string(),
    }
}

/// Retrieve clarification questions for a session in waiting_clarification status.
pub async fn buyer_messages(auth: &AuthContext, session_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match session_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid session_id — must be a valid UUID" })
                .to_string();
        }
    };

    match client
        .get_api_v1_lead_discovery_db_sessions_by_session_id_messages(&uuid)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer messages failed: {e}") }).to_string(),
    }
}

/// List all past search sessions for a workspace.
pub async fn buyer_sessions(auth: &AuthContext, user_id: Option<String>) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let workspace_uuid = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in token. Please re-authenticate."
            })
            .to_string();
        }
    };

    let user_uuid = match user_id {
        Some(id) => match id.parse::<Uuid>() {
            Ok(u) => Some(u),
            Err(_) => {
                return serde_json::json!({ "error": "Invalid user_id — must be a valid UUID" })
                    .to_string();
            }
        },
        None => None,
    };

    match client
        .get_api_v1_lead_discovery_db_sessions(user_uuid.as_ref(), &workspace_uuid)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer sessions failed: {e}") }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_auth(workspace_id: &str) -> AuthContext {
        AuthContext {
            access_token: "invalid.token.value".to_string(),
            workspace_id: workspace_id.to_string(),
            user_id: "user-test".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    /// Acceptance criteria: buyer_sessions should always return valid JSON,
    /// either the sessions list or an error object.
    #[tokio::test]
    async fn buyer_sessions_always_returns_valid_json() {
        let auth = make_auth("00000000-0000-0000-0000-000000000001");
        let result = buyer_sessions(&auth, None).await;

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("buyer_sessions should always return valid JSON");

        assert!(
            parsed.is_object(),
            "buyer_sessions should return a JSON object, got: {result}"
        );
    }

    /// When an invalid workspace_id is in the auth context, the function returns
    /// an error JSON with a descriptive message about re-authentication.
    #[tokio::test]
    async fn buyer_sessions_invalid_workspace_id_returns_error() {
        let auth = make_auth("not-a-uuid");
        let result = buyer_sessions(&auth, None).await;

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should be valid JSON");

        let error = parsed
            .get("error")
            .and_then(|v| v.as_str())
            .expect("error key must be present");
        assert!(
            error.contains("Invalid workspace ID"),
            "error should mention 'Invalid workspace ID', got: {error}"
        );
    }

    /// When an invalid user_id is provided, the function returns an error JSON
    /// explaining the UUID requirement — not a panic.
    #[tokio::test]
    async fn buyer_sessions_invalid_user_id_returns_error() {
        let auth = make_auth("00000000-0000-0000-0000-000000000001");
        let result = buyer_sessions(&auth, Some("not-a-uuid".to_string())).await;

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should be valid JSON");

        let error = parsed
            .get("error")
            .and_then(|v| v.as_str())
            .expect("error key must be present");
        assert!(
            error.contains("Invalid user_id"),
            "error should mention 'Invalid user_id', got: {error}"
        );
    }

    /// When a valid user_id UUID is provided (but API call fails in CI),
    /// the function should still return valid JSON with an error key.
    #[tokio::test]
    async fn buyer_sessions_with_valid_user_id_returns_valid_json() {
        let auth = make_auth("00000000-0000-0000-0000-000000000001");
        let result = buyer_sessions(
            &auth,
            Some("00000000-0000-0000-0000-000000000002".to_string()),
        )
        .await;

        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should be valid JSON");

        assert!(
            parsed.is_object(),
            "buyer_sessions should return a JSON object, got: {result}"
        );
    }
}

/// Submit clarification answers for a search session in waiting_clarification status.
pub async fn buyer_clarify(auth: &AuthContext, session_id: String, answers: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let workspace_id = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in token. Please re-authenticate."
            })
            .to_string();
        }
    };

    let answers_map: serde_json::Map<String, serde_json::Value> =
        match serde_json::from_str(&answers) {
            Ok(serde_json::Value::Object(m)) => m,
            Ok(_) => {
                return serde_json::json!({
                    "error": "answers must be a JSON object (e.g. {\"field\": \"value\"})"
                })
                .to_string();
            }
            Err(e) => {
                return serde_json::json!({
                    "error": format!("Invalid answers JSON: {e}")
                })
                .to_string();
            }
        };

    let body = rinda_sdk::types::PostApiV1LeadDiscoveryClarifyBody {
        session_id,
        answers: answers_map,
        workspace_id,
    };

    match client.post_api_v1_lead_discovery_clarify(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("buyer clarify failed: {e}") }).to_string(),
    }
}
