// Local HTTP server + browser open for OAuth callback flow.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::Query;
use axum::response::Html;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use rinda_common::error::{Result, RindaError};
use rinda_sdk::apis::configuration::Configuration;
use rinda_sdk::apis::default_api;

use crate::credentials::Credentials;

/// Build an SDK configuration with optional bearer token.
pub fn sdk_config(bearer_token: Option<&str>) -> Configuration {
    let mut config = Configuration::new();
    config.bearer_access_token = bearer_token.map(|t| t.to_string());
    config
}

/// Run the full OAuth flow: open browser, wait for callback, exchange code, fetch profile.
pub async fn run_oauth_flow() -> Result<Credentials> {
    // Channel to receive the OAuth code from the callback handler.
    let (tx, rx) = oneshot::channel::<String>();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    // Build the axum router.
    let tx_clone = Arc::clone(&tx);
    let app = Router::new().route(
        "/callback",
        get(move |Query(params): Query<HashMap<String, String>>| {
            let tx_inner = Arc::clone(&tx_clone);
            async move {
                if let Some(code) = params.get("code") {
                    let mut guard = tx_inner.lock().await;
                    if let Some(sender) = guard.take() {
                        let _ = sender.send(code.clone());
                    }
                }
                Html(
                    "<html><body><h2>Login successful!</h2>\
                     <p>You can close this tab and return to the terminal.</p></body></html>",
                )
            }
        }),
    );

    // Bind to 127.0.0.1:9876.
    let listener = TcpListener::bind("127.0.0.1:9876").await.map_err(|e| {
        RindaError::Auth(format!(
            "Cannot bind to port 9876: {e}. Is another rinda process running?"
        ))
    })?;

    // Get the Google auth URL.
    let config = sdk_config(None);
    let google_url = format!(
        "{}/api/v1/auth/google?redirect_uri={}",
        config.base_path,
        urlencoding::encode("http://localhost:9876/callback")
    );

    // Open the browser.
    println!("Opening browser for Google login...");
    if let Err(e) = open::that(&google_url) {
        eprintln!("Could not open browser automatically: {e}");
        println!("Please open this URL manually:\n  {google_url}");
    }

    // Race: serve requests vs. 120-second timeout.
    let code = tokio::time::timeout(Duration::from_secs(120), async move {
        // Run the server until the code arrives.
        tokio::select! {
            result = axum::serve(listener, app) => {
                match result {
                    Ok(()) => Err(RindaError::Auth("Server exited before receiving callback".into())),
                    Err(e) => Err(RindaError::Io(e)),
                }
            }
            code = rx => {
                code.map_err(|_| RindaError::Auth("Callback channel closed unexpectedly".into()))
            }
        }
    })
    .await
    .map_err(|_| RindaError::Auth("Timed out waiting for OAuth callback (120 s). Please try again.".into()))??;

    // Exchange the code for tokens.
    let mut body = HashMap::new();
    body.insert("code".to_string(), serde_json::Value::String(code));

    let callback_resp = default_api::post_api_v1_auth_google_callback(&config, body)
        .await
        .map_err(|e| RindaError::Auth(format!("Token exchange failed: {e}")))?;

    let access_token = callback_resp
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RindaError::Auth("No access token in callback response".into()))?
        .to_string();

    let refresh_token = callback_resp
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // Compute expiry: now + 1 hour in milliseconds.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let expires_at = now_ms + 3_600_000;

    // Fetch user profile with the new token.
    let authed_config = sdk_config(Some(&access_token));

    let profile = default_api::get_api_v1_auth_me(&authed_config)
        .await
        .map_err(|e| RindaError::Auth(format!("Failed to fetch user profile: {e}")))?;

    let email = profile
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let user_id = profile
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let workspace_id = profile
        .get("workspaceId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(Credentials {
        access_token,
        refresh_token,
        expires_at,
        workspace_id,
        user_id,
        email,
    })
}
