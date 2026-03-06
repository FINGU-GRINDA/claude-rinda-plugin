use crate::client::RindaClient;
use crate::error::Result;
use crate::models::replies::{BulkReadRequest, EmailReply, ListRepliesParams, ReplyStatsByIntent};

impl RindaClient {
    /// GET /api/v1/email-replies — list email replies.
    pub async fn list_replies(&self, params: &ListRepliesParams) -> Result<serde_json::Value> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(l) = params.limit {
            query.push(("limit", l.to_string()));
        }
        if let Some(o) = params.offset {
            query.push(("offset", o.to_string()));
        }
        if let Some(ref wid) = params.workspace_id {
            query.push(("workspaceId", wid.clone()));
        }
        if let Some(is_read) = params.is_read {
            query.push(("isRead", is_read.to_string()));
        }
        if let Some(ref s) = params.sentiment {
            query.push(("sentiment", s.clone()));
        }
        if let Some(ref search) = params.search {
            query.push(("search", search.clone()));
        }
        if let Some(ref eaid) = params.email_account_id {
            query.push(("emailAccountId", eaid.clone()));
        }
        let resp = self
            .get("/api/v1/email-replies/")
            .query(&query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/email-replies/{id} — get a single reply.
    pub async fn get_reply(&self, id: &str) -> Result<EmailReply> {
        let path = format!("/api/v1/email-replies/{}", id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/email-replies/{id} — update a reply.
    pub async fn update_reply(
        &self,
        id: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v1/email-replies/{}", id);
        let resp = self.put(&path).json(&body).send().await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/email-replies/{id} — delete a reply.
    pub async fn delete_reply(&self, id: &str) -> Result<()> {
        let path = format!("/api/v1/email-replies/{}", id);
        let resp = self.delete(&path).send().await?;
        Self::handle_response_empty(resp).await
    }

    /// PUT /api/v1/email-replies/{id}/read — mark a reply as read.
    pub async fn mark_reply_read(&self, id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/email-replies/{}/read", id);
        let resp = self.put(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/email-replies/{id}/unread — mark a reply as unread.
    pub async fn mark_reply_unread(&self, id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/email-replies/{}/unread", id);
        let resp = self.put(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/email-replies/bulk/read — bulk mark replies as read.
    pub async fn bulk_mark_read(&self, req: &BulkReadRequest) -> Result<serde_json::Value> {
        let resp = self
            .put("/api/v1/email-replies/bulk/read")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/email-replies/bulk/unread — bulk mark replies as unread.
    pub async fn bulk_mark_unread(&self, req: &BulkReadRequest) -> Result<serde_json::Value> {
        let resp = self
            .put("/api/v1/email-replies/bulk/unread")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/email-replies/{id}/reclassify — reclassify a reply's intent.
    pub async fn reclassify_reply(&self, id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/email-replies/{}/reclassify", id);
        let resp = self.post(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/email-replies/bulk — bulk delete replies.
    pub async fn bulk_delete_replies(&self, req: &BulkReadRequest) -> Result<serde_json::Value> {
        let resp = self
            .delete("/api/v1/email-replies/bulk")
            .json(req)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/email-replies/thread/{threadId}/important — mark thread as important.
    pub async fn mark_thread_important(&self, thread_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/email-replies/thread/{}/important", thread_id);
        let resp = self.put(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/email-replies/thread/{threadId}/read — mark thread as read.
    pub async fn mark_thread_read(&self, thread_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/email-replies/thread/{}/read", thread_id);
        let resp = self.put(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/email-replies/stats/by-intent — get reply stats grouped by intent.
    pub async fn reply_stats_by_intent(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<ReplyStatsByIntent> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            params.push(("workspaceId", wid.to_string()));
        }
        let resp = self
            .get("/api/v1/email-replies/stats/by-intent")
            .query(&params)
            .send()
            .await?;
        Self::handle_response(resp).await
    }
}
