// CLI commands for buyer (lead) operations.

use std::process;

use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api_helper::{
    exit_api_error, get_authenticated_client, print_json, require_workspace_id,
};
use rinda_common::config::base_url;

#[derive(Debug, Args)]
pub struct BuyerArgs {
    #[command(subcommand)]
    pub command: BuyerCommands,
}

#[derive(Debug, Subcommand)]
pub enum BuyerCommands {
    /// Search for buyers (leads) matching criteria
    Search {
        /// Industry filter (e.g. "manufacturing")
        #[arg(long)]
        industry: Option<String>,

        /// Comma-separated list of country codes (e.g. "US,DE")
        #[arg(long)]
        countries: Option<String>,

        /// Buyer type filter (e.g. "importer")
        #[arg(long)]
        buyer_type: Option<String>,

        /// Minimum annual revenue in USD
        #[arg(long)]
        min_revenue: Option<f64>,

        /// Maximum number of results to return
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Check the status of an async search session
    Status {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,
    },

    /// View the results of a completed search session
    Results {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,
    },

    /// Select leads from discovery results to save them
    Select {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,

        /// Selected recommendation ID
        #[arg(long)]
        recommendation_id: String,
    },

    /// Enrich a buyer (lead) with additional data
    Enrich {
        /// Buyer ID (lead ID) to enrich
        #[arg(long)]
        buyer_id: String,
    },

    /// Retrieve clarification questions for a session in waiting_clarification status
    Messages {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,
    },

    /// Submit answers to clarification questions and resume the search session
    Clarify {
        /// Session ID (UUID) from the search request
        #[arg(long)]
        session_id: String,

        /// Answers as a JSON object string (e.g. '{"field": "value"}')
        #[arg(long)]
        answers: String,
    },

    /// List all past search sessions for the current workspace
    Sessions {
        /// Optional user ID filter (UUID)
        #[arg(long)]
        user_id: Option<String>,
    },
}

pub async fn run(args: BuyerArgs) {
    let (client, creds) = get_authenticated_client().await;

    match args.command {
        BuyerCommands::Search {
            industry,
            countries,
            buyer_type,
            min_revenue,
            limit,
        } => {
            let workspace_id = require_workspace_id(&creds);

            // Build natural-language query from filters.
            let mut parts: Vec<String> = Vec::new();
            if let Some(ind) = industry {
                parts.push(ind);
            }
            if let Some(c) = countries {
                parts.push(format!("countries:{c}"));
            }
            if let Some(bt) = buyer_type {
                parts.push(format!("type:{bt}"));
            }
            if let Some(rev) = min_revenue {
                parts.push(format!("min_revenue:{rev}"));
            }
            parts.push(format!("limit:{limit}"));
            let query = if parts.len() == 1 {
                // Only the limit part — use a default.
                "buyer search".to_string()
            } else {
                parts.join(" ")
            };

            let body = serde_json::json!({
                "query": query,
                "workspaceId": workspace_id,
                "useAutoTimeout": true,
            });

            match sse_search(&creds.access_token, &body).await {
                Ok(result) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                    );
                }
                Err(e) => {
                    eprintln!("buyer search failed: {e}");
                    process::exit(1);
                }
            }
        }

        BuyerCommands::Status { session_id } => {
            let uuid = session_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid session ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_lead_discovery_db_sessions_by_session_id(&uuid)
                .await
            {
                Ok(resp) => {
                    let json = resp.into_inner();

                    // Try to extract human-readable fields from the response.
                    let data = json.get("data").and_then(|v| v.as_object());
                    let session = data
                        .and_then(|d| d.get("session"))
                        .and_then(|v| v.as_object());

                    if let Some(s) = session {
                        let status = s
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let progress = s
                            .get("progress")
                            .and_then(|v| v.as_f64())
                            .map(|p| format!("{:.0}", p))
                            .unwrap_or_else(|| "?".to_string());
                        let total_count = s
                            .get("totalCount")
                            .and_then(|v| v.as_u64())
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "?".to_string());
                        let query = s.get("query").and_then(|v| v.as_str()).unwrap_or("-");
                        let job_phase = s.get("jobPhase").and_then(|v| v.as_str()).unwrap_or("-");
                        let message = s.get("message").and_then(|v| v.as_str()).unwrap_or("-");

                        println!("Session: {session_id}");
                        println!("Query:    {query}");
                        println!("Status:   {status} ({progress}%)");
                        println!("Phase:    {job_phase}");
                        println!("Results:  {total_count}");
                        println!("Message:  {message}");

                        if status == "waiting_clarification" {
                            println!();
                            println!(
                                "Hint: Run `rinda-cli buyer messages --session-id {session_id}` to see questions,"
                            );
                            println!(
                                "      then `rinda-cli buyer clarify --session-id {session_id} --answers '{{...}}'` to respond."
                            );
                        }
                    } else {
                        // Fallback: print raw JSON if structure is unexpected.
                        print_json(&json);
                    }
                }
                Err(e) => exit_api_error("buyer status failed", e),
            }
        }

        BuyerCommands::Results { session_id } => {
            let uuid = session_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid session ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_lead_discovery_db_sessions_by_session_id_results(&uuid)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer results failed", e),
            }
        }

        BuyerCommands::Select {
            session_id,
            recommendation_id,
        } => {
            let workspace_id = require_workspace_id(&creds);

            let body = rinda_sdk::types::PostApiV1LeadDiscoverySelectBody {
                session_id,
                selected_recommendation_id: recommendation_id,
                workspace_id,
            };

            match client.post_api_v1_lead_discovery_select(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer select failed", e),
            }
        }

        BuyerCommands::Enrich { buyer_id } => {
            let body = rinda_sdk::types::PostApiV1LeadDiscoveryEnrichBody {
                website_url: buyer_id,
                workspace_id: creds.workspace_id.clone(),
            };

            match client.post_api_v1_lead_discovery_enrich(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer enrich failed", e),
            }
        }

        BuyerCommands::Messages { session_id } => {
            let uuid = session_id.parse::<Uuid>().unwrap_or_else(|_| {
                eprintln!("Invalid session ID — must be a valid UUID");
                process::exit(1);
            });

            match client
                .get_api_v1_lead_discovery_db_sessions_by_session_id_messages(&uuid)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer messages failed", e),
            }
        }

        BuyerCommands::Sessions { user_id } => {
            let workspace_id = require_workspace_id(&creds);
            let user_uuid = user_id.map(|id| {
                id.parse::<Uuid>().unwrap_or_else(|_| {
                    eprintln!("Invalid user ID — must be a valid UUID");
                    process::exit(1);
                })
            });
            match client
                .get_api_v1_lead_discovery_db_sessions(user_uuid.as_ref(), &workspace_id)
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer sessions failed", e),
            }
        }

        BuyerCommands::Clarify {
            session_id,
            answers,
        } => {
            let workspace_id = require_workspace_id(&creds);

            let answers_map = parse_answers_json(&answers).unwrap_or_else(|e| {
                eprintln!("{e}");
                process::exit(1);
            });

            let body = rinda_sdk::types::PostApiV1LeadDiscoveryClarifyBody {
                session_id,
                answers: answers_map,
                workspace_id,
            };

            match client.post_api_v1_lead_discovery_clarify(&body).await {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("buyer clarify failed", e),
            }
        }
    }

    process::exit(0);
}

/// Perform an SSE POST to `/lead-discovery/search` and collect events until
/// a session_id is obtained or the stream ends.
async fn sse_search(
    access_token: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use futures_util::StreamExt;

    let base = base_url();
    let url = format!("{base}/api/v1/lead-discovery/search");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Accept", "text/event-stream")
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {text}"));
    }

    let mut stream = resp.bytes_stream();
    let mut session_id: Option<String> = None;
    let mut last_status: Option<String> = None;
    let mut last_data: Option<serde_json::Value> = None;
    let mut buf = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream read error: {e}"))?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buf.find("\n\n") {
            let event_block = buf[..pos].to_string();
            buf = buf[pos + 2..].to_string();

            for line in event_block.lines() {
                if let Some(data_str) = line
                    .strip_prefix("data: ")
                    .or_else(|| line.strip_prefix("data:"))
                {
                    let data_str = data_str.trim();
                    if data_str == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data_str) {
                        if session_id.is_none() {
                            session_id = parsed
                                .get("sessionId")
                                .or_else(|| parsed.get("session_id"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        if let Some(s) = parsed.get("status").and_then(|v| v.as_str()) {
                            last_status = Some(s.to_string());
                        }
                        last_data = Some(parsed);
                    }
                }
            }
        }

        if session_id.is_some() && last_status.is_some() {
            break;
        }
    }

    let mut result = serde_json::json!({});
    if let Some(sid) = &session_id {
        result["sessionId"] = serde_json::json!(sid);
    }
    if let Some(st) = &last_status {
        result["status"] = serde_json::json!(st);
    }
    if let Some(data) = last_data {
        result["lastEvent"] = data;
    }

    if session_id.is_none() {
        result["warning"] = serde_json::json!(
            "No session_id received from SSE stream. The search may not have started."
        );
    } else {
        println!("Search session started: {}", session_id.as_ref().unwrap());
        println!("Use `rinda buyer status --session-id <id>` to check progress.");
    }

    Ok(result)
}

/// Parse a JSON string into a `serde_json::Map` suitable for the clarify body.
/// Returns an error message string if parsing fails or the value is not an object.
pub fn parse_answers_json(s: &str) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let parsed: serde_json::Value = serde_json::from_str(s)
        .map_err(|_| "Invalid answers — must be a valid JSON object string".to_string())?;

    parsed.as_object().cloned().ok_or_else(|| {
        "Invalid answers — must be a JSON object (not array or primitive)".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Sessions subcommand ────────────────────────────────────────────────

    #[test]
    fn buyer_sessions_no_user_id_is_valid_variant() {
        // The Sessions variant must be constructable with no user_id (optional).
        let cmd = BuyerCommands::Sessions { user_id: None };
        assert!(matches!(cmd, BuyerCommands::Sessions { user_id: None }));
    }

    #[test]
    fn buyer_sessions_with_valid_uuid_user_id_parses() {
        // A well-formed UUID string should parse successfully (mirrors run() logic).
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let result = uuid_str.parse::<uuid::Uuid>();
        assert!(result.is_ok(), "valid UUID must parse without error");
    }

    #[test]
    fn buyer_sessions_invalid_uuid_user_id_fails_to_parse() {
        // An invalid UUID should fail to parse, triggering the process::exit branch.
        let bad = "not-a-uuid";
        let result = bad.parse::<uuid::Uuid>();
        assert!(
            result.is_err(),
            "invalid UUID must fail to parse so the error branch fires"
        );
    }

    // ── parse_answers_json ─────────────────────────────────────────────────

    #[test]
    fn parse_answers_json_valid_object() {
        let result = parse_answers_json(r#"{"industry": "cosmetics", "region": "EU"}"#);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(
            map.get("industry").and_then(|v| v.as_str()),
            Some("cosmetics")
        );
        assert_eq!(map.get("region").and_then(|v| v.as_str()), Some("EU"));
    }

    #[test]
    fn parse_answers_json_empty_object() {
        let result = parse_answers_json("{}");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_answers_json_invalid_json_returns_error() {
        let result = parse_answers_json("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("valid JSON object string"));
    }

    #[test]
    fn parse_answers_json_array_returns_error() {
        // Arrays are valid JSON but not objects — should be rejected
        let result = parse_answers_json(r#"["a", "b"]"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JSON object"));
    }

    #[test]
    fn parse_answers_json_primitive_returns_error() {
        // Primitives are not objects — should be rejected
        let result = parse_answers_json("42");
        assert!(result.is_err());
    }

    #[test]
    fn parse_answers_json_nested_values_preserved() {
        // Should handle nested values correctly
        let result = parse_answers_json(r#"{"count": 5, "active": true}"#);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("count").and_then(|v| v.as_u64()), Some(5));
        assert_eq!(map.get("active").and_then(|v| v.as_bool()), Some(true));
    }
}
