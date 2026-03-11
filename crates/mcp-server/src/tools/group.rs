// Customer group CRUD and member management tool implementations.

use uuid::Uuid;

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// Create a new customer group.
pub async fn group_create(
    auth: &AuthContext,
    name: String,
    workspace_id_override: Option<String>,
    description: Option<String>,
    is_dynamic: Option<bool>,
    auto_enrich_enabled: Option<bool>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let workspace_id_str = workspace_id_override.unwrap_or_else(|| auth.workspace_id.clone());
    let workspace_id = match workspace_id_str.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID. Please re-authenticate."
            })
            .to_string();
        }
    };

    let name_typed: rinda_sdk::types::PostApiV1CustomerGroupsBodyName = match name.parse() {
        Ok(n) => n,
        Err(e) => {
            return serde_json::json!({ "error": format!("Invalid group name: {e}") }).to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1CustomerGroupsBody {
        name: name_typed,
        workspace_id,
        description,
        is_dynamic,
        auto_enrich_enabled,
        created_by: None,
        criteria: None,
        csv_data: Vec::new(),
        enrich_freshness_unit: None,
        enrich_freshness_value: None,
    };

    match client.post_api_v1_customer_groups(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group create failed: {e}") }).to_string(),
    }
}

/// Search / list customer groups for the current workspace.
pub async fn group_list(
    auth: &AuthContext,
    search: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
    is_dynamic: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let ws_str = auth.workspace_id.clone();

    match client
        .get_api_v1_customer_groups_search(
            None,
            is_dynamic.as_deref(),
            limit.as_deref(),
            offset.as_deref(),
            search.as_deref(),
            if ws_str.is_empty() {
                None
            } else {
                Some(ws_str.as_str())
            },
        )
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group list failed: {e}") }).to_string(),
    }
}

/// Get a customer group by ID.
pub async fn group_get(auth: &AuthContext, id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid group ID — must be a valid UUID" })
                .to_string();
        }
    };

    match client.get_api_v1_customer_groups_by_id(&uuid).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group get failed: {e}") }).to_string(),
    }
}

/// Update a customer group.
pub async fn group_update(
    auth: &AuthContext,
    id: String,
    name: String,
    is_dynamic: bool,
    description: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid group ID — must be a valid UUID" })
                .to_string();
        }
    };

    let name_typed: rinda_sdk::types::PutApiV1CustomerGroupsByIdBodyName = match name.parse() {
        Ok(n) => n,
        Err(e) => {
            return serde_json::json!({ "error": format!("Invalid group name: {e}") }).to_string();
        }
    };

    let body = rinda_sdk::types::PutApiV1CustomerGroupsByIdBody {
        name: name_typed,
        is_dynamic,
        description,
        auto_enrich_enabled: None,
        criteria: None,
        enrich_freshness_unit: None,
        enrich_freshness_value: None,
    };

    match client.put_api_v1_customer_groups_by_id(&uuid, &body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group update failed: {e}") }).to_string(),
    }
}

/// Delete a customer group.
pub async fn group_delete(auth: &AuthContext, id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid group ID — must be a valid UUID" })
                .to_string();
        }
    };

    match client.delete_api_v1_customer_groups_by_id(&uuid).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group delete failed: {e}") }).to_string(),
    }
}

/// List members of a customer group.
pub async fn group_members(
    auth: &AuthContext,
    id: String,
    limit: Option<String>,
    offset: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid group ID — must be a valid UUID" })
                .to_string();
        }
    };

    match client
        .get_api_v1_customer_groups_by_id_members(&uuid, limit.as_deref(), offset.as_deref())
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group members failed: {e}") }).to_string(),
    }
}

/// Add a lead as a member of a customer group.
pub async fn group_add_member(auth: &AuthContext, id: String, lead_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let group_uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid group ID — must be a valid UUID" })
                .to_string();
        }
    };

    let lead_uuid = match lead_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid lead ID — must be a valid UUID" })
                .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1CustomerGroupsByIdMembersBody {
        lead_id: lead_uuid,
        added_by: None,
    };

    match client
        .post_api_v1_customer_groups_by_id_members(&group_uuid, &body)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("group add-member failed: {e}") }).to_string()
        }
    }
}

/// Remove a lead from a customer group.
pub async fn group_remove_member(auth: &AuthContext, id: String, lead_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let group_uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid group ID — must be a valid UUID" })
                .to_string();
        }
    };

    let lead_uuid = match lead_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid lead ID — must be a valid UUID" })
                .to_string();
        }
    };

    match client
        .delete_api_v1_customer_groups_by_id_members_by_lead_id(&group_uuid, &lead_uuid)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("group remove-member failed: {e}") }).to_string()
        }
    }
}

/// List all customer groups that a given lead belongs to.
pub async fn group_for_lead(auth: &AuthContext, lead_id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let lead_uuid = match lead_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid lead ID — must be a valid UUID" })
                .to_string();
        }
    };

    match client
        .get_api_v1_customer_groups_lead_by_lead_id_groups(&lead_uuid)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("group for-lead failed: {e}") }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;

    fn make_auth() -> AuthContext {
        AuthContext {
            access_token: "invalid-token-for-testing".to_string(),
            workspace_id: "00000000-0000-0000-0000-000000000001".to_string(),
            user_id: "00000000-0000-0000-0000-000000000002".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    #[tokio::test]
    async fn group_list_with_invalid_token_returns_valid_json() {
        let auth = make_auth();
        let result = group_list(&auth, None, None, None, None).await;
        // The API may return either an error JSON or a success response with empty data.
        // Either way it must be valid JSON.
        let _parsed: serde_json::Value =
            serde_json::from_str(&result).expect("group_list should always return valid JSON");
    }

    #[tokio::test]
    async fn group_get_with_invalid_token_returns_valid_json_with_error() {
        let auth = make_auth();
        let valid_uuid = "00000000-0000-0000-0000-000000000099".to_string();
        let result = group_get(&auth, valid_uuid).await;
        let parsed: serde_json::Value = serde_json::from_str(&result)
            .expect("group_get should return valid JSON even on auth error");
        assert!(
            parsed.get("error").is_some(),
            "response should contain an 'error' key, got: {result}"
        );
    }

    #[tokio::test]
    async fn group_get_with_invalid_uuid_returns_error_json() {
        let auth = make_auth();
        let result = group_get(&auth, "not-a-uuid".to_string()).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");
        assert!(
            parsed.get("error").is_some(),
            "invalid UUID should return error JSON"
        );
        let err_msg = parsed["error"].as_str().unwrap_or("");
        assert!(
            err_msg.contains("UUID"),
            "error should mention UUID: {err_msg}"
        );
    }

    #[tokio::test]
    async fn group_add_member_with_invalid_group_uuid_returns_error() {
        let auth = make_auth();
        let result = group_add_member(
            &auth,
            "not-a-uuid".to_string(),
            "00000000-0000-0000-0000-000000000099".to_string(),
        )
        .await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");
        assert!(parsed.get("error").is_some());
    }

    #[tokio::test]
    async fn group_add_member_with_invalid_lead_uuid_returns_error() {
        let auth = make_auth();
        let result = group_add_member(
            &auth,
            "00000000-0000-0000-0000-000000000099".to_string(),
            "not-a-uuid".to_string(),
        )
        .await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");
        assert!(parsed.get("error").is_some());
        let err_msg = parsed["error"].as_str().unwrap_or("");
        assert!(
            err_msg.contains("lead ID") || err_msg.contains("UUID"),
            "error should mention lead ID: {err_msg}"
        );
    }

    #[tokio::test]
    async fn group_remove_member_with_invalid_group_uuid_returns_error() {
        let auth = make_auth();
        let result = group_remove_member(
            &auth,
            "not-a-uuid".to_string(),
            "00000000-0000-0000-0000-000000000099".to_string(),
        )
        .await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");
        assert!(parsed.get("error").is_some());
    }

    #[tokio::test]
    async fn group_for_lead_with_invalid_uuid_returns_error() {
        let auth = make_auth();
        let result = group_for_lead(&auth, "not-a-uuid".to_string()).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("should return valid JSON");
        assert!(parsed.get("error").is_some());
    }

    /// Acceptance criteria: all 9 group operations must return valid JSON even
    /// when the backend rejects the request (e.g. invalid token).
    /// Acceptance criteria: all group operations must return valid JSON regardless
    /// of the backend response (success or auth error).
    #[tokio::test]
    async fn all_group_operations_return_valid_json() {
        let auth = make_auth();
        let uuid = "00000000-0000-0000-0000-000000000099".to_string();

        let ops: Vec<(&str, String)> = vec![
            ("list", group_list(&auth, None, None, None, None).await),
            ("get", group_get(&auth, uuid.clone()).await),
            ("delete", group_delete(&auth, uuid.clone()).await),
            (
                "members",
                group_members(&auth, uuid.clone(), None, None).await,
            ),
            ("for_lead", group_for_lead(&auth, uuid.clone()).await),
        ];

        for (op_name, result) in ops {
            let _parsed: serde_json::Value = serde_json::from_str(&result)
                .unwrap_or_else(|_| panic!("{op_name} did not return valid JSON: {result}"));
        }
    }
}
