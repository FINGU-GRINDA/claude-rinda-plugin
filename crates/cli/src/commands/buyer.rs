// CLI commands for buyer (lead) operations.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

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
            let workspace_id = creds.workspace_id.parse::<uuid::Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid workspace ID in credentials");
                process::exit(1);
            });

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
