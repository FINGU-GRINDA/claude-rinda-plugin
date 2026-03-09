// Shared helpers for authenticated API calls.

use std::fmt::Display;
use std::process;

use crate::credentials::{self, Credentials, is_token_valid, load_credentials, save_credentials};
use crate::oauth;

/// Load credentials, ensure the token is valid (refreshing if needed), and build
/// an authenticated SDK client. Exits with a non-zero status on any error.
pub async fn get_authenticated_client() -> (rinda_sdk::Client, Credentials) {
    let creds = match load_credentials() {
        Ok(c) => c,
        Err(credentials::CredError::NotLoggedIn) => {
            eprintln!("Not logged in. Run: rinda auth login");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    // Fast path: token is still valid.
    if is_token_valid(&creds) {
        let client = oauth::sdk_client(Some(&creds.access_token));
        return (client, creds);
    }

    // Token expired or expiring soon — attempt a refresh.
    let refresh_client = oauth::sdk_client(None);
    let mut body = serde_json::Map::new();
    body.insert(
        "refreshToken".to_string(),
        serde_json::Value::String(creds.refresh_token.clone()),
    );

    match refresh_client.post_api_v1_auth_refresh(&body).await {
        Ok(resp) => {
            let resp = resp.into_inner();
            let new_token = match resp.get("token").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => {
                    eprintln!("Session expired. Run: rinda auth login");
                    process::exit(1);
                }
            };
            let new_refresh_token = resp
                .get("refreshToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or(creds.refresh_token.clone());
            let new_expires_at = crate::credentials::extract_exp_from_jwt(&new_token);

            let new_creds = Credentials {
                access_token: new_token.clone(),
                refresh_token: new_refresh_token,
                expires_at: new_expires_at,
                workspace_id: creds.workspace_id,
                user_id: creds.user_id,
                email: creds.email,
            };

            if let Err(e) = save_credentials(&new_creds) {
                eprintln!("Failed to save refreshed credentials: {e}");
                process::exit(1);
            }

            let client = oauth::sdk_client(Some(&new_token));
            (client, new_creds)
        }
        Err(e) => {
            let err_str = format!("{e}");
            if err_str.contains("401") || err_str.contains("status code 401") {
                eprintln!("Session expired. Run: rinda auth login");
            } else if err_str.contains("connect") || err_str.contains("timeout") {
                eprintln!("Cannot reach RINDA API. Check your connection.");
            } else {
                eprintln!("Token refresh failed: {e}");
            }
            process::exit(1);
        }
    }
}

/// Pretty-print a JSON map to stdout.
pub fn print_json(value: &serde_json::Map<String, serde_json::Value>) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("Failed to serialize response: {e}");
            process::exit(1);
        }
    }
}

/// Print an API error to stderr and exit with a non-zero status.
pub fn exit_api_error(context: &str, e: impl Display) -> ! {
    eprintln!("{context}: {e}");
    process::exit(1);
}
