// CLI commands for email operations.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{
    exit_api_error, get_authenticated_client, print_json, require_user_id, require_workspace_id,
};

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
    let (client, creds) = get_authenticated_client().await;

    match args.command {
        EmailCommands::Send { to, subject, body } => {
            let user_id = require_user_id(&creds);
            let workspace_id = require_workspace_id(&creds);
            let subject_typed: rinda_sdk::types::PostApiV1EmailsSendBodySubject =
                subject.parse().unwrap_or_else(|e| {
                    eprintln!("Invalid subject: {e}");
                    process::exit(1);
                });

            let req_body = rinda_sdk::types::PostApiV1EmailsSendBody {
                to_email: to,
                subject: subject_typed,
                body_text: Some(body),
                body_html: None,
                user_id,
                workspace_id,
                bcc_emails: Vec::new(),
                cc_emails: Vec::new(),
                files: Vec::new(),
                from_name: None,
                in_reply_to: None,
                include_signature: None,
                lead_id: None,
                references: None,
                reply_to: None,
                scheduled_at: None,
                sequence_id: None,
                step_id: None,
            };

            match client.post_api_v1_emails_send(&req_body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("email send failed", e),
            }
        }
    }

    process::exit(0);
}
