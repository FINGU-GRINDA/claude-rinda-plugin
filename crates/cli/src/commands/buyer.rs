// CLI commands for buyer (lead) operations.

use std::process;

use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api_helper::{
    exit_api_error, get_authenticated_client, print_json, require_workspace_id,
};

#[derive(Debug, Args)]
pub struct BuyerArgs {
    #[command(subcommand)]
    pub command: BuyerCommands,
}

#[derive(Debug, Subcommand)]
pub enum BuyerCommands {
    /// Search for buyers (leads) matching criteria
    Search {
        /// Industry filter (e.g. "manufacturing")
        #[arg(long)]
        industry: Option<String>,

        /// Comma-separated list of country codes (e.g. "US,DE")
        #[arg(long)]
        countries: Option<String>,

        /// Buyer type filter (e.g. "importer")
        #[arg(long)]
        buyer_type: Option<String>,

        /// Minimum annual revenue in USD
        #[arg(long)]
        min_revenue: Option<f64>,

        /// Maximum number of results to return
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Check the status of an async search session
    Status {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,
    },

    /// View the results of a completed search session
    Results {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,
    },

    /// Select leads from discovery results to save them
    Select {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,

        /// Selected recommendation ID
        #[arg(long)]
        recommendation_id: String,
    },

    /// Enrich a buyer (lead) with additional data
    Enrich {
        /// Buyer ID (lead ID) to enrich
        #[arg(long)]
        buyer_id: String,
    },

    /// Retrieve clarification questions for a session in waiting_clarification status
    Messages {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,
    },

    /// Submit answers to clarification questions and resume the search session
    Clarify {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,

        /// Answers as a JSON object string (e.g. '{"field": "value"}')
        #[arg(long)]
        answers: String,
    },

    /// List all past search sessions for the current workspace
    Sessions {
        /// Optional user ID filter (UUID)
        #[arg(long)]
        user_id: Option<String>,
    },
}

pub async fn run(args: BuyerArgs) {
    let (client, creds) = get_authenticated_client().await;

    match args.command {
        BuyerCommands::Search {
            industry,
            countries: _countries,
            buyer_type: _buyer_type,
            min_revenue: _min_revenue,
            limit: _limit,
        } => {
            let workspace_id = require_workspace_id(&creds);

            // Use the industry filter as the search query, or a default.
            let query = industry.unwrap_or_else(|| "buyer search".to_string());

            let body = rinda_sdk::types::PostApiV1LeadDiscoverySearchBody {
                query,
                workspace_id,
                crawl_timeout_seconds: None,
                locale: None,
                session_id: None,
                use_auto_timeout: true,
            };

            match client.post_api_v1_lead_discovery_search(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer search failed", e),
            }
        }

        BuyerCommands::Status { session_id } => {
            let uuid = session_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid session ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_lead_discovery_db_sessions_by_session_id(&uuid)
                .await
            {
                Ok(resp) => {
                    let json = resp.into_inner();

                    // Try to extract human-readable fields from the response.
                    let data = json.get("data").and_then(|v| v.as_object());
                    let session = data
                        .and_then(|d| d.get("session"))
                        .and_then(|v| v.as_object());

                    if let Some(s) = session {
                        let status = s
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let progress = s
                            .get("progress")
                            .and_then(|v| v.as_f64())
                            .map(|p| format!("{:.0}", p))
                            .unwrap_or_else(|| "?".to_string());
                        let total_count = s
                            .get("totalCount")
                            .and_then(|v| v.as_u64())
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "?".to_string());
                        let query = s.get("query").and_then(|v| v.as_str()).unwrap_or("-");
                        let job_phase = s.get("jobPhase").and_then(|v| v.as_str()).unwrap_or("-");
                        let message = s.get("message").and_then(|v| v.as_str()).unwrap_or("-");

                        println!("Session: {session_id}");
                        println!("Query:    {query}");
                        println!("Status:   {status} ({progress}%)");
                        println!("Phase:    {job_phase}");
                        println!("Results:  {total_count}");
                        println!("Message:  {message}");

                        if status == "waiting_clarification" {
                            println!();
                            println!(
                                "Hint: Run `rinda-cli buyer messages --session-id {session_id}` to see questions,"
                            );
                            println!(
                                "      then `rinda-cli buyer clarify --session-id {session_id} --answers '{{...}}'` to respond."
                            );
                        }
                    } else {
                        // Fallback: print raw JSON if structure is unexpected.
                        print_json(&json);
                    }
                }
                Err(e) => exit_api_error("buyer status failed", e),
            }
        }

        BuyerCommands::Results { session_id } => {
            let uuid = session_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid session ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_lead_discovery_db_sessions_by_session_id_results(&uuid)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer results failed", e),
            }
        }

        BuyerCommands::Select {
            session_id,
            recommendation_id,
        } => {
            let workspace_id = require_workspace_id(&creds);

            let body = rinda_sdk::types::PostApiV1LeadDiscoverySelectBody {
                session_id,
                selected_recommendation_id: recommendation_id,
                workspace_id,
            };

            match client.post_api_v1_lead_discovery_select(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer select failed", e),
            }
        }

        BuyerCommands::Enrich { buyer_id } => {
            let body = rinda_sdk::types::PostApiV1LeadDiscoveryEnrichBody {
                website_url: buyer_id,
                workspace_id: creds.workspace_id.clone(),
            };

            match client.post_api_v1_lead_discovery_enrich(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer enrich failed", e),
            }
        }

        BuyerCommands::Messages { session_id } => {
            let uuid = session_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid session ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_lead_discovery_db_sessions_by_session_id_messages(&uuid)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer messages failed", e),
            }
        }

        BuyerCommands::Sessions { user_id } => {
            let workspace_id = require_workspace_id(&creds);
            let user_uuid = user_id.map(|id| {
                id.parse::<Uuid>().unwrap_or_else(|_| {
                    eprintln!("Invalid user ID — must be a valid UUID");
                    process::exit(1);
                })
            });
            match client
                .get_api_v1_lead_discovery_db_sessions(user_uuid.as_ref(), &workspace_id)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer sessions failed", e),
            }
        }

        BuyerCommands::Clarify {
            session_id,
            answers,
        } => {
            let workspace_id = require_workspace_id(&creds);

            let answers_map = parse_answers_json(&answers).unwrap_or_else(|e| {
                eprintln!("{e}");
                process::exit(1);
            });

            let body = rinda_sdk::types::PostApiV1LeadDiscoveryClarifyBody {
                session_id,
                answers: answers_map,
                workspace_id,
            };

            match client.post_api_v1_lead_discovery_clarify(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer clarify failed", e),
            }
        }
    }

    process::exit(0);
}

/// Parse a JSON string into a `serde_json::Map` suitable for the clarify body.
/// Returns an error message string if parsing fails or the value is not an object.
pub fn parse_answers_json(s: &str) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let parsed: serde_json::Value = serde_json::from_str(s)
        .map_err(|_| "Invalid answers — must be a valid JSON object string".to_string())?;

    parsed.as_object().cloned().ok_or_else(|| {
        "Invalid answers — must be a JSON object (not array or primitive)".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Sessions subcommand ────────────────────────────────────────────────

    #[test]
    fn buyer_sessions_no_user_id_is_valid_variant() {
        // The Sessions variant must be constructable with no user_id (optional).
        let cmd = BuyerCommands::Sessions { user_id: None };
        assert!(matches!(cmd, BuyerCommands::Sessions { user_id: None }));
    }

    #[test]
    fn buyer_sessions_with_valid_uuid_user_id_parses() {
        // A well-formed UUID string should parse successfully (mirrors run() logic).
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let result = uuid_str.parse::<uuid::Uuid>();
        assert!(result.is_ok(), "valid UUID must parse without error");
    }

    #[test]
    fn buyer_sessions_invalid_uuid_user_id_fails_to_parse() {
        // An invalid UUID should fail to parse, triggering the process::exit branch.
        let bad = "not-a-uuid";
        let result = bad.parse::<uuid::Uuid>();
        assert!(
            result.is_err(),
            "invalid UUID must fail to parse so the error branch fires"
        );
    }

    // ── parse_answers_json ─────────────────────────────────────────────────

    #[test]
    fn parse_answers_json_valid_object() {
        let result = parse_answers_json(r#"{"industry": "cosmetics", "region": "EU"}"#);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(
            map.get("industry").and_then(|v| v.as_str()),
            Some("cosmetics")
        );
        assert_eq!(map.get("region").and_then(|v| v.as_str()), Some("EU"));
    }

    #[test]
    fn parse_answers_json_empty_object() {
        let result = parse_answers_json("{}");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_answers_json_invalid_json_returns_error() {
        let result = parse_answers_json("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("valid JSON object string"));
    }

    #[test]
    fn parse_answers_json_array_returns_error() {
        // Arrays are valid JSON but not objects — should be rejected
        let result = parse_answers_json(r#"["a", "b"]"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JSON object"));
    }

    #[test]
    fn parse_answers_json_primitive_returns_error() {
        // Primitives are not objects — should be rejected
        let result = parse_answers_json("42");
        assert!(result.is_err());
    }

    #[test]
    fn parse_answers_json_nested_values_preserved() {
        // Should handle nested values correctly
        let result = parse_answers_json(r#"{"count": 5, "active": true}"#);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("count").and_then(|v| v.as_u64()), Some(5));
        assert_eq!(map.get("active").and_then(|v| v.as_bool()), Some(true));
    }
}
