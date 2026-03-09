// CLI commands for order history.
//
// Note: The RINDA API does not have a dedicated /orders endpoint. This command
// uses the leads/search endpoint as the closest available approximation.
// buyer_id maps to the `search` query param and days_inactive maps to `updatedBefore`.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

#[derive(Debug, Args)]
pub struct OrderArgs {
    #[command(subcommand)]
    pub command: OrderCommands,
}

#[derive(Debug, Subcommand)]
pub enum OrderCommands {
    /// Retrieve order history (uses leads/search as approximation; no orders API exists)
    History {
        /// Filter by buyer/lead ID or name
        #[arg(long)]
        buyer_id: Option<String>,

        /// Only include leads inactive for at least this many days
        #[arg(long)]
        days_inactive: Option<u32>,
    },
}

pub async fn run(args: OrderArgs) {
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        OrderCommands::History {
            buyer_id,
            days_inactive,
        } => {
            // Compute updatedBefore if days_inactive is specified.
            let updated_before_str = days_inactive.map(|days| {
                let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
                cutoff.format("%Y-%m-%d").to_string()
            });

            let search_ref = buyer_id.as_deref();
            let updated_before_ref = updated_before_str.as_deref();

            match client
                .get_api_v1_leads_search(
                    None,               // business_type
                    None,               // city
                    None,               // country
                    None,               // created_after
                    None,               // created_before
                    None,               // created_by_ids
                    None,               // customer_group_id
                    None,               // filters
                    None,               // lead_status
                    None,               // limit
                    None,               // offset
                    search_ref,         // search (buyer_id as name/id filter)
                    None,               // search_type
                    None,               // sort_field
                    None,               // sort_order
                    None,               // updated_after
                    updated_before_ref, // updated_before (days_inactive cutoff)
                    None,               // workspace_ids
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("order history failed", e),
            }
        }
    }

    process::exit(0);
}
