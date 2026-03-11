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

        /// Sequence type forwarded as description (e.g. "email", "linkedin")
        #[arg(long, name = "type")]
        seq_type: Option<String>,

        /// JSON array of steps (e.g. '[{"delay":1,"subject":"Hello","body":"..."}]')
        /// Each element may include: delay/delayDays, subject/emailSubject,
        /// body/emailBodyText, bodyHtml/emailBodyHtml, stepOrder.
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
            seq_type,
            steps,
        } => {
            let workspace_id = require_workspace_id(&creds);
            let name_typed: rinda_sdk::types::PostApiV1SequencesBodyName =
                name.parse().unwrap_or_else(|e| {
                    eprintln!("Invalid sequence name: {e}");
                    process::exit(1);
                });

            // Forward seq_type via the description field.
            let description = seq_type.map(|t| format!("Type: {t}"));

            let body = rinda_sdk::types::PostApiV1SequencesBody {
                name: name_typed,
                workspace_id,
                created_by: None,
                customer_group_id: None,
                customer_group_ids: Vec::new(),
                description,
                memo: None,
                personalization_config: None,
                personalization_mode: None,
                status: None,
                timezone_mode: None,
                workflow_data: None,
            };

            let sequence_resp = match client.post_api_v1_sequences(&body).await {
                Ok(resp) => resp.into_inner(),
                Err(e) => exit_api_error("sequence create failed", e),
            };

            // Extract the created sequence ID for step creation and verification.
            let sequence_id_str = sequence_resp
                .get("id")
                .or_else(|| sequence_resp.get("data").and_then(|d| d.get("id")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if sequence_id_str.is_none() {
                eprintln!(
                    "Warning: sequence ID not found in API response — verify creation with: rinda-cli sequence list"
                );
            }

            // If no steps provided, print the create response and exit.
            let Some(steps_json) = steps else {
                print_json(&sequence_resp);
                if let Some(id) = &sequence_id_str {
                    eprintln!("Sequence created with ID: {id}");
                    eprintln!(
                        "Tip: run `rinda-cli sequence generate --id {id}` to AI-generate email steps."
                    );
                }
                process::exit(0);
            };

            // Parse the sequence ID UUID needed to create steps.
            let seq_uuid = match sequence_id_str.as_deref().unwrap_or("").parse::<Uuid>() {
                Ok(u) => u,
                Err(_) => {
                    eprintln!(
                        "Sequence was created but its ID could not be parsed as UUID; cannot add steps."
                    );
                    eprintln!("Verify with: rinda-cli sequence list");
                    print_json(&sequence_resp);
                    process::exit(1);
                }
            };

            // Parse the JSON steps array.
            let parsed_steps: serde_json::Value =
                serde_json::from_str(&steps_json).unwrap_or_else(|e| {
                    eprintln!("Invalid steps JSON: {e}");
                    process::exit(1);
                });

            let step_array = match parsed_steps.as_array() {
                Some(a) => a.clone(),
                None => {
                    eprintln!("--steps must be a JSON array.");
                    process::exit(1);
                }
            };

            println!(
                "Sequence created with ID: {}",
                sequence_id_str.as_deref().unwrap_or("unknown")
            );
            println!("Creating {} step(s)...", step_array.len());

            let mut all_ok = true;
            for (idx, step) in step_array.iter().enumerate() {
                let step_order_num = step
                    .get("stepOrder")
                    .or_else(|| step.get("step_order"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or((idx + 1) as f64);

                let delay_days_num = step
                    .get("delayDays")
                    .or_else(|| step.get("delay"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);

                let subject_str = step
                    .get("emailSubject")
                    .or_else(|| step.get("subject"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no subject)");

                let body_text = step
                    .get("emailBodyText")
                    .or_else(|| step.get("body"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let body_html = step
                    .get("emailBodyHtml")
                    .or_else(|| step.get("bodyHtml"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let subject_typed: rinda_sdk::types::PostApiV1SequencesByIdStepsBodyEmailSubject =
                    subject_str.parse().unwrap_or_else(|e| {
                        eprintln!("Step {}: invalid email subject: {e}", idx + 1);
                        process::exit(1);
                    });

                let step_body = rinda_sdk::types::PostApiV1SequencesByIdStepsBody {
                    delay_days: rinda_sdk::types::PostApiV1SequencesByIdStepsBodyDelayDays::Number(
                        delay_days_num,
                    ),
                    email_body_html: body_html,
                    email_body_text: body_text,
                    email_subject: subject_typed,
                    email_template_id: None,
                    files: Vec::new(),
                    generation_source: None,
                    original_email_body_html: None,
                    original_email_body_text: None,
                    original_email_subject: None,
                    original_language: None,
                    scheduled_hour: None,
                    scheduled_minute: None,
                    step_order: rinda_sdk::types::PostApiV1SequencesByIdStepsBodyStepOrder::Number(
                        step_order_num,
                    ),
                    timezone: None,
                    translated_language: None,
                };

                match client
                    .post_api_v1_sequences_by_id_steps(&seq_uuid, &step_body)
                    .await
                {
                    Ok(resp) => {
                        println!("Step {} created.", idx + 1);
                        print_json(&resp.into_inner());
                    }
                    Err(e) => {
                        eprintln!("Step {} failed: {e}", idx + 1);
                        all_ok = false;
                    }
                }
            }

            if !all_ok {
                eprintln!("One or more steps failed. Verify with: rinda-cli sequence list");
                process::exit(1);
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
