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

use rinda_common::config::base_url;
use rinda_common::credentials::Credentials;
use rinda_common::error::{Result, RindaError};

/// Build a progenitor SDK client with an optional bearer token.
/// When `bearer_token` is `None`, no Authorization header is added.
pub fn sdk_client(bearer_token: Option<&str>) -> rinda_sdk::Client {
    if let Some(token) = bearer_token {
        let auth_value = format!("Bearer {token}");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_value)
                .expect("Bearer token contains invalid header characters"),
        );
        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build reqwest client");
        rinda_sdk::Client::new_with_client(base_url(), reqwest_client)
    } else {
        rinda_sdk::Client::new(base_url())
    }
}

/// Run the full OAuth flow: open browser, wait for callback, exchange code, fetch profile.
#[allow(dead_code)]
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

    // Build the Google auth URL.
    let google_url = format!(
        "{}/api/v1/auth/google?redirect_uri={}",
        base_url(),
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
    let body = rinda_sdk::types::PostApiV1AuthGoogleCallbackBody {
        code,
        country: None,
        experience: None,
        industry: None,
        invite_code: None,
        lang: None,
        marketing_email_consented: None,
        state: None,
        target: None,
        turnstile_token: None,
        utm_campaign: None,
        utm_medium: None,
        utm_source: None,
    };

    let client = sdk_client(None);
    let callback_resp = client
        .post_api_v1_auth_google_callback(&body)
        .await
        .map_err(|e| RindaError::Auth(format!("Token exchange failed: {e}")))?
        .into_inner();

    // The API wraps the payload in a `data` envelope.
    let data = callback_resp
        .get("data")
        .and_then(|v| v.as_object())
        .unwrap_or(&callback_resp);

    let access_token = data
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RindaError::Auth("No access token in callback response".into()))?
        .to_string();

    let refresh_token = data
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
    let authed_client = sdk_client(Some(&access_token));
    let profile_resp = authed_client
        .get_api_v1_auth_me()
        .await
        .map_err(|e| RindaError::Auth(format!("Failed to fetch user profile: {e}")))?
        .into_inner();

    // Response shape: { data: { user: { id, email, ... } } }
    let user = profile_resp
        .get("data")
        .and_then(|d| d.get("user"))
        .cloned()
        .unwrap_or(serde_json::Value::Object(profile_resp.clone()));

    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let user_id = user
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    // Try workspaceId from /auth/me first (may not be present in newer API versions).
    let mut workspace_id = user
        .get("workspaceId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // If /auth/me didn't include workspaceId, fetch from /workspaces/user.
    if workspace_id.is_empty()
        && let Ok(ws_resp) = authed_client.get_api_v1_workspaces_user().await
    {
        let ws_data = ws_resp.into_inner();
        workspace_id = ws_data
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|ws| ws.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
    }

    Ok(Credentials {
        access_token,
        refresh_token,
        expires_at,
        workspace_id,
        user_id,
        email,
    })
}
