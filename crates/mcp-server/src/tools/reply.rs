// Reply tool implementations: rinda_reply_check.

use crate::auth::{get_authenticated_client, json_to_text};

/// Get recent email replies.
pub async fn reply_check(limit: Option<u32>) -> String {
    let (client, _creds) = match get_authenticated_client().await {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({ "error": e }).to_string();
        }
    };

    let limit_val = limit.unwrap_or(50);
    let limit_str = limit_val.to_string();

    match client
        .get_api_v1_email_replies(
            None,             // email_account_id
            None,             // is_read
            Some(&limit_str), // limit
            None,             // offset
            None,             // search
            None,             // sentiment
            None,             // workspace_id
        )
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("reply check failed: {e}") }).to_string(),
    }
}
