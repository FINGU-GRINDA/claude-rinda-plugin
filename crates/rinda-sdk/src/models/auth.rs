use serde::{Deserialize, Serialize};

/// Request body for email/password login.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response from a successful login.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LoginResponse {
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<serde_json::Value>,
}

/// Request body for token refresh.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Response from a token refresh.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RefreshResponse {
    pub token: Option<String>,
    pub refresh_token: Option<String>,
}

/// Request body for Google OAuth callback (code exchange).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCallbackRequest {
    pub code: String,
}

/// Response from Google OAuth callback.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleCallbackResponse {
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<serde_json::Value>,
}

/// Minimal user profile as returned by /auth/me.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UserProfile {
    pub id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub workspace_id: Option<String>,
}

/// Request body for user signup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
