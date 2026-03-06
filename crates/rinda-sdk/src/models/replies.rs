use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single email reply record.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailReply {
    pub id: Option<String>,
    pub from: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub intent: Option<String>,
    pub sentiment: Option<String>,
    pub is_read: Option<bool>,
    pub thread_id: Option<String>,
    pub sequence_id: Option<String>,
    pub lead_id: Option<String>,
    pub received_at: Option<String>,
}

/// Query parameters for listing email replies.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRepliesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_account_id: Option<String>,
}

/// Stats grouped by reply intent.
/// Keys are intent labels (e.g. "meeting_request"), values are counts.
pub type ReplyStatsByIntent = HashMap<String, i64>;

/// Request body for bulk marking replies as read/unread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkReadRequest {
    pub ids: Vec<String>,
}
