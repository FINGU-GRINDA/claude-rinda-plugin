use crate::client::RindaClient;
use crate::error::Result;
use crate::models::sequences::{
    CreateSequenceRequest, CreateStepRequest, EnrollmentRequest, EnrollmentResponse,
    GenerateSequenceRequest, GenerateSequenceResponse, OverallStats, SequenceMetrics,
    SequenceResponse, SequenceStep, UpdateStepRequest,
};

impl RindaClient {
    // ---- CRUD ----

    /// POST /api/v1/sequences — create a new sequence.
    pub async fn create_sequence(&self, req: &CreateSequenceRequest) -> Result<SequenceResponse> {
        let resp = self.post("/api/v1/sequences").json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/{id} — get a sequence by ID.
    pub async fn get_sequence(&self, id: &str) -> Result<SequenceResponse> {
        let path = format!("/api/v1/sequences/{}", id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/sequences/{id} — update a sequence.
    pub async fn update_sequence(
        &self,
        id: &str,
        body: serde_json::Value,
    ) -> Result<SequenceResponse> {
        let path = format!("/api/v1/sequences/{}", id);
        let resp = self.put(&path).json(&body).send().await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/sequences/{id} — delete a sequence.
    pub async fn delete_sequence(&self, id: &str) -> Result<()> {
        let path = format!("/api/v1/sequences/{}", id);
        let resp = self.delete(&path).send().await?;
        Self::handle_response_empty(resp).await
    }

    /// GET /api/v1/sequences — list sequences.
    pub async fn list_sequences(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<serde_json::Value> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(o) = offset {
            params.push(("offset", o.to_string()));
        }
        let resp = self.get("/api/v1/sequences").query(&params).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/search — search sequences.
    pub async fn search_sequences(&self, query: &str) -> Result<serde_json::Value> {
        let resp = self
            .get("/api/v1/sequences/search")
            .query(&[("q", query)])
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/workspace/{workspaceId} — list sequences by workspace.
    pub async fn list_sequences_by_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v1/sequences/workspace/{}", workspace_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    // ---- Steps ----

    /// GET /api/v1/sequences/{id}/steps — list steps for a sequence.
    pub async fn list_steps(&self, sequence_id: &str) -> Result<Vec<SequenceStep>> {
        let path = format!("/api/v1/sequences/{}/steps", sequence_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/sequences/{id}/steps — create a step.
    pub async fn create_step(
        &self,
        sequence_id: &str,
        req: &CreateStepRequest,
    ) -> Result<SequenceStep> {
        let path = format!("/api/v1/sequences/{}/steps", sequence_id);
        let resp = self.post(&path).json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/sequences/{id}/steps/{stepId} — update a step.
    pub async fn update_step(
        &self,
        sequence_id: &str,
        step_id: &str,
        req: &UpdateStepRequest,
    ) -> Result<SequenceStep> {
        let path = format!("/api/v1/sequences/{}/steps/{}", sequence_id, step_id);
        let resp = self.put(&path).json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// DELETE /api/v1/sequences/{id}/steps/{stepId} — delete a step.
    pub async fn delete_step(&self, sequence_id: &str, step_id: &str) -> Result<()> {
        let path = format!("/api/v1/sequences/{}/steps/{}", sequence_id, step_id);
        let resp = self.delete(&path).send().await?;
        Self::handle_response_empty(resp).await
    }

    // ---- Generation ----

    /// POST /api/v1/sequences/{id}/generate — AI-generate sequence steps.
    pub async fn generate_sequence(
        &self,
        sequence_id: &str,
        req: &GenerateSequenceRequest,
    ) -> Result<GenerateSequenceResponse> {
        let path = format!("/api/v1/sequences/{}/generate", sequence_id);
        let resp = self.post(&path).json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/sequences/generate-template — generate a sequence template.
    pub async fn generate_template(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/sequences/generate-template")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/sequences/suggest-name — suggest a sequence name.
    pub async fn suggest_name(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .post("/api/v1/sequences/suggest-name")
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    // ---- Enrollments ----

    /// POST /api/v1/admin/sequences/{id}/enrollments/bulk-with-scheduling — bulk enroll with scheduling.
    pub async fn create_enrollment_bulk(
        &self,
        sequence_id: &str,
        req: &EnrollmentRequest,
    ) -> Result<EnrollmentResponse> {
        let path = format!(
            "/api/v1/admin/sequences/{}/enrollments/bulk-with-scheduling",
            sequence_id
        );
        let resp = self.post(&path).json(req).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/{id}/enrollments — list enrollments.
    pub async fn list_enrollments(&self, sequence_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/sequences/{}/enrollments", sequence_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// PUT /api/v1/sequences/{id}/enrollments/{enrollmentId}/status — update enrollment status.
    pub async fn update_enrollment_status(
        &self,
        sequence_id: &str,
        enrollment_id: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let path = format!(
            "/api/v1/sequences/{}/enrollments/{}/status",
            sequence_id, enrollment_id
        );
        let resp = self.put(&path).json(&body).send().await?;
        Self::handle_response(resp).await
    }

    // ---- Stats / Metrics ----

    /// GET /api/v1/sequences/stats/overall — overall stats across all sequences.
    pub async fn overall_stats(&self, workspace_id: Option<&str>) -> Result<OverallStats> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(wid) = workspace_id {
            params.push(("workspaceId", wid.to_string()));
        }
        let resp = self
            .get("/api/v1/sequences/stats/overall")
            .query(&params)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/{id}/metrics — per-sequence metrics.
    pub async fn get_sequence_metrics(&self, sequence_id: &str) -> Result<SequenceMetrics> {
        let path = format!("/api/v1/sequences/{}/metrics", sequence_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/enrollments/{enrollmentId}/metrics — enrollment metrics.
    pub async fn get_enrollment_metrics(&self, enrollment_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/sequences/enrollments/{}/metrics", enrollment_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }

    // ---- Other ----

    /// POST /api/v1/sequences/{id}/copy — copy a sequence.
    pub async fn copy_sequence(&self, sequence_id: &str) -> Result<SequenceResponse> {
        let path = format!("/api/v1/sequences/{}/copy", sequence_id);
        let resp = self.post(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// POST /api/v1/sequences/{id}/activate-step-based — activate step-based mode.
    pub async fn activate_step_based(&self, sequence_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/sequences/{}/activate-step-based", sequence_id);
        let resp = self.post(&path).send().await?;
        Self::handle_response(resp).await
    }

    /// GET /api/v1/sequences/{id}/leads — list leads in a sequence.
    pub async fn list_sequence_leads(&self, sequence_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/v1/sequences/{}/leads", sequence_id);
        let resp = self.get(&path).send().await?;
        Self::handle_response(resp).await
    }
}
