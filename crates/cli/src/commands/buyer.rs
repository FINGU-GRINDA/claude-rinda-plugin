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
    }

    process::exit(0);
}
