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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Wrapper CLI used only in tests so we can parse `workspace list` args
    /// without requiring the full top-level `Cli` struct.
    #[derive(Debug, Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: WorkspaceCommands,
    }

    /// Acceptance criteria: `rinda workspace list` should be a valid subcommand.
    /// Verifies that clap accepts "list" and maps it to `WorkspaceCommands::List`.
    #[test]
    fn workspace_list_parses_to_list_variant() {
        let cli = TestCli::try_parse_from(["workspace", "list"])
            .expect("'workspace list' should parse successfully");
        assert!(
            matches!(cli.command, WorkspaceCommands::List),
            "expected WorkspaceCommands::List variant"
        );
    }

    #[test]
    fn workspace_list_rejects_unknown_subcommand() {
        let result = TestCli::try_parse_from(["workspace", "unknown"]);
        assert!(
            result.is_err(),
            "unknown subcommand should produce a parse error"
        );
    }

    #[test]
    fn workspace_list_rejects_no_subcommand() {
        let result = TestCli::try_parse_from(["workspace"]);
        assert!(
            result.is_err(),
            "missing subcommand should produce a parse error"
        );
    }
}
