mod auth;
mod tools;

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

// ── Parameter structs ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EmptyParams {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuyerSearchParams {
    #[schemars(description = "Industry filter (e.g. \"manufacturing\", \"cosmetics\")")]
    industry: Option<String>,
    #[schemars(description = "Comma-separated list of country codes (e.g. \"US,DE\")")]
    countries: Option<String>,
    #[schemars(description = "Buyer type filter (e.g. \"importer\", \"distributor\")")]
    buyer_type: Option<String>,
    #[schemars(description = "Minimum annual revenue in USD")]
    min_revenue: Option<f64>,
    #[schemars(description = "Maximum number of results to return (default 20)")]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SessionIdParams {
    #[schemars(description = "Session ID (UUID) from the search request")]
    session_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuyerSelectParams {
    #[schemars(description = "Session ID (UUID) from the search request")]
    session_id: String,
    #[schemars(description = "Selected recommendation ID")]
    recommendation_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuyerEnrichParams {
    #[schemars(description = "Buyer ID or website URL of the lead to enrich")]
    buyer_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuyerClarifyParams {
    #[schemars(description = "Session ID (UUID) from the search request")]
    session_id: String,
    #[schemars(
        description = "Clarification answers as a JSON object string (e.g. '{\"field\": \"value\"}')"
    )]
    answers: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CampaignStatsParams {
    #[schemars(
        description = "Period to query (e.g. \"7d\", \"30d\", \"90d\"). Defaults to \"30d\"."
    )]
    period: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EmailSendParams {
    #[schemars(description = "Recipient email address")]
    to: String,
    #[schemars(description = "Email subject line")]
    subject: String,
    #[schemars(description = "Email body text")]
    body: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReplyCheckParams {
    #[schemars(description = "Maximum number of replies to return (default 50)")]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SequenceCreateParams {
    #[schemars(description = "Sequence name")]
    name: String,
    #[schemars(description = "Sequence type (e.g. \"email\", \"linkedin\")")]
    seq_type: Option<String>,
    #[schemars(
        description = "JSON array of steps (e.g. '[{\"delay\":1,\"template\":\"intro\"}]')"
    )]
    steps: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SequenceListParams {
    #[schemars(description = "Maximum number of sequences to return")]
    limit: Option<String>,
    #[schemars(description = "Offset for pagination")]
    offset: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SequenceIdParams {
    #[schemars(description = "Sequence ID (UUID)")]
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SequenceAddContactParams {
    #[schemars(description = "Sequence ID (UUID)")]
    sequence_id: String,
    #[schemars(description = "Buyer / lead ID (UUID) to enroll")]
    buyer_id: String,
}

// ── Server struct ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct RindaMcpServer {
    tool_router: ToolRouter<Self>,
}

impl RindaMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

// ── Tool implementations ─────────────────────────────────────────────────────

#[tool_router]
impl RindaMcpServer {
    #[tool(
        description = "Return current authentication status: email, workspace, token expiry. Works without credentials — returns not-authenticated status."
    )]
    async fn rinda_auth_status(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        tools::auth::auth_status().await
    }

    #[tool(
        description = "Return the browser login URL and instructions to authenticate with RINDA. Use this when not logged in."
    )]
    async fn rinda_auth_login(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        tools::auth::auth_login().await
    }

    #[tool(
        description = "Start an async buyer search. Returns sessionId for polling. Params: industry, countries (comma-separated codes), buyer_type, min_revenue (USD), limit."
    )]
    async fn rinda_buyer_search(&self, Parameters(p): Parameters<BuyerSearchParams>) -> String {
        tools::buyer::buyer_search(
            p.industry,
            p.countries,
            p.buyer_type,
            p.min_revenue,
            p.limit,
        )
        .await
    }

    #[tool(
        description = "Poll the status of an async buyer search session. Param: session_id (UUID)."
    )]
    async fn rinda_buyer_status(&self, Parameters(p): Parameters<SessionIdParams>) -> String {
        tools::buyer::buyer_status(p.session_id).await
    }

    #[tool(
        description = "Get the results of a completed buyer search session. Param: session_id (UUID)."
    )]
    async fn rinda_buyer_results(&self, Parameters(p): Parameters<SessionIdParams>) -> String {
        tools::buyer::buyer_results(p.session_id).await
    }

    #[tool(
        description = "Save selected leads from a discovery session. Params: session_id (UUID), recommendation_id."
    )]
    async fn rinda_buyer_select(&self, Parameters(p): Parameters<BuyerSelectParams>) -> String {
        tools::buyer::buyer_select(p.session_id, p.recommendation_id).await
    }

    #[tool(
        description = "Enrich a buyer/lead with additional contact and company data. Param: buyer_id (website URL or lead ID)."
    )]
    async fn rinda_buyer_enrich(&self, Parameters(p): Parameters<BuyerEnrichParams>) -> String {
        tools::buyer::buyer_enrich(p.buyer_id).await
    }

    #[tool(
        description = "Submit answers to clarification questions for a search session in waiting_clarification status. Params: session_id (UUID), answers (JSON object string)."
    )]
    async fn rinda_buyer_clarify(&self, Parameters(p): Parameters<BuyerClarifyParams>) -> String {
        tools::buyer::buyer_clarify(p.session_id, p.answers).await
    }

    #[tool(
        description = "Get campaign dashboard statistics. Param: period (e.g. \"7d\", \"30d\", \"90d\"; default \"30d\")."
    )]
    async fn rinda_campaign_stats(&self, Parameters(p): Parameters<CampaignStatsParams>) -> String {
        tools::campaign::campaign_stats(p.period).await
    }

    #[tool(
        description = "Send an email via RINDA. Params: to (recipient email), subject, body (plain text)."
    )]
    async fn rinda_email_send(&self, Parameters(p): Parameters<EmailSendParams>) -> String {
        tools::email::email_send(p.to, p.subject, p.body).await
    }

    #[tool(
        description = "Get recent email replies. Param: limit (max replies to return; default 50)."
    )]
    async fn rinda_reply_check(&self, Parameters(p): Parameters<ReplyCheckParams>) -> String {
        tools::reply::reply_check(p.limit).await
    }

    #[tool(
        description = "Create a new email sequence. Params: name (required), seq_type (e.g. \"email\"), steps (JSON array of step objects)."
    )]
    async fn rinda_sequence_create(
        &self,
        Parameters(p): Parameters<SequenceCreateParams>,
    ) -> String {
        tools::sequence::sequence_create(p.name, p.seq_type, p.steps).await
    }

    #[tool(description = "List existing email sequences. Params: limit, offset (for pagination).")]
    async fn rinda_sequence_list(&self, Parameters(p): Parameters<SequenceListParams>) -> String {
        tools::sequence::sequence_list(p.limit, p.offset).await
    }

    #[tool(
        description = "AI-generate email steps for an existing sequence. Param: id (sequence UUID)."
    )]
    async fn rinda_sequence_generate(&self, Parameters(p): Parameters<SequenceIdParams>) -> String {
        tools::sequence::sequence_generate(p.id).await
    }

    #[tool(
        description = "Enroll a lead/buyer into an email sequence. Params: sequence_id (UUID), buyer_id (UUID)."
    )]
    async fn rinda_sequence_add_contact(
        &self,
        Parameters(p): Parameters<SequenceAddContactParams>,
    ) -> String {
        tools::sequence::sequence_add_contact(p.sequence_id, p.buyer_id).await
    }
}

// ── ServerHandler implementation ─────────────────────────────────────────────

#[tool_handler(router = self.tool_router)]
impl ServerHandler for RindaMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("rinda-mcp", env!("CARGO_PKG_VERSION")))
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = RindaMcpServer::new();
    let transport = rmcp::transport::stdio();
    let service = server.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}
