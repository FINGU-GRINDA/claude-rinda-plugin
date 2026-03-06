use crate::client::RindaClient;
use crate::error::Result;
use crate::models::auth::{
    GoogleCallbackRequest, GoogleCallbackResponse, LoginRequest, LoginResponse, RefreshRequest,
    RefreshResponse, UserProfile,
};

impl RindaClient {
    /// POST /api/v1/auth/login — email/password login.
    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse> {
        let resp = self.post("/api/v1/auth/login").json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/auth/me — fetch authenticated user profile.
    pub async fn me(&self) -> Result<UserProfile> {
        let resp = self.get("/api/v1/auth/me").send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/auth/signup — register a new user.
    pub async fn signup(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self.post("/api/v1/auth/signup").json(&body).send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/auth/refresh — refresh the access token.
    pub async fn refresh(&self, req: &RefreshRequest) -> Result<RefreshResponse> {
        let resp = self.post("/api/v1/auth/refresh").json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// Returns the URL to redirect the user to for Google OAuth.
    pub fn google_auth_url(&self) -> String {
        format!("{}/api/v1/auth/google", self.base_url())
    }

    /// POST /api/v1/auth/google/callback — exchange OAuth code for tokens.
    pub async fn google_callback(
        &self,
        req: &GoogleCallbackRequest,
    ) -> Result<GoogleCallbackResponse> {
        let resp = self
            .post("/api/v1/auth/google/callback")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/auth/google/verify — verify a Google ID token.
    pub async fn google_verify(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/auth/google/verify")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/auth/admin-check — check admin status.
    pub async fn admin_check(&self) -> Result<serde_json::Value> {
        let resp = self.get("/api/v1/auth/admin-check").send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/auth/profile — update user profile.
    pub async fn update_profile(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self.put("/api/v1/auth/profile").json(&body).send().await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/auth/account — delete user account.
    pub async fn delete_account(&self) -> Result<serde_json::Value> {
        let resp = self.delete("/api/v1/auth/account").send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/auth/heartbeat — keep-alive / auth check.
    pub async fn heartbeat(&self) -> Result<serde_json::Value> {
        let resp = self.get("/api/v1/auth/heartbeat").send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/auth/register-email — register email address.
    pub async fn register_email(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/auth/register-email")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/auth/verify — verify account.
    pub async fn verify_account(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self.post("/api/v1/auth/verify").json(&body).send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/auth/verify-license — verify a license key.
    pub async fn verify_license(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/auth/verify-license")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }
}
