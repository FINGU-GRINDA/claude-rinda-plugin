mod auth;
mod oauth;
mod tools;

use rmcp::{
    ServerHandler,
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
struct BuyerSessionsParams {
    #[schemars(description = "Optional user ID (UUID) to filter sessions by a specific user")]
    user_id: Option<String>,
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OrderHistoryParams {
    #[schemars(description = "Filter by buyer/lead ID or name")]
    buyer_id: Option<String>,
    #[schemars(description = "Only include leads inactive for at least this many days")]
    days_inactive: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadSearchParams {
    #[schemars(description = "Business type filter (e.g. \"manufacturer\")")]
    business_type: Option<String>,
    #[schemars(description = "City filter")]
    city: Option<String>,
    #[schemars(description = "Country filter (e.g. \"US\")")]
    country: Option<String>,
    #[schemars(description = "Customer group ID filter")]
    customer_group_id: Option<String>,
    #[schemars(
        description = "Lead status filter (new|contacted|qualified|unqualified|converted|lost|unsubscribed)"
    )]
    lead_status: Option<String>,
    #[schemars(description = "Search query string")]
    search: Option<String>,
    #[schemars(description = "Search type (all|company|country|email|website|industry|category)")]
    search_type: Option<String>,
    #[schemars(description = "Sort field")]
    sort_field: Option<String>,
    #[schemars(description = "Sort order (asc|desc)")]
    sort_order: Option<String>,
    #[schemars(description = "Maximum number of results")]
    limit: Option<String>,
    #[schemars(description = "Pagination offset")]
    offset: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadIdParams {
    #[schemars(description = "Lead UUID")]
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadCreateParams {
    #[schemars(description = "Company name")]
    company_name: Option<String>,
    #[schemars(description = "Website URL")]
    website_url: Option<String>,
    #[schemars(description = "Country")]
    country: Option<String>,
    #[schemars(description = "City")]
    city: Option<String>,
    #[schemars(description = "Business type")]
    business_type: Option<String>,
    #[schemars(description = "Contact name")]
    contact_name: Option<String>,
    #[schemars(
        description = "Lead status (new|contacted|qualified|unqualified|converted|lost|unsubscribed)"
    )]
    lead_status: Option<String>,
    #[schemars(description = "Description")]
    description: Option<String>,
    #[schemars(description = "Notes")]
    notes: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadUpdateParams {
    #[schemars(description = "Lead UUID")]
    id: String,
    #[schemars(description = "Company name")]
    company_name: Option<String>,
    #[schemars(description = "Website URL")]
    website_url: Option<String>,
    #[schemars(description = "Country")]
    country: Option<String>,
    #[schemars(description = "City")]
    city: Option<String>,
    #[schemars(description = "Business type")]
    business_type: Option<String>,
    #[schemars(description = "Contact name")]
    contact_name: Option<String>,
    #[schemars(
        description = "Lead status (new|contacted|qualified|unqualified|converted|lost|unsubscribed)"
    )]
    lead_status: Option<String>,
    #[schemars(description = "Description")]
    description: Option<String>,
    #[schemars(description = "Notes")]
    notes: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadByStatusParams {
    #[schemars(
        description = "Lead status (new|contacted|qualified|unqualified|converted|lost|unsubscribed)"
    )]
    status: String,
    #[schemars(description = "Maximum number of results")]
    limit: Option<String>,
    #[schemars(description = "Pagination offset")]
    offset: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadTopParams {
    #[schemars(description = "Customer group ID filter")]
    customer_group_id: Option<String>,
    #[schemars(description = "Maximum number of results")]
    limit: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeadByTierParams {
    #[schemars(description = "Assessment tier (e.g. \"A\", \"B\", \"C\")")]
    tier: String,
    #[schemars(description = "Customer group ID filter")]
    customer_group_id: Option<String>,
    #[schemars(description = "Maximum number of results")]
    limit: Option<String>,
    #[schemars(description = "Pagination offset")]
    offset: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupCreateParams {
    #[schemars(description = "Group name (required, 1-255 chars)")]
    name: String,
    #[schemars(
        description = "Workspace ID override (UUID). Defaults to the workspace in the token."
    )]
    workspace_id: Option<String>,
    #[schemars(description = "Optional description")]
    description: Option<String>,
    #[schemars(description = "Whether this is a dynamic group")]
    is_dynamic: Option<bool>,
    #[schemars(description = "Enable automatic enrichment for group members")]
    auto_enrich_enabled: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupListParams {
    #[schemars(description = "Search query string")]
    search: Option<String>,
    #[schemars(description = "Maximum number of results to return")]
    limit: Option<String>,
    #[schemars(description = "Pagination offset")]
    offset: Option<String>,
    #[schemars(description = "Filter by dynamic flag (\"true\" or \"false\")")]
    is_dynamic: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupIdParams {
    #[schemars(description = "Customer group UUID")]
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupUpdateParams {
    #[schemars(description = "Customer group UUID")]
    id: String,
    #[schemars(description = "New group name (required, 1-255 chars)")]
    name: String,
    #[schemars(description = "Whether this is a dynamic group (required)")]
    is_dynamic: bool,
    #[schemars(description = "New description")]
    description: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupMembersParams {
    #[schemars(description = "Customer group UUID")]
    id: String,
    #[schemars(description = "Maximum number of results to return")]
    limit: Option<String>,
    #[schemars(description = "Pagination offset")]
    offset: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupMemberParams {
    #[schemars(description = "Customer group UUID")]
    id: String,
    #[schemars(description = "Lead UUID")]
    lead_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupLeadParams {
    #[schemars(description = "Lead UUID to look up group memberships for")]
    lead_id: String,
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

// ── Helper: extract auth or return error JSON ─────────────────────────────────

/// Extract the `AuthContext` from the HTTP request parts injected by the
/// `StreamableHttpService`. Returns `Some(auth)` on success or writes an error
/// response string and returns `None`.
macro_rules! require_auth {
    ($parts:expr) => {
        match auth::extract_auth_from_parts(&$parts) {
            Ok(ctx) => ctx,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        }
    };
}

// ── Tool implementations ─────────────────────────────────────────────────────

#[tool_router]
impl RindaMcpServer {
    #[tool(
        description = "Return current authentication status: email, workspace, token expiry. Works without credentials — returns not-authenticated status."
    )]
    async fn rinda_auth_status(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(_): Parameters<EmptyParams>,
    ) -> String {
        tools::auth::auth_status(Some(&parts)).await
    }

    #[tool(
        description = "Start an async buyer search. Returns sessionId for polling. Params: industry, countries (comma-separated codes), buyer_type, min_revenue (USD), limit."
    )]
    async fn rinda_buyer_search(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<BuyerSearchParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_search(
            &auth,
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
    async fn rinda_buyer_status(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SessionIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_status(&auth, p.session_id).await
    }

    #[tool(
        description = "Get the results of a completed buyer search session. Param: session_id (UUID)."
    )]
    async fn rinda_buyer_results(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SessionIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_results(&auth, p.session_id).await
    }

    #[tool(
        description = "Save selected leads from a discovery session. Params: session_id (UUID), recommendation_id."
    )]
    async fn rinda_buyer_select(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<BuyerSelectParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_select(&auth, p.session_id, p.recommendation_id).await
    }

    #[tool(
        description = "Enrich a buyer/lead with additional contact and company data. Param: buyer_id (website URL or lead ID)."
    )]
    async fn rinda_buyer_enrich(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<BuyerEnrichParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_enrich(&auth, p.buyer_id).await
    }

    #[tool(
        description = "Submit answers to clarification questions for a search session in waiting_clarification status. Params: session_id (UUID), answers (JSON object string)."
    )]
    async fn rinda_buyer_clarify(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<BuyerClarifyParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_clarify(&auth, p.session_id, p.answers).await
    }

    #[tool(
        description = "Get campaign dashboard statistics. Param: period (e.g. \"7d\", \"30d\", \"90d\"; default \"30d\")."
    )]
    async fn rinda_campaign_stats(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<CampaignStatsParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::campaign::campaign_stats(&auth, p.period).await
    }

    #[tool(
        description = "Send an email via RINDA. Params: to (recipient email), subject, body (plain text)."
    )]
    async fn rinda_email_send(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<EmailSendParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::email::email_send(&auth, p.to, p.subject, p.body).await
    }

    #[tool(
        description = "Get recent email replies. Param: limit (max replies to return; default 50)."
    )]
    async fn rinda_reply_check(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<ReplyCheckParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::reply::reply_check(&auth, p.limit).await
    }

    #[tool(
        description = "Create a new email sequence. Params: name (required), seq_type (e.g. \"email\"), steps (JSON array of step objects)."
    )]
    async fn rinda_sequence_create(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SequenceCreateParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::sequence::sequence_create(&auth, p.name, p.seq_type, p.steps).await
    }

    #[tool(description = "List existing email sequences. Params: limit, offset (for pagination).")]
    async fn rinda_sequence_list(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SequenceListParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::sequence::sequence_list(&auth, p.limit, p.offset).await
    }

    #[tool(
        description = "AI-generate email steps for an existing sequence. Param: id (sequence UUID)."
    )]
    async fn rinda_sequence_generate(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SequenceIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::sequence::sequence_generate(&auth, p.id).await
    }

    #[tool(
        description = "Enroll a lead/buyer into an email sequence. Params: sequence_id (UUID), buyer_id (UUID)."
    )]
    async fn rinda_sequence_add_contact(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SequenceAddContactParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::sequence::sequence_add_contact(&auth, p.sequence_id, p.buyer_id).await
    }

    #[tool(
        description = "Retrieve clarification questions for a buyer search session in waiting_clarification status. Param: session_id (UUID)."
    )]
    async fn rinda_buyer_messages(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<SessionIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_messages(&auth, p.session_id).await
    }

    #[tool(
        description = "List all past buyer search sessions for the current workspace. Param: user_id (optional UUID to filter by user)."
    )]
    async fn rinda_buyer_sessions(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<BuyerSessionsParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::buyer::buyer_sessions(&auth, p.user_id).await
    }

    #[tool(
        description = "Retrieve order history (uses leads/search; no dedicated orders API). Params: buyer_id (filter by name/ID), days_inactive (minimum inactive days)."
    )]
    async fn rinda_order_history(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<OrderHistoryParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::order::order_history(&auth, p.buyer_id, p.days_inactive).await
    }

    #[tool(
        description = "List workspaces the authenticated user belongs to. No parameters required."
    )]
    async fn rinda_workspace_list(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(_): Parameters<EmptyParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::workspace::workspace_list(&auth).await
    }

    #[tool(
        description = "Search leads with advanced filtering. Params: business_type, city, country, customer_group_id, lead_status, search, search_type (all|company|country|email|website|industry|category), sort_field, sort_order (asc|desc), limit, offset."
    )]
    async fn rinda_lead_search(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadSearchParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_search(
            &auth,
            p.business_type,
            p.city,
            p.country,
            p.customer_group_id,
            p.lead_status,
            p.search,
            p.search_type,
            p.sort_field,
            p.sort_order,
            p.limit,
            p.offset,
        )
        .await
    }

    #[tool(description = "Get a lead by its UUID. Param: id (lead UUID).")]
    async fn rinda_lead_get(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_get(&auth, p.id).await
    }

    #[tool(
        description = "Create a new lead. Params: company_name, website_url, country, city, business_type, contact_name, lead_status (new|contacted|qualified|unqualified|converted|lost|unsubscribed), description, notes."
    )]
    async fn rinda_lead_create(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadCreateParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_create(
            &auth,
            p.company_name,
            p.website_url,
            p.country,
            p.city,
            p.business_type,
            p.contact_name,
            p.lead_status,
            p.description,
            p.notes,
        )
        .await
    }

    #[tool(
        description = "Update an existing lead. Params: id (UUID, required), plus any of: company_name, website_url, country, city, business_type, contact_name, lead_status (new|contacted|qualified|unqualified|converted|lost|unsubscribed), description, notes."
    )]
    async fn rinda_lead_update(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadUpdateParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_update(
            &auth,
            p.id,
            p.company_name,
            p.website_url,
            p.country,
            p.city,
            p.business_type,
            p.contact_name,
            p.lead_status,
            p.description,
            p.notes,
        )
        .await
    }

    #[tool(description = "Delete a lead by UUID. Param: id (lead UUID).")]
    async fn rinda_lead_delete(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_delete(&auth, p.id).await
    }

    #[tool(
        description = "List leads filtered by status. Params: status (new|contacted|qualified|unqualified|converted|lost|unsubscribed, required), limit, offset."
    )]
    async fn rinda_lead_by_status(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadByStatusParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_by_status(&auth, p.status, p.limit, p.offset).await
    }

    #[tool(
        description = "Get top-scored leads from AI assessment. Params: customer_group_id (optional), limit (optional). Uses workspace from token."
    )]
    async fn rinda_lead_top(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadTopParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_top(&auth, p.customer_group_id, p.limit).await
    }

    #[tool(
        description = "Get leads grouped by assessment tier. Params: tier (required, e.g. \"A\", \"B\", \"C\"), customer_group_id, limit, offset. Uses workspace from token."
    )]
    async fn rinda_lead_by_tier(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<LeadByTierParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::lead::lead_by_tier(&auth, p.tier, p.customer_group_id, p.limit, p.offset).await
    }

    #[tool(
        description = "Create a new customer group. Params: name (required, 1-255 chars), workspace_id (UUID, defaults to token workspace), description, is_dynamic, auto_enrich_enabled."
    )]
    async fn rinda_group_create(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupCreateParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_create(
            &auth,
            p.name,
            p.workspace_id,
            p.description,
            p.is_dynamic,
            p.auto_enrich_enabled,
        )
        .await
    }

    #[tool(
        description = "Search / list customer groups for the current workspace. Params: search (query string), limit, offset, is_dynamic (\"true\"/\"false\")."
    )]
    async fn rinda_group_list(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupListParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_list(&auth, p.search, p.limit, p.offset, p.is_dynamic).await
    }

    #[tool(description = "Get a customer group by ID. Param: id (group UUID).")]
    async fn rinda_group_get(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_get(&auth, p.id).await
    }

    #[tool(
        description = "Update a customer group. Params: id (UUID), name (required, 1-255 chars), is_dynamic (required bool), description."
    )]
    async fn rinda_group_update(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupUpdateParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_update(&auth, p.id, p.name, p.is_dynamic, p.description).await
    }

    #[tool(description = "Delete a customer group. Param: id (group UUID).")]
    async fn rinda_group_delete(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupIdParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_delete(&auth, p.id).await
    }

    #[tool(
        description = "List members of a customer group. Params: id (group UUID), limit, offset."
    )]
    async fn rinda_group_members(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupMembersParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_members(&auth, p.id, p.limit, p.offset).await
    }

    #[tool(
        description = "Add a lead as a member of a customer group. Params: id (group UUID), lead_id (lead UUID)."
    )]
    async fn rinda_group_add_member(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupMemberParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_add_member(&auth, p.id, p.lead_id).await
    }

    #[tool(
        description = "Remove a lead from a customer group. Params: id (group UUID), lead_id (lead UUID)."
    )]
    async fn rinda_group_remove_member(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupMemberParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_remove_member(&auth, p.id, p.lead_id).await
    }

    #[tool(
        description = "List all customer groups that a given lead belongs to. Param: lead_id (lead UUID)."
    )]
    async fn rinda_group_for_lead(
        &self,
        rmcp::handler::server::tool::Extension(parts): rmcp::handler::server::tool::Extension<
            http::request::Parts,
        >,
        Parameters(p): Parameters<GroupLeadParams>,
    ) -> String {
        let auth = require_auth!(parts);
        tools::group::group_for_lead(&auth, p.lead_id).await
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

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::routing::get;
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let server_url =
        std::env::var("MCP_SERVER_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));

    let rinda_base_url = rinda_common::config::base_url().to_string();

    let oauth_state = Arc::new(oauth::OAuthState::new(rinda_base_url, server_url));

    let ct = CancellationToken::new();
    let service: StreamableHttpService<RindaMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(RindaMcpServer::new()),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

    let app = axum::Router::new()
        .route("/health", get(health))
        .route(
            "/.well-known/oauth-protected-resource",
            get(oauth::protected_resource_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(oauth::metadata),
        )
        .route("/oauth/authorize", get(oauth::authorize))
        .route("/oauth/callback", get(oauth::oauth_callback))
        .route("/oauth/token", axum::routing::post(oauth::token))
        .route("/oauth/register", axum::routing::post(oauth::register))
        .with_state(oauth_state.clone())
        .fallback_service(
            tower::ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    oauth_state,
                    oauth::auth_middleware,
                ))
                .service(service),
        );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    eprintln!("MCP server listening on 0.0.0.0:{port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await?;
    Ok(())
}
