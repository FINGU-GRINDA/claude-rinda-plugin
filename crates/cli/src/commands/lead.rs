// CLI commands for lead/buyer management (CRUD, search, status, assessment).

use std::process;

use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api_helper::{
    exit_api_error, get_authenticated_client, print_json, require_workspace_id,
};

#[derive(Debug, Args)]
pub struct LeadArgs {
    #[command(subcommand)]
    pub command: LeadCommands,
}

#[derive(Debug, Subcommand)]
pub enum LeadCommands {
    /// Search leads with advanced filtering
    Search {
        /// Business type filter (e.g. "manufacturer")
        #[arg(long)]
        business_type: Option<String>,

        /// City filter
        #[arg(long)]
        city: Option<String>,

        /// Country filter (e.g. "US")
        #[arg(long)]
        country: Option<String>,

        /// Customer group ID filter
        #[arg(long)]
        customer_group_id: Option<String>,

        /// Lead status filter (new|contacted|qualified|unqualified|converted|lost|unsubscribed)
        #[arg(long)]
        lead_status: Option<String>,

        /// Search query string
        #[arg(long)]
        search: Option<String>,

        /// Search type (all|company|country|email|website|industry|category)
        #[arg(long)]
        search_type: Option<String>,

        /// Sort field
        #[arg(long)]
        sort_field: Option<String>,

        /// Sort order (asc|desc)
        #[arg(long)]
        sort_order: Option<String>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<String>,

        /// Pagination offset
        #[arg(long)]
        offset: Option<String>,
    },

    /// Get a lead by ID
    Get {
        /// Lead UUID
        #[arg(long)]
        id: String,
    },

    /// Create a new lead
    Create {
        /// Company name
        #[arg(long)]
        company_name: Option<String>,

        /// Website URL
        #[arg(long)]
        website_url: Option<String>,

        /// Country
        #[arg(long)]
        country: Option<String>,

        /// City
        #[arg(long)]
        city: Option<String>,

        /// Business type
        #[arg(long)]
        business_type: Option<String>,

        /// Contact name
        #[arg(long)]
        contact_name: Option<String>,

        /// Lead status (new|contacted|qualified|unqualified|converted|lost|unsubscribed)
        #[arg(long)]
        lead_status: Option<String>,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Notes
        #[arg(long)]
        notes: Option<String>,

        /// Workspace ID (defaults to credential workspace)
        #[arg(long)]
        workspace_id: Option<String>,
    },

    /// Update an existing lead
    Update {
        /// Lead UUID
        #[arg(long)]
        id: String,

        /// Company name
        #[arg(long)]
        company_name: Option<String>,

        /// Website URL
        #[arg(long)]
        website_url: Option<String>,

        /// Country
        #[arg(long)]
        country: Option<String>,

        /// City
        #[arg(long)]
        city: Option<String>,

        /// Business type
        #[arg(long)]
        business_type: Option<String>,

        /// Contact name
        #[arg(long)]
        contact_name: Option<String>,

        /// Lead status (new|contacted|qualified|unqualified|converted|lost|unsubscribed)
        #[arg(long)]
        lead_status: Option<String>,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Notes
        #[arg(long)]
        notes: Option<String>,
    },

    /// Delete a lead by ID
    Delete {
        /// Lead UUID
        #[arg(long)]
        id: String,
    },

    /// List leads filtered by status
    ByStatus {
        /// Lead status (new|contacted|qualified|unqualified|converted|lost|unsubscribed)
        #[arg(long)]
        status: String,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<String>,

        /// Pagination offset
        #[arg(long)]
        offset: Option<String>,
    },

    /// Get top-scored leads from assessment
    Top {
        /// Customer group ID filter
        #[arg(long)]
        customer_group_id: Option<String>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<String>,
    },

    /// Get leads grouped by assessment tier
    ByTier {
        /// Tier value (e.g. "A", "B", "C")
        #[arg(long)]
        tier: String,

        /// Customer group ID filter
        #[arg(long)]
        customer_group_id: Option<String>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<String>,

        /// Pagination offset
        #[arg(long)]
        offset: Option<String>,
    },
}

pub async fn run(args: LeadArgs) {
    let (client, creds) = get_authenticated_client().await;

    match args.command {
        LeadCommands::Search {
            business_type,
            city,
            country,
            customer_group_id,
            lead_status,
            search,
            search_type,
            sort_field,
            sort_order,
            limit,
            offset,
        } => {
            let search_type_parsed = search_type.as_deref().map(|s| {
                s.parse::<rinda_sdk::types::GetApiV1LeadsSearchSearchType>()
                    .unwrap_or_else(|_| {
                        eprintln!(
                            "Invalid search_type '{}'. Valid values: all, company, country, email, website, industry, category",
                            s
                        );
                        process::exit(1);
                    })
            });

            let sort_order_parsed = sort_order.as_deref().map(|s| {
                s.parse::<rinda_sdk::types::GetApiV1LeadsSearchSortOrder>()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid sort_order '{}'. Valid values: asc, desc", s);
                        process::exit(1);
                    })
            });

            match client
                .get_api_v1_leads_search(
                    business_type.as_deref(),
                    city.as_deref(),
                    country.as_deref(),
                    None, // created_after
                    None, // created_before
                    None, // created_by_ids
                    customer_group_id.as_deref(),
                    None, // filters
                    lead_status.as_deref(),
                    limit.as_deref(),
                    offset.as_deref(),
                    search.as_deref(),
                    search_type_parsed,
                    sort_field.as_deref(),
                    sort_order_parsed,
                    None, // updated_after
                    None, // updated_before
                    None, // workspace_ids
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead search failed", e),
            }
        }

        LeadCommands::Get { id } => {
            let uuid = parse_uuid(&id, "lead ID");

            match client.get_api_v1_leads_by_id(&uuid).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead get failed", e),
            }
        }

        LeadCommands::Create {
            company_name,
            website_url,
            country,
            city,
            business_type,
            contact_name,
            lead_status,
            description,
            notes,
            workspace_id,
        } => {
            let ws_id = if let Some(ws) = workspace_id {
                ws.parse::<Uuid>().unwrap_or_else(|_| {
                    eprintln!("Invalid workspace ID — must be a valid UUID");
                    process::exit(1);
                })
            } else {
                require_workspace_id(&creds)
            };

            let lead_status_parsed = lead_status.as_deref().map(|s| {
                s.parse::<rinda_sdk::types::PostApiV1LeadsBodyLeadStatus>()
                    .unwrap_or_else(|_| {
                        eprintln!(
                            "Invalid lead_status '{}'. Valid values: new, contacted, qualified, unqualified, converted, lost, unsubscribed",
                            s
                        );
                        process::exit(1);
                    })
            });

            let body = rinda_sdk::types::PostApiV1LeadsBody {
                workspace_id: ws_id,
                company_name: parse_opt_newtype(company_name.as_deref(), "company_name"),
                website_url: parse_opt_newtype(website_url.as_deref(), "website_url"),
                country: parse_opt_newtype(country.as_deref(), "country"),
                city: parse_opt_newtype(city.as_deref(), "city"),
                business_type: parse_opt_newtype(business_type.as_deref(), "business_type"),
                contact_name: parse_opt_newtype(contact_name.as_deref(), "contact_name"),
                lead_status: lead_status_parsed,
                description,
                notes,
                // Unused fields set to None/defaults
                address: None,
                collected_at: None,
                contacts: vec![],
                crawl_time_seconds: None,
                created_by: None,
                customer_group_id: None,
                employee_count: None,
                error_message: None,
                final_url: None,
                found_company_name: None,
                founded_year: None,
                gpt_time_seconds: None,
                http_status: None,
                is_business_type_matched: None,
                lead_score: None,
                lead_source: None,
                name_url_match: None,
                social_media: vec![],
                state: None,
            };

            match client.post_api_v1_leads(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead create failed", e),
            }
        }

        LeadCommands::Update {
            id,
            company_name,
            website_url,
            country,
            city,
            business_type,
            contact_name,
            lead_status,
            description,
            notes,
        } => {
            let uuid = parse_uuid(&id, "lead ID");

            let lead_status_parsed = lead_status.as_deref().map(|s| {
                s.parse::<rinda_sdk::types::PutApiV1LeadsByIdBodyLeadStatus>()
                    .unwrap_or_else(|_| {
                        eprintln!(
                            "Invalid lead_status '{}'. Valid values: new, contacted, qualified, unqualified, converted, lost, unsubscribed",
                            s
                        );
                        process::exit(1);
                    })
            });

            let body = rinda_sdk::types::PutApiV1LeadsByIdBody {
                company_name: parse_opt_newtype(company_name.as_deref(), "company_name"),
                website_url: parse_opt_newtype(website_url.as_deref(), "website_url"),
                country: parse_opt_newtype(country.as_deref(), "country"),
                city: parse_opt_newtype(city.as_deref(), "city"),
                business_type: parse_opt_newtype(business_type.as_deref(), "business_type"),
                contact_name: parse_opt_newtype(contact_name.as_deref(), "contact_name"),
                lead_status: lead_status_parsed,
                description,
                notes,
                ..Default::default()
            };

            match client.put_api_v1_leads_by_id(&uuid, &body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead update failed", e),
            }
        }

        LeadCommands::Delete { id } => {
            let uuid = parse_uuid(&id, "lead ID");

            match client.delete_api_v1_leads_by_id(&uuid).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead delete failed", e),
            }
        }

        LeadCommands::ByStatus {
            status,
            limit,
            offset,
        } => {
            let status_parsed = status
                .parse::<rinda_sdk::types::GetApiV1LeadsStatusByStatusStatus>()
                .unwrap_or_else(|_| {
                    eprintln!(
                        "Invalid status '{}'. Valid values: new, contacted, qualified, unqualified, converted, lost, unsubscribed",
                        status
                    );
                    process::exit(1);
                });

            match client
                .get_api_v1_leads_status_by_status(
                    status_parsed,
                    limit.as_deref(),
                    offset.as_deref(),
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead by-status failed", e),
            }
        }

        LeadCommands::Top {
            customer_group_id,
            limit,
        } => {
            let workspace_id = require_workspace_id(&creds);
            let ws_str = workspace_id.to_string();

            match client
                .get_api_v1_assessment_top_leads(
                    customer_group_id.as_deref(),
                    limit.as_deref(),
                    &ws_str,
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead top failed", e),
            }
        }

        LeadCommands::ByTier {
            tier,
            customer_group_id,
            limit,
            offset,
        } => {
            let workspace_id = require_workspace_id(&creds);
            let ws_str = workspace_id.to_string();

            match client
                .get_api_v1_assessment_leads_by_tier(
                    customer_group_id.as_deref(),
                    limit.as_deref(),
                    offset.as_deref(),
                    &tier,
                    &ws_str,
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("lead by-tier failed", e),
            }
        }
    }

    process::exit(0);
}

/// Parse a string as a UUID, exiting with a helpful error message on failure.
fn parse_uuid(s: &str, label: &str) -> Uuid {
    s.parse::<Uuid>().unwrap_or_else(|_| {
        eprintln!("Invalid {label} — must be a valid UUID");
        process::exit(1);
    })
}

/// Parse an optional string into an optional SDK newtype that implements `FromStr`.
/// Exits with an error message if parsing fails.
fn parse_opt_newtype<T>(value: Option<&str>, field: &str) -> Option<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value.map(|s| {
        s.parse::<T>().unwrap_or_else(|e| {
            eprintln!("Invalid value for {field}: {e}");
            process::exit(1);
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that parse_uuid succeeds for a well-formed UUID.
    #[test]
    fn parse_uuid_valid() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(id);
        assert!(result.is_ok());
    }

    /// Acceptance criteria: `rinda lead by-status` requires a valid status string.
    /// Parsing an invalid status should fail — verified here via the SDK enum directly.
    #[test]
    fn by_status_invalid_status_is_rejected() {
        use rinda_sdk::types::GetApiV1LeadsStatusByStatusStatus;
        let result = "invalid_status".parse::<GetApiV1LeadsStatusByStatusStatus>();
        assert!(result.is_err(), "invalid status should be rejected");
    }

    /// Acceptance criteria: all valid status values parse successfully.
    #[test]
    fn by_status_all_valid_statuses_parse() {
        use rinda_sdk::types::GetApiV1LeadsStatusByStatusStatus;
        for s in &[
            "new",
            "contacted",
            "qualified",
            "unqualified",
            "converted",
            "lost",
            "unsubscribed",
        ] {
            let result = s.parse::<GetApiV1LeadsStatusByStatusStatus>();
            assert!(result.is_ok(), "status '{}' should parse successfully", s);
        }
    }

    /// Acceptance criteria: valid search_type values round-trip through Display.
    #[test]
    fn search_type_display_round_trips() {
        use rinda_sdk::types::GetApiV1LeadsSearchSearchType;
        let cases = [
            ("all", GetApiV1LeadsSearchSearchType::All),
            ("company", GetApiV1LeadsSearchSearchType::Company),
            ("email", GetApiV1LeadsSearchSearchType::Email),
        ];
        for (s, expected) in cases {
            let parsed = s.parse::<GetApiV1LeadsSearchSearchType>().unwrap();
            assert_eq!(parsed, expected, "parse mismatch for '{s}'");
            assert_eq!(parsed.to_string(), s, "display mismatch for '{s}'");
        }
    }

    /// Acceptance criteria: sort_order enum accepts asc and desc.
    #[test]
    fn sort_order_valid_values() {
        use rinda_sdk::types::GetApiV1LeadsSearchSortOrder;
        assert!("asc".parse::<GetApiV1LeadsSearchSortOrder>().is_ok());
        assert!("desc".parse::<GetApiV1LeadsSearchSortOrder>().is_ok());
        assert!("DESC".parse::<GetApiV1LeadsSearchSortOrder>().is_err());
    }

    /// Acceptance criteria: lead_status enum parses all valid values.
    #[test]
    fn lead_status_create_enum_parses() {
        use rinda_sdk::types::PostApiV1LeadsBodyLeadStatus;
        for s in &[
            "new",
            "contacted",
            "qualified",
            "unqualified",
            "converted",
            "lost",
            "unsubscribed",
        ] {
            assert!(
                s.parse::<PostApiV1LeadsBodyLeadStatus>().is_ok(),
                "lead_status '{}' should parse for create",
                s
            );
        }
    }

    /// Acceptance criteria: update lead_status enum parses all valid values.
    #[test]
    fn lead_status_update_enum_parses() {
        use rinda_sdk::types::PutApiV1LeadsByIdBodyLeadStatus;
        for s in &[
            "new",
            "contacted",
            "qualified",
            "unqualified",
            "converted",
            "lost",
            "unsubscribed",
        ] {
            assert!(
                s.parse::<PutApiV1LeadsByIdBodyLeadStatus>().is_ok(),
                "lead_status '{}' should parse for update",
                s
            );
        }
    }
}
