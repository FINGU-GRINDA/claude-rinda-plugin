use std::process;

use clap::{Args, Subcommand};

use crate::credentials::{
    self, Credentials, extract_exp_from_jwt, is_token_valid, load_credentials, save_credentials,
};
use crate::oauth;

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Log in with Google OAuth
    Login,
    /// Check authentication status
    Status,
    /// Log out and clear credentials
    Logout,
    /// Ensure credentials are valid (refresh if needed). Called by plugin hook.
    EnsureValid,
}

pub async fn run(args: AuthArgs) {
    match args.command {
        AuthCommands::Login => match oauth::run_oauth_flow().await {
            Ok(creds) => {
                let email = creds.email.clone();
                if let Err(e) = creds.save() {
                    eprintln!("Error saving credentials: {e}");
                    process::exit(1);
                }
                println!("Logged in as {email}");
            }
            Err(e) => {
                eprintln!("Login failed: {e}");
                process::exit(1);
            }
        },

        AuthCommands::Status => {
            if !Credentials::exists() {
                println!("Not logged in. Run: rinda auth login");
                return;
            }
            match Credentials::load() {
                Ok(creds) => {
                    println!("Logged in as: {}", creds.email);
                    println!("Workspace:    {}", creds.workspace_id);
                    println!("User ID:      {}", creds.user_id);

                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);

                    let diff_ms = creds.expires_at - now_ms;
                    if diff_ms > 0 {
                        let mins = diff_ms / 60_000;
                        if mins >= 60 {
                            println!("Token:        expires in {} hour(s)", mins / 60);
                        } else {
                            println!("Token:        expires in {mins} minute(s)");
                        }
                    } else {
                        let ago_ms = now_ms - creds.expires_at;
                        let mins = ago_ms / 60_000;
                        if mins >= 60 {
                            println!("Token:        expired {} hour(s) ago", mins / 60);
                        } else {
                            println!("Token:        expired {mins} minute(s) ago");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading credentials: {e}");
                    process::exit(1);
                }
            }
        }

        AuthCommands::Logout => {
            if let Err(e) = Credentials::delete() {
                eprintln!("Error during logout: {e}");
                process::exit(1);
            }
            println!("Logged out successfully.");
        }

        AuthCommands::EnsureValid => {
            ensure_valid().await;
        }
    }
}

async fn ensure_valid() {
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

    // Fast path: token is still valid — no network call needed.
    if is_token_valid(&creds) {
        process::exit(0);
    }

    // Token is expired or expiring soon — attempt a refresh.
    let client = oauth::sdk_client(None);
    let mut body = serde_json::Map::new();
    body.insert(
        "refreshToken".to_string(),
        serde_json::Value::String(creds.refresh_token.clone()),
    );

    match client.post_api_v1_auth_refresh(&body).await {
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
                .unwrap_or(creds.refresh_token);
            let new_expires_at = extract_exp_from_jwt(&new_token);

            let new_creds = Credentials {
                access_token: new_token,
                refresh_token: new_refresh_token,
                expires_at: new_expires_at,
                workspace_id: creds.workspace_id,
                user_id: creds.user_id,
                email: creds.email,
            };

            if let Err(e) = save_credentials(&new_creds) {
                eprintln!("Failed to save credentials: {e}");
                process::exit(1);
            }

            process::exit(0);
        }
        Err(e) => {
            let err_str = format!("{e}");
            if err_str.contains("401") || err_str.contains("status code 401") {
                eprintln!("Session expired. Run: rinda auth login");
                process::exit(1);
            }
            if err_str.contains("connect") || err_str.contains("timeout") {
                eprintln!("Cannot reach RINDA API. Check your connection.");
                process::exit(1);
            }
            eprintln!("Token refresh failed: {e}");
            process::exit(1);
        }
    }
}
