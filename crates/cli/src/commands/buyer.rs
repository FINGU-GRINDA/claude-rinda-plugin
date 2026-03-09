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
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        BuyerCommands::Search {
            industry,
            countries,
            buyer_type,
            min_revenue,
            limit,
        } => {
            let mut body = serde_json::Map::new();

            if let Some(ind) = industry {
                body.insert("industry".to_string(), serde_json::Value::String(ind));
            }
            if let Some(c) = countries {
                // Convert comma-separated string to array.
                let country_list: Vec<serde_json::Value> = c
                    .split(',')
                    .map(|s| serde_json::Value::String(s.trim().to_string()))
                    .collect();
                body.insert(
                    "countries".to_string(),
                    serde_json::Value::Array(country_list),
                );
            }
            if let Some(bt) = buyer_type {
                body.insert("buyerType".to_string(), serde_json::Value::String(bt));
            }
            if let Some(rev) = min_revenue {
                body.insert(
                    "minRevenue".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(rev).unwrap_or(serde_json::Number::from(0)),
                    ),
                );
            }
            body.insert(
                "limit".to_string(),
                serde_json::Value::Number(serde_json::Number::from(limit)),
            );

            match client.post_api_v1_lead_discovery_search(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer search failed", e),
            }
        }

        BuyerCommands::Enrich { buyer_id } => {
            let mut body = serde_json::Map::new();
            body.insert("leadId".to_string(), serde_json::Value::String(buyer_id));

            match client.post_api_v1_lead_discovery_enrich(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer enrich failed", e),
            }
        }
    }

    process::exit(0);
}
