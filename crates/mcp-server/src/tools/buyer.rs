// Buyer (lead discovery) tool implementations.

use uuid::Uuid;

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// Build the natural-language query string from filter params.
fn build_search_query(
    industry: Option<String>,
    countries: Option<String>,
    buyer_type: Option<String>,
    min_revenue: Option<f64>,
    limit: Option<u32>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(ind) = industry {
        parts.push(ind);
    }
    if let Some(c) = countries {
        parts.push(format!("countries:{c}"));
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
    if parts.is_empty() {
        "buyer search".to_string()
    } else {
        parts.join(" ")
    }
}

/// Start an async buyer search via SSE endpoint. Returns session_id and initial status.
///
/// The `/lead-discovery/search` endpoint is SSE-streaming. We POST with
/// `Accept: text/event-stream`, read events until we get a session_id or
/// terminal state, then return the collected info as JSON.
pub async fn buyer_search(
    auth: &AuthContext,
    industry: Option<String>,
    countries: Option<String>,
    buyer_type: Option<String>,
    min_revenue: Option<f64>,
    limit: Option<u32>,
) -> String {
    let workspace_id = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in token. Please re-authenticate."
            })
            .to_string();
        }
    };

    let query = build_search_query(industry, countries, buyer_type, min_revenue, limit);

    let body = serde_json::json!({
        "query": query,
        "workspaceId": workspace_id,
        "useAutoTimeout": true,
    });

    match sse_search_request(&auth.access_token, &body).await {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::json!({ "error": format!("buyer search failed: {e}") }).to_string(),
    }
}

/// Perform the SSE POST request to `/lead-discovery/search` and collect events
/// until a session_id is obtained or the stream ends / times out.
async fn sse_search_request(
    access_token: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use futures_util::StreamExt;

    let base = rinda_common::config::base_url();
    let url = format!("{base}/api/v1/lead-discovery/search");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Accept", "text/event-stream")
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {text}"));
    }

    // Read SSE events from the response stream.
    let mut stream = resp.bytes_stream();
    let mut session_id: Option<String> = None;
    let mut last_status: Option<String> = None;
    let mut last_data: Option<serde_json::Value> = None;
    let mut buf = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream read error: {e}"))?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete SSE events (delimited by double newline).
        while let Some(pos) = buf.find("\n\n") {
            let event_block = buf[..pos].to_string();
            buf = buf[pos + 2..].to_string();

            for line in event_block.lines() {
                if let Some(data_str) = line.strip_prefix("data: ").or_else(|| line.strip_prefix("data:")) {
                    let data_str = data_str.trim();
                    if data_str == "[DONE]" {
                        // Stream finished.
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data_str) {
                        // Extract session_id from various possible field names.
                        if session_id.is_none() {
                            session_id = parsed
                                .get("sessionId")
                                .or_else(|| parsed.get("session_id"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        if let Some(s) = parsed.get("status").and_then(|v| v.as_str()) {
                            last_status = Some(s.to_string());
                        }
                        last_data = Some(parsed);
                    }
                }
            }
        }

        // Once we have a session_id, we can return early.
        // The caller can poll for progress with buyer_status.
        if session_id.is_some() && last_status.is_some() {
            break;
        }
    }

    // Build the result.
    let mut result = serde_json::json!({});
    if let Some(sid) = &session_id {
        result["sessionId"] = serde_json::json!(sid);
    }
    if let Some(st) = &last_status {
        result["status"] = serde_json::json!(st);
    }
    if let Some(data) = last_data {
        result["lastEvent"] = data;
    }

    if session_id.is_none() {
        result["warning"] = serde_json::json!(
            "No session_id received from SSE stream. The search may not have started."
        );
    } else {
        result["hint"] = serde_json::json!(
            "Use buyer_status with the sessionId to poll for progress, then buyer_results to get the final results."
        );
    }

    Ok(result)
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
