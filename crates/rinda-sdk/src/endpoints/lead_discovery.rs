use crate::client::RindaClient;
use crate::error::Result;
use crate::models::leads::{
    LeadClarifyRequest, LeadFeedbackRequest, LeadScoreRequest, LeadSearchRequest,
    LeadSearchResponse, LeadSelectRequest, SessionResults, SessionStatus,
};

impl RindaClient {
    /// POST /api/v1/lead-discovery/search — start a new lead discovery session.
    pub async fn lead_search(&self, req: &LeadSearchRequest) -> Result<LeadSearchResponse> {
        let resp = self
            .post("/api/v1/lead-discovery/search")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/lead-discovery/select — select leads from a session.
    pub async fn lead_select(&self, req: &LeadSelectRequest) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/select")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/lead-discovery/clarify — answer clarification questions.
    pub async fn lead_clarify(&self, req: &LeadClarifyRequest) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/clarify")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/lead-discovery/cancel — cancel an active session.
    pub async fn lead_cancel(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/cancel")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/session/{sessionId} — get session info.
    pub async fn get_session(&self, session_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/lead-discovery/session/{}", session_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/session/{sessionId}/status — poll session status.
    pub async fn get_session_status(&self, session_id: &str) -> Result<SessionStatus> {
        let path = format!("/api/v1/lead-discovery/session/{}/status", session_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/lead-discovery/db/sessions/{sessionId} — delete a DB session.
    pub async fn delete_db_session(&self, session_id: &str) -> Result<()> {
        let path = format!("/api/v1/lead-discovery/db/sessions/{}", session_id);
        let resp = self.delete(&path).send().await?;
        Self::handle_response_empty(resp).await
    }

    /// POST /api/v1/lead-discovery/more — get more results for a session.
    pub async fn lead_more(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/more")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/lead-discovery/sessions — clear all sessions.
    pub async fn clear_sessions(&self) -> Result<()> {
        let resp = self
            .delete("/api/v1/lead-discovery/sessions")
            .send()
            .await?;
        Self::handle_response_empty(resp).await
    }

    /// POST /api/v1/lead-discovery/score — score a list of leads.
    pub async fn lead_score(&self, req: &LeadScoreRequest) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/score")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/lead-discovery/enrich — enrich leads from discovery.
    pub async fn lead_discovery_enrich(
        &self,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/enrich")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/usage/{workspaceId} — get usage stats.
    pub async fn lead_discovery_usage(&self, workspace_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/lead-discovery/usage/{}", workspace_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/lead-discovery/feedback — submit feedback.
    pub async fn submit_feedback(&self, req: &LeadFeedbackRequest) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/lead-discovery/feedback")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/feedback/{sessionId} — get feedback for a session.
    pub async fn get_feedback(&self, session_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/lead-discovery/feedback/{}", session_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/db/sessions — list all DB sessions.
    pub async fn list_db_sessions(&self) -> Result<serde_json::Value> {
        let resp = self
            .get("/api/v1/lead-discovery/db/sessions")
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/db/sessions/{sessionId} — get a specific DB session.
    pub async fn get_db_session(&self, session_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/lead-discovery/db/sessions/{}", session_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PATCH /api/v1/lead-discovery/db/sessions/{sessionId} — update a DB session.
    pub async fn update_db_session(
        &self,
        session_id: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v1/lead-discovery/db/sessions/{}", session_id);
        let resp = self.patch(&path).json(&body).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/db/sessions/{sessionId}/results — get session results.
    pub async fn get_session_results(&self, session_id: &str) -> Result<SessionResults> {
        let path = format!("/api/v1/lead-discovery/db/sessions/{}/results", session_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/db/sessions/{sessionId}/messages — get session messages.
    pub async fn get_session_messages(&self, session_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/lead-discovery/db/sessions/{}/messages", session_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/lead-discovery/health — check service health.
    pub async fn lead_discovery_health(&self) -> Result<serde_json::Value> {
        let resp = self.get("/api/v1/lead-discovery/health").send().await?;
        Self::handle_response(resp).await
    }
}
