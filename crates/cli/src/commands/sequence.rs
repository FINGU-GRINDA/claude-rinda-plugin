// CLI commands for sequence operations.

use std::process;

use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api_helper::{
    exit_api_error, get_authenticated_client, print_json, require_workspace_id,
};

#[derive(Debug, Args)]
pub struct SequenceArgs {
    #[command(subcommand)]
    pub command: SequenceCommands,
}

#[derive(Debug, Subcommand)]
pub enum SequenceCommands {
    /// Create a new email sequence
    Create {
        /// Sequence name
        #[arg(long)]
        name: String,

        /// Sequence type (e.g. "email", "linkedin")
        #[arg(long, name = "type")]
        seq_type: Option<String>,

        /// JSON array of steps (e.g. '[{"delay":1,"template":"intro"}]')
        #[arg(long)]
        steps: Option<String>,
    },

    /// List existing sequences for the workspace
    List {
        /// Maximum number of results to return
        #[arg(long)]
        limit: Option<String>,

        /// Offset for pagination
        #[arg(long)]
        offset: Option<String>,
    },

    /// AI-generate email steps for a sequence
    Generate {
        /// Sequence ID (UUID)
        #[arg(long)]
        id: String,
    },

    /// Add a contact (buyer/lead) to a sequence
    AddContact {
        /// Sequence ID (UUID)
        #[arg(long)]
        sequence_id: String,

        /// Buyer / lead ID to add
        #[arg(long)]
        buyer_id: String,
    },
}

pub async fn run(args: SequenceArgs) {
    let (client, creds) = get_authenticated_client().await;

    match args.command {
        SequenceCommands::Create {
            name,
            seq_type: _seq_type,
            steps: _steps,
        } => {
            let workspace_id = require_workspace_id(&creds);
            let name_typed: rinda_sdk::types::PostApiV1SequencesBodyName =
                name.parse().unwrap_or_else(|e| {
                    eprintln!("Invalid sequence name: {e}");
                    process::exit(1);
                });

            let body = rinda_sdk::types::PostApiV1SequencesBody {
                name: name_typed,
                workspace_id,
                created_by: None,
                customer_group_id: None,
                customer_group_ids: Vec::new(),
                description: None,
                memo: None,
                personalization_config: None,
                personalization_mode: None,
                status: None,
                timezone_mode: None,
                workflow_data: None,
            };

            match client.post_api_v1_sequences(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("sequence create failed", e),
            }
        }

        SequenceCommands::List { limit, offset } => {
            let _workspace_id = require_workspace_id(&creds);

            match client
                .get_api_v1_sequences(limit.as_deref(), offset.as_deref())
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("sequence list failed", e),
            }
        }

        SequenceCommands::Generate { id } => {
            let uuid = match id.parse::<Uuid>() {
                Ok(u) => u,
                Err(_) => {
                    eprintln!("Invalid sequence ID — must be a valid UUID");
                    process::exit(1);
                }
            };

            let body = rinda_sdk::types::PostApiV1SequencesByIdGenerateBody {
                user_email_account_id: None,
            };

            match client
                .post_api_v1_sequences_by_id_generate(&uuid, &body)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("sequence generate failed", e),
            }
        }

        SequenceCommands::AddContact {
            sequence_id,
            buyer_id,
        } => {
            let uuid = match sequence_id.parse::<Uuid>() {
                Ok(u) => u,
                Err(_) => {
                    eprintln!("Invalid sequence ID — must be a valid UUID");
                    process::exit(1);
                }
            };

            let lead_id = match buyer_id.parse::<Uuid>() {
                Ok(u) => u,
                Err(_) => {
                    eprintln!("Invalid buyer ID — must be a valid UUID");
                    process::exit(1);
                }
            };

            // user_email_account_id is required but we don't have it from CLI args;
            // use a nil UUID as placeholder (the API will use the default account).
            let body = rinda_sdk::types::PostApiV1SequencesByIdEnrollmentsBody {
                lead_id,
                user_email_account_id: Uuid::nil(),
                enrolled_by: None,
                status: None,
            };

            match client
                .post_api_v1_sequences_by_id_enrollments(&uuid, &body)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("sequence add-contact failed", e),
            }
        }
    }

    process::exit(0);
}
