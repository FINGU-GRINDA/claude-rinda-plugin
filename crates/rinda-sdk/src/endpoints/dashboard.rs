use crate::client::RindaClient;
use crate::error::Result;
use crate::models::dashboard::{DashboardParams, DashboardStats, TrendData, UnifiedDashboard};

fn build_dashboard_query(params: &DashboardParams) -> Vec<(&'static str, String)> {
    let mut query: Vec<(&'static str, String)> = Vec::new();
    if let Some(ref wid) = params.workspace_id {
        query.push(("workspaceId", wid.clone()));
    }
    if let Some(ref sid) = params.sequence_id {
        query.push(("sequenceId", sid.clone()));
    }
    if let Some(ref start) = params.start_date {
        query.push(("startDate", start.clone()));
    }
    if let Some(ref end) = params.end_date {
        query.push(("endDate", end.clone()));
    }
    query
}

impl RindaClient {
    /// GET /api/v1/dashboard/unified — full unified dashboard data.
    pub async fn dashboard_unified(&self, params: &DashboardParams) -> Result<UnifiedDashboard> {
        let query = build_dashboard_query(params);
        let resp = self
            .get("/api/v1/dashboard/unified")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/stats — dashboard summary stats.
    pub async fn dashboard_stats(&self, params: &DashboardParams) -> Result<DashboardStats> {
        let query = build_dashboard_query(params);
        let resp = self
            .get("/api/v1/dashboard/stats")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/trends/leads — lead discovery trend data.
    pub async fn trends_leads(&self, params: &DashboardParams) -> Result<TrendData> {
        let query = build_dashboard_query(params);
        let resp = self
            .get("/api/v1/dashboard/trends/leads")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/trends/emails — email send trend data.
    pub async fn trends_emails(&self, params: &DashboardParams) -> Result<TrendData> {
        let query = build_dashboard_query(params);
        let resp = self
            .get("/api/v1/dashboard/trends/emails")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/trends/opens — email open trend data.
    pub async fn trends_opens(&self, params: &DashboardParams) -> Result<TrendData> {
        let query = build_dashboard_query(params);
        let resp = self
            .get("/api/v1/dashboard/trends/opens")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/notifications/lead-discovery — lead discovery notifications.
    pub async fn notifications_lead_discovery(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            query.push(("workspaceId", wid.to_string()));
        }
        let resp = self
            .get("/api/v1/dashboard/notifications/lead-discovery")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/notifications/campaigns — campaign notifications.
    pub async fn notifications_campaigns(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            query.push(("workspaceId", wid.to_string()));
        }
        let resp = self
            .get("/api/v1/dashboard/notifications/campaigns")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/notifications/replies — reply notifications.
    pub async fn notifications_replies(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            query.push(("workspaceId", wid.to_string()));
        }
        let resp = self
            .get("/api/v1/dashboard/notifications/replies")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/dashboard/insights — AI-generated insights.
    pub async fn dashboard_insights(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            query.push(("workspaceId", wid.to_string()));
        }
        let resp = self
            .get("/api/v1/dashboard/insights")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }
}
