// CLI commands for email reply operations.
//
// Note: the --hours flag is accepted for interface compatibility with the plugin spec,
// but the backend API does not support filtering replies by hours. Use --limit instead
// to control how many replies are returned.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

#[derive(Debug, Args)]
pub struct ReplyArgs {
    #[command(subcommand)]
    pub command: ReplyCommands,
}

#[derive(Debug, Subcommand)]
pub enum ReplyCommands {
    /// Check recent email replies
    Check {
        /// Number of hours to look back (accepted for compatibility; not supported by API)
        #[arg(long)]
        hours: Option<u32>,

        /// Maximum number of replies to return
        #[arg(long, default_value = "50")]
        limit: u32,
    },
}

pub async fn run(args: ReplyArgs) {
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        ReplyCommands::Check { hours: _, limit } => {
            let limit_str = limit.to_string();

            match client
                .get_api_v1_email_replies(
                    None,             // email_account_id
                    None,             // is_read
                    Some(&limit_str), // limit
                    None,             // offset
                    None,             // search
                    None,             // sentiment
                    None,             // workspace_id
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("reply check failed", e),
            }
        }
    }

    process::exit(0);
}
