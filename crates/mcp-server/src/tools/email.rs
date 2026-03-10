// Email tool implementations: rinda_email_send.

use uuid::Uuid;

use crate::auth::{get_authenticated_client, json_to_text};

/// Send an email via the RINDA API.
pub async fn email_send(to: String, subject: String, body: String) -> String {
    let (client, creds) = match get_authenticated_client().await {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({ "error": e }).to_string();
        }
    };

    let user_id = match creds.user_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid user ID in credentials. Please log in again."
            })
            .to_string();
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

    let subject_typed: rinda_sdk::types::PostApiV1EmailsSendBodySubject = match subject.parse() {
        Ok(s) => s,
        Err(e) => {
            return serde_json::json!({ "error": format!("Invalid subject: {e}") }).to_string();
        }
    };

    let req_body = rinda_sdk::types::PostApiV1EmailsSendBody {
        to_email: to,
        subject: subject_typed,
        body_text: Some(body),
        body_html: None,
        user_id,
        workspace_id,
        bcc_emails: Vec::new(),
        cc_emails: Vec::new(),
        files: Vec::new(),
        from_name: None,
        in_reply_to: None,
        include_signature: None,
        lead_id: None,
        references: None,
        reply_to: None,
        scheduled_at: None,
        sequence_id: None,
        step_id: None,
    };

    match client.post_api_v1_emails_send(&req_body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("email send failed: {e}") }).to_string(),
    }
}
