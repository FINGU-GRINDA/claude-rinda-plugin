use serde::{Deserialize, Serialize};

/// Unified dashboard response (funnel, hot leads, activity, subscription).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UnifiedDashboard {
    pub funnel: Option<serde_json::Value>,
    pub hot_leads: Option<serde_json::Value>,
    pub activity: Option<serde_json::Value>,
    pub subscription: Option<serde_json::Value>,
}

/// Generic dashboard stats.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct DashboardStats {
    pub total_leads: Option<i64>,
    pub total_sequences: Option<i64>,
    pub total_emails_sent: Option<i64>,
    pub active_sequences: Option<i64>,
}

/// Trend data series (time-series values).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TrendData {
    pub data: Option<Vec<serde_json::Value>>,
}

/// Query parameters for dashboard endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DashboardParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}
