use std::process;

use clap::{Args, Subcommand};

use crate::credentials::Credentials;
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
    }
}
