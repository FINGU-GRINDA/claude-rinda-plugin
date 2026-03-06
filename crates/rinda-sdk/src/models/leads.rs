use serde::{Deserialize, Serialize};

/// Request body for lead discovery search.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LeadSearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_countries: Option<Vec<String>>,
    pub workspace_id: String,
}

/// Response from lead discovery search — contains a session ID.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LeadSearchResponse {
    pub session_id: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
}

/// Request body for selecting leads from a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadSelectRequest {
    pub session_id: String,
    pub selected_buyers: Vec<serde_json::Value>,
}

/// Request body for answering clarification questions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadClarifyRequest {
    pub session_id: String,
    pub answers: serde_json::Value,
}

/// Session status polling response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SessionStatus {
    pub status: Option<String>,
    pub progress: Option<f64>,
    pub message: Option<String>,
}

/// Final results for a completed session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SessionResults {
    pub leads: Option<Vec<serde_json::Value>>,
    pub total: Option<i64>,
}

/// Request body for scoring leads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadScoreRequest {
    pub leads: Vec<serde_json::Value>,
    pub workspace_id: String,
}

/// Request body for submitting feedback on a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadFeedbackRequest {
    pub session_id: String,
    pub feedback: serde_json::Value,
}
