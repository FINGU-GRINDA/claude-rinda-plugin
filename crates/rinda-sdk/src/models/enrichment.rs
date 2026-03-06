use serde::{Deserialize, Serialize};

/// Request body for enriching leads with contact info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichLeadsRequest {
    pub lead_ids: Vec<String>,
}

/// Response from lead enrichment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EnrichLeadsResponse {
    pub results: Option<Vec<serde_json::Value>>,
    pub total: Option<i64>,
    pub enriched: Option<i64>,
    pub failed: Option<i64>,
}

/// Request body for applying enrichment results.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResultsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

/// Response for email status check.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailStatusResponse {
    pub lead_ids: Option<Vec<String>>,
    pub has_email: Option<Vec<String>>,
    pub no_email: Option<Vec<String>>,
}

/// Response for leads without email query.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LeadsWithoutEmailResponse {
    pub leads: Option<Vec<serde_json::Value>>,
    pub total: Option<i64>,
}
