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
use rinda_sdk::RindaClient;
use rinda_sdk::models::auth::GoogleCallbackRequest;

use crate::credentials::Credentials;

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
    let client = RindaClient::new();
    let google_url = client.google_auth_url();

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
    let callback_resp = client
        .google_callback(&GoogleCallbackRequest { code })
        .await
        .map_err(|e| RindaError::Auth(format!("Token exchange failed: {e}")))?;

    let access_token = callback_resp
        .token
        .ok_or_else(|| RindaError::Auth("No access token in callback response".into()))?;

    let refresh_token = callback_resp.refresh_token.unwrap_or_default();

    // Compute expiry: now + 1 hour in milliseconds.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let expires_at = now_ms + 3_600_000;

    // Fetch user profile with the new token.
    let mut authed_client = RindaClient::new();
    authed_client.set_access_token(&access_token);

    let profile = authed_client
        .me()
        .await
        .map_err(|e| RindaError::Auth(format!("Failed to fetch user profile: {e}")))?;

    let email = profile.email.unwrap_or_default();
    let user_id = profile.id.unwrap_or_default();
    let workspace_id = profile.workspace_id.unwrap_or_default();

    Ok(Credentials {
        access_token,
        refresh_token,
        expires_at,
        workspace_id,
        user_id,
        email,
    })
}
