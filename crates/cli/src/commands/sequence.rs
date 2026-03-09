// CLI commands for sequence operations.

use std::process;

use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

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
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        SequenceCommands::Create {
            name,
            seq_type,
            steps,
        } => {
            let mut body = serde_json::Map::new();
            body.insert("name".to_string(), serde_json::Value::String(name));

            if let Some(t) = seq_type {
                body.insert("type".to_string(), serde_json::Value::String(t));
            }
            if let Some(s) = steps {
                match serde_json::from_str::<serde_json::Value>(&s) {
                    Ok(steps_val) => {
                        body.insert("steps".to_string(), steps_val);
                    }
                    Err(e) => {
                        eprintln!("Invalid JSON for --steps: {e}");
                        process::exit(1);
                    }
                }
            }

            match client.post_api_v1_sequences(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("sequence create failed", e),
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

            let mut body = serde_json::Map::new();
            body.insert("leadId".to_string(), serde_json::Value::String(buyer_id));

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
