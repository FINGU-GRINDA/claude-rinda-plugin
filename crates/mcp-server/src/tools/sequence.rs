// Sequence tool implementations.

use uuid::Uuid;

use crate::auth::{get_authenticated_client, json_to_text};

/// Create a new email sequence.
pub async fn sequence_create(
    name: String,
    seq_type: Option<String>,
    steps: Option<String>,
) -> String {
    let _ = (seq_type, steps); // currently unused — SDK doesn't expose these directly

    let (client, creds) = match get_authenticated_client().await {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({ "error": e }).to_string();
        }
    };

    let workspace_id = match creds.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in credentials. Please log in again."
            })
            .to_string();
        }
    };

    let name_typed: rinda_sdk::types::PostApiV1SequencesBodyName = match name.parse() {
        Ok(n) => n,
        Err(e) => {
            return serde_json::json!({ "error": format!("Invalid sequence name: {e}") })
                .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1SequencesBody {
        name: name_typed,
        workspace_id,
        created_by: None,
        customer_group_id: None,
        customer_group_ids: Vec::new(),
        description: None,
        memo: None,
        personalization_config: None,
        personalization_mode: None,
        status: None,
        timezone_mode: None,
        workflow_data: None,
    };

    match client.post_api_v1_sequences(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("sequence create failed: {e}") }).to_string()
        }
    }
}

/// List existing sequences for the workspace.
pub async fn sequence_list(limit: Option<String>, offset: Option<String>) -> String {
    let (client, _creds) = match get_authenticated_client().await {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({ "error": e }).to_string();
        }
    };

    match client
        .get_api_v1_sequences(limit.as_deref(), offset.as_deref())
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("sequence list failed: {e}") }).to_string(),
    }
}

/// AI-generate email steps for a sequence.
pub async fn sequence_generate(id: String) -> String {
    let (client, _creds) = match get_authenticated_client().await {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({ "error": e }).to_string();
        }
    };

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid sequence id — must be a valid UUID" })
                .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1SequencesByIdGenerateBody {
        user_email_account_id: None,
    };

    match client
        .post_api_v1_sequences_by_id_generate(&uuid, &body)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("sequence generate failed: {e}") }).to_string()
        }
    }
}

/// Enroll a lead/buyer into a sequence.
pub async fn sequence_add_contact(sequence_id: String, buyer_id: String) -> String {
    let (client, _creds) = match get_authenticated_client().await {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({ "error": e }).to_string();
        }
    };

    let seq_uuid = match sequence_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid sequence_id — must be a valid UUID" })
                .to_string();
        }
    };

    let lead_uuid = match buyer_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid buyer_id — must be a valid UUID" })
                .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1SequencesByIdEnrollmentsBody {
        lead_id: lead_uuid,
        user_email_account_id: Uuid::nil(),
        enrolled_by: None,
        status: None,
    };

    match client
        .post_api_v1_sequences_by_id_enrollments(&seq_uuid, &body)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("sequence add-contact failed: {e}") }).to_string()
        }
    }
}
