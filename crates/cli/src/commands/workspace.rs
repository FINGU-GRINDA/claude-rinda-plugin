// CLI commands for workspace management.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

#[derive(Debug, Args)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

#[derive(Debug, Subcommand)]
pub enum WorkspaceCommands {
    /// List workspaces the authenticated user belongs to
    List,
}

pub async fn run(args: WorkspaceArgs) {
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        WorkspaceCommands::List => match client.get_api_v1_workspaces_user().await {
            Ok(resp) => print_json(&resp.into_inner()),
            Err(e) => exit_api_error("workspace list failed", e),
        },
    }

    process::exit(0);
}
