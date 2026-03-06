use crate::client::RindaClient;
use crate::error::Result;
use crate::models::enrichment::{EnrichLeadsRequest, EnrichLeadsResponse};

impl RindaClient {
    /// POST /api/v1/contact-enrichment/enrich-leads — enrich leads with contact info.
    pub async fn enrich_leads(&self, req: &EnrichLeadsRequest) -> Result<EnrichLeadsResponse> {
        let resp = self
            .post("/api/v1/contact-enrichment/enrich-leads")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/contact-enrichment/check-email-status — check email enrichment status.
    pub async fn check_email_status(
        &self,
        workspace_id: Option<&str>,
        lead_ids: Option<&[&str]>,
    ) -> Result<serde_json::Value> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            params.push(("workspaceId", wid.to_string()));
        }
        if let Some(ids) = lead_ids {
            for id in ids {
                params.push(("leadIds[]", id.to_string()));
            }
        }
        let resp = self
            .get("/api/v1/contact-enrichment/check-email-status")
            .query(&params)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/contact-enrichment/leads-without-email — list leads without email.
    pub async fn leads_without_email(
        &self,
        workspace_id: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<serde_json::Value> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            params.push(("workspaceId", wid.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(o) = offset {
            params.push(("offset", o.to_string()));
        }
        let resp = self
            .get("/api/v1/contact-enrichment/leads-without-email")
            .query(&params)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/contact-enrichment/apply-results — apply enrichment results to leads.
    pub async fn apply_enrichment_results(
        &self,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/contact-enrichment/apply-results")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }
}
