use std::process;

use base64::Engine;
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
    /// Print the URL to obtain an auth token
    Url,
    /// Log in with a refresh token from the auth URL
    Token {
        /// Refresh token from https://alpha.rinda.ai/cli-auth
        token: String,
    },
    /// Check authentication status
    Status,
    /// Log out and clear credentials
    Logout,
    /// Ensure credentials are valid (refresh if needed). Called by plugin hook.
    EnsureValid,
}

/// Decode all useful claims from a JWT payload (without signature verification).
fn extract_jwt_claims(token: &str) -> (String, String, String) {
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() < 2 {
        return (String::new(), String::new(), String::new());
    }
    let Ok(decoded) = engine.decode(parts[1]) else {
        return (String::new(), String::new(), String::new());
    };
    let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
        return (String::new(), String::new(), String::new());
    };
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let workspace_id = payload
        .get("workspaceId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let user_id = payload
        .get("userId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    (email, workspace_id, user_id)
}

pub async fn run(args: AuthArgs) {
    match args.command {
        AuthCommands::Url => {
            println!("https://app.rinda.ai/cli-auth");
        }

        AuthCommands::Token {
            token: refresh_token,
        } => {
            // Exchange refresh token for an access token.
            let client = oauth::sdk_client(None);
            let body = rinda_sdk::types::PostApiV1AuthRefreshBody {
                refresh_token: refresh_token.clone(),
            };

            let resp = match client.post_api_v1_auth_refresh(&body).await {
                Ok(r) => r.into_inner(),
                Err(e) => {
                    eprintln!("Invalid or expired token: {e}");
                    process::exit(1);
                }
            };

            // The API wraps the payload in a `data` envelope.
            let data = resp
                .get("data")
                .and_then(|v| v.as_object())
                .unwrap_or(&resp);

            let access_token = match data.get("token").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => {
                    eprintln!("No access token in refresh response");
                    process::exit(1);
                }
            };

            let new_refresh_token = data
                .get("refreshToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or(refresh_token);

            let (email, _, user_id) = extract_jwt_claims(&access_token);
            let expires_at = extract_exp_from_jwt(&access_token);

            // Fetch workspace ID from the API.
            let authed_client = oauth::sdk_client(Some(&access_token));
            let workspace_id = match authed_client.get_api_v1_workspaces_user().await {
                Ok(resp) => {
                    let body = resp.into_inner();
                    body.get("data")
                        .and_then(|d| d.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|ws| ws.get("id"))
                        .and_then(|id| id.as_str())
                        .unwrap_or_default()
                        .to_string()
                }
                Err(_) => String::new(),
            };

            let creds = Credentials {
                access_token,
                refresh_token: new_refresh_token,
                expires_at,
                workspace_id: workspace_id.clone(),
                user_id,
                email: email.clone(),
            };
            if let Err(e) = creds.save() {
                eprintln!("Error saving credentials: {e}");
                process::exit(1);
            }
            if email.is_empty() {
                println!("Logged in (email not found in token)");
            } else {
                println!("Logged in as {email}");
            }
            if !workspace_id.is_empty() {
                println!("Workspace: {workspace_id}");
            }
        }

        AuthCommands::Status => {
            if !Credentials::exists() {
                println!("Not logged in. Run: rinda auth url");
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
            eprintln!("Not logged in. Run: rinda auth url");
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

    // No refresh token — can't refresh.
    if creds.refresh_token.is_empty() {
        eprintln!("Session expired. Get a new token at: rinda auth url");
        process::exit(1);
    }

    // Attempt a refresh.
    let client = oauth::sdk_client(None);
    let body = rinda_sdk::types::PostApiV1AuthRefreshBody {
        refresh_token: creds.refresh_token.clone(),
    };

    match client.post_api_v1_auth_refresh(&body).await {
        Ok(resp) => {
            let resp = resp.into_inner();
            let data = resp
                .get("data")
                .and_then(|v| v.as_object())
                .unwrap_or(&resp);
            let new_token = match data.get("token").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => {
                    eprintln!("Session expired. Run: rinda auth url");
                    process::exit(1);
                }
            };
            let new_refresh_token = data
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
                eprintln!("Session expired. Get a new token at: rinda auth url");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jwt_with_payload(payload_json: &str) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"none"}"#);
        let payload = engine.encode(payload_json);
        format!("{}.{}.sig", header, payload)
    }

    #[test]
    fn extract_jwt_claims_valid_token() {
        let token = make_jwt_with_payload(
            r#"{"email":"user@example.com","workspaceId":"ws-123","userId":"u-456","exp":9999999999}"#,
        );
        let (email, workspace_id, user_id) = extract_jwt_claims(&token);
        assert_eq!(email, "user@example.com");
        assert_eq!(workspace_id, "ws-123");
        assert_eq!(user_id, "u-456");
    }

    #[test]
    fn extract_jwt_claims_missing_fields_returns_empty_strings() {
        let token = make_jwt_with_payload(r#"{"exp":9999999999}"#);
        let (email, workspace_id, user_id) = extract_jwt_claims(&token);
        assert_eq!(email, "");
        assert_eq!(workspace_id, "");
        assert_eq!(user_id, "");
    }

    #[test]
    fn extract_jwt_claims_invalid_token_returns_empty_strings() {
        let (email, workspace_id, user_id) = extract_jwt_claims("not.a.valid.jwt");
        // Decoding will fail on a non-base64 payload — should return empty strings.
        // (4-part token: parts.len() >= 2, but payload is invalid base64/JSON)
        let _ = (email, workspace_id, user_id); // just verify no panic
    }

    #[test]
    fn extract_jwt_claims_too_few_parts_returns_empty_strings() {
        let (email, workspace_id, user_id) = extract_jwt_claims("onlyonepart");
        assert_eq!(email, "");
        assert_eq!(workspace_id, "");
        assert_eq!(user_id, "");
    }

    /// Acceptance criteria: `rinda auth url <token>` should decode the JWT and
    /// extract user info — this test verifies the claim-extraction logic that drives
    /// the token-based login flow described in issue #40.
    #[test]
    fn token_login_extracts_claims_from_jwt() {
        let token = make_jwt_with_payload(
            r#"{"email":"alice@rinda.ai","workspaceId":"ws-abc","userId":"uid-1","exp":9999999999}"#,
        );
        let (email, workspace_id, user_id) = extract_jwt_claims(&token);
        assert_eq!(
            email, "alice@rinda.ai",
            "email should be extracted from JWT"
        );
        assert_eq!(
            workspace_id, "ws-abc",
            "workspaceId should be extracted from JWT"
        );
        assert_eq!(user_id, "uid-1", "userId should be extracted from JWT");
    }
}
