// CLI commands for email operations.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

#[derive(Debug, Args)]
pub struct EmailArgs {
    #[command(subcommand)]
    pub command: EmailCommands,
}

#[derive(Debug, Subcommand)]
pub enum EmailCommands {
    /// Send an email
    Send {
        /// Recipient email address
        #[arg(long)]
        to: String,

        /// Email subject
        #[arg(long)]
        subject: String,

        /// Email body text
        #[arg(long)]
        body: String,
    },
}

pub async fn run(args: EmailArgs) {
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        EmailCommands::Send { to, subject, body } => {
            let mut req_body = serde_json::Map::new();
            req_body.insert("to".to_string(), serde_json::Value::String(to));
            req_body.insert("subject".to_string(), serde_json::Value::String(subject));
            req_body.insert("body".to_string(), serde_json::Value::String(body));

            match client.post_api_v1_emails_send(&req_body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("email send failed", e),
            }
        }
    }

    process::exit(0);
}
