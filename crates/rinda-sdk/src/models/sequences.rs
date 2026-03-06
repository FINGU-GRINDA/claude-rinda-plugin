use serde::{Deserialize, Serialize};

/// Request body for creating a sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSequenceRequest {
    pub name: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_group_id: Option<String>,
}

/// Response from sequence create/get.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SequenceResponse {
    pub id: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub workspace_id: Option<String>,
    pub customer_group_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Request body for AI-generating sequence steps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSequenceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Response containing AI-generated steps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GenerateSequenceResponse {
    pub steps: Option<Vec<serde_json::Value>>,
}

/// A single step in an email sequence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SequenceStep {
    pub id: Option<String>,
    pub sequence_id: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub delay_days: Option<i32>,
    pub order: Option<i32>,
}

/// Request body for creating a step.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateStepRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

/// Request body for updating a step.
pub type UpdateStepRequest = CreateStepRequest;

/// Request body for enrolling leads in a sequence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnrollmentRequest {
    pub lead_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email_account_id: Option<String>,
}

/// Response from bulk enrollment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EnrollmentResponse {
    pub enrolled: Option<i64>,
    pub skipped: Option<i64>,
    pub failed: Option<i64>,
}

/// Per-sequence metrics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SequenceMetrics {
    pub open_rate: Option<f64>,
    pub click_rate: Option<f64>,
    pub reply_rate: Option<f64>,
    pub by_step: Option<Vec<serde_json::Value>>,
}

/// Aggregate stats across all sequences.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverallStats {
    pub total_sent: Option<i64>,
    pub opened: Option<i64>,
    pub clicked: Option<i64>,
    pub replied: Option<i64>,
    pub bounced: Option<i64>,
}

/// Pagination parameters for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}
