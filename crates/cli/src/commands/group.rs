// CLI commands for customer group CRUD and member management.

use std::process;

use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api_helper::{
    exit_api_error, get_authenticated_client, print_json, require_workspace_id,
};

#[derive(Debug, Args)]
pub struct GroupArgs {
    #[command(subcommand)]
    pub command: GroupCommands,
}

#[derive(Debug, Subcommand)]
pub enum GroupCommands {
    /// Create a new customer group
    Create {
        /// Group name (required, 1-255 chars)
        #[arg(long)]
        name: String,

        /// Optional description
        #[arg(long)]
        description: Option<String>,

        /// Whether this is a dynamic group
        #[arg(long)]
        is_dynamic: bool,

        /// Enable automatic enrichment for group members
        #[arg(long)]
        auto_enrich_enabled: Option<bool>,
    },

    /// List / search customer groups for the current workspace
    List {
        /// Search query string
        #[arg(long)]
        search: Option<String>,

        /// Maximum number of results to return
        #[arg(long)]
        limit: Option<String>,

        /// Pagination offset
        #[arg(long)]
        offset: Option<String>,

        /// Filter by dynamic flag (true/false)
        #[arg(long)]
        is_dynamic: Option<String>,
    },

    /// Get a customer group by ID
    Get {
        /// Customer group UUID
        #[arg(long)]
        id: String,
    },

    /// Update a customer group
    Update {
        /// Customer group UUID
        #[arg(long)]
        id: String,

        /// New group name (required, 1-255 chars)
        #[arg(long)]
        name: String,

        /// Whether this is a dynamic group (required)
        #[arg(long)]
        is_dynamic: bool,

        /// New description
        #[arg(long)]
        description: Option<String>,
    },

    /// Delete a customer group
    Delete {
        /// Customer group UUID
        #[arg(long)]
        id: String,
    },

    /// List members of a customer group
    Members {
        /// Customer group UUID
        #[arg(long)]
        id: String,

        /// Maximum number of results to return
        #[arg(long)]
        limit: Option<String>,

        /// Pagination offset
        #[arg(long)]
        offset: Option<String>,
    },

    /// Add a lead as a member of a customer group
    AddMember {
        /// Customer group UUID
        #[arg(long)]
        id: String,

        /// Lead UUID to add
        #[arg(long)]
        lead_id: String,
    },

    /// Remove a lead from a customer group
    RemoveMember {
        /// Customer group UUID
        #[arg(long)]
        id: String,

        /// Lead UUID to remove
        #[arg(long)]
        lead_id: String,
    },

    /// List all customer groups that a given lead belongs to
    ForLead {
        /// Lead UUID
        #[arg(long)]
        lead_id: String,
    },
}

pub async fn run(args: GroupArgs) {
    let (client, creds) = get_authenticated_client().await;

    match args.command {
        GroupCommands::Create {
            name,
            description,
            is_dynamic,
            auto_enrich_enabled,
        } => {
            let workspace_id = require_workspace_id(&creds);

            let name_typed: rinda_sdk::types::PostApiV1CustomerGroupsBodyName = match name.parse() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Invalid group name: {e}");
                    process::exit(1);
                }
            };

            let body = rinda_sdk::types::PostApiV1CustomerGroupsBody {
                name: name_typed,
                workspace_id,
                description,
                is_dynamic: Some(is_dynamic),
                auto_enrich_enabled,
                created_by: None,
                criteria: None,
                csv_data: Vec::new(),
                enrich_freshness_unit: None,
                enrich_freshness_value: None,
            };

            match client.post_api_v1_customer_groups(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group create failed", e),
            }
        }

        GroupCommands::List {
            search,
            limit,
            offset,
            is_dynamic,
        } => {
            let workspace_id = require_workspace_id(&creds);
            let ws_str = workspace_id.to_string();

            match client
                .get_api_v1_customer_groups_search(
                    None,
                    is_dynamic.as_deref(),
                    limit.as_deref(),
                    offset.as_deref(),
                    search.as_deref(),
                    Some(&ws_str),
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group list failed", e),
            }
        }

        GroupCommands::Get { id } => {
            let uuid = id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid group ID — must be a valid UUID");
                process::exit(1);
            });

            match client.get_api_v1_customer_groups_by_id(&uuid).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group get failed", e),
            }
        }

        GroupCommands::Update {
            id,
            name,
            is_dynamic,
            description,
        } => {
            let uuid = id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid group ID — must be a valid UUID");
                process::exit(1);
            });

            let name_typed: rinda_sdk::types::PutApiV1CustomerGroupsByIdBodyName =
                match name.parse() {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Invalid group name: {e}");
                        process::exit(1);
                    }
                };

            let body = rinda_sdk::types::PutApiV1CustomerGroupsByIdBody {
                name: name_typed,
                is_dynamic,
                description,
                auto_enrich_enabled: None,
                criteria: None,
                enrich_freshness_unit: None,
                enrich_freshness_value: None,
            };

            match client.put_api_v1_customer_groups_by_id(&uuid, &body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group update failed", e),
            }
        }

        GroupCommands::Delete { id } => {
            let uuid = id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid group ID — must be a valid UUID");
                process::exit(1);
            });

            match client.delete_api_v1_customer_groups_by_id(&uuid).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group delete failed", e),
            }
        }

        GroupCommands::Members { id, limit, offset } => {
            let uuid = id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid group ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_customer_groups_by_id_members(
                    &uuid,
                    limit.as_deref(),
                    offset.as_deref(),
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group members failed", e),
            }
        }

        GroupCommands::AddMember { id, lead_id } => {
            let group_uuid = id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid group ID — must be a valid UUID");
                process::exit(1);
            });

            let lead_uuid = lead_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid lead ID — must be a valid UUID");
                process::exit(1);
            });

            let body = rinda_sdk::types::PostApiV1CustomerGroupsByIdMembersBody {
                lead_id: lead_uuid,
                added_by: None,
            };

            match client
                .post_api_v1_customer_groups_by_id_members(&group_uuid, &body)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group add-member failed", e),
            }
        }

        GroupCommands::RemoveMember { id, lead_id } => {
            let group_uuid = id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid group ID — must be a valid UUID");
                process::exit(1);
            });

            let lead_uuid = lead_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid lead ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .delete_api_v1_customer_groups_by_id_members_by_lead_id(&group_uuid, &lead_uuid)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group remove-member failed", e),
            }
        }

        GroupCommands::ForLead { lead_id } => {
            let lead_uuid = lead_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid lead ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_customer_groups_lead_by_lead_id_groups(&lead_uuid)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("group for-lead failed", e),
            }
        }
    }

    process::exit(0);
}
