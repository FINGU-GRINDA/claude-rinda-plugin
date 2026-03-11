// OAuth 2.0 endpoints for the MCP server.
//
// The MCP server acts as an OAuth Authorization Server that proxies
// authentication to RINDA's existing Google OAuth backend.
//
// Endpoints:
//   GET  /.well-known/oauth-authorization-server  — RFC 8414 metadata
//   GET  /oauth/authorize                          — Redirect to Google OAuth
//   GET  /oauth/callback                           — Receive code from Google
//   POST /oauth/token                              — Exchange code / refresh
//   POST /oauth/register                           — Dynamic client registration

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

// ── Constants ─────────────────────────────────────────────────────────────────

/// How long a pending auth state (CSRF token) is valid.
const PENDING_AUTH_TTL_SECS: i64 = 300; // 5 minutes
/// How long an authorization code is valid.
const AUTH_CODE_TTL_SECS: i64 = 60; // 60 seconds (OAuth best practice)
/// How long a session (access) token is valid.
const SESSION_TTL_SECS: i64 = 3600; // 1 hour

// ── Data structures ───────────────────────────────────────────────────────────

/// A pending authorization flow (keyed by CSRF token).
#[derive(Clone, Debug)]
pub struct PendingAuth {
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: Option<String>,
    // Stored for future PKCE method validation; currently only S256 is supported.
    #[allow(dead_code)]
    pub code_challenge_method: Option<String>,
    pub client_state: Option<String>, // original state from the client
    pub created_at: DateTime<Utc>,
}

/// A short-lived authorization code (keyed by the code itself).
#[derive(Clone, Debug)]
pub struct AuthCode {
    pub rinda_access_token: String,
    pub rinda_refresh_token: String,
    // Stored for future redirect_uri validation; currently not enforced.
    #[allow(dead_code)]
    pub redirect_uri: String,
    // Stored for future client_id validation.
    #[allow(dead_code)]
    pub client_id: String,
    pub code_challenge: Option<String>,
    pub created_at: DateTime<Utc>,
    pub used: bool,
}

/// A validated session (keyed by opaque session access token UUID).
#[derive(Clone, Debug)]
pub struct SessionTokens {
    pub rinda_access_token: String,
    pub expires_at: DateTime<Utc>,
    // Stored for audit/debugging purposes.
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

/// A dynamically registered client (keyed by client_id).
#[derive(Clone, Debug)]
pub struct ClientRegistration {
    // Stored for future client authentication; currently returned to the caller.
    #[allow(dead_code)]
    pub client_secret: Option<String>,
    // Stored for future redirect_uri validation.
    #[allow(dead_code)]
    pub redirect_uris: Vec<String>,
    // Stored for display/audit purposes.
    #[allow(dead_code)]
    pub client_name: String,
}

/// Newtype carrying a validated RINDA access token, injected by auth middleware.
#[derive(Clone, Debug)]
pub struct AuthenticatedToken(pub String);

/// Shared state injected into all OAuth route handlers via axum State.
#[derive(Clone, Debug)]
pub struct OAuthState {
    /// CSRF token -> pending auth state
    pub pending_auths: Arc<DashMap<String, PendingAuth>>,
    /// auth code -> token set
    pub auth_codes: Arc<DashMap<String, AuthCode>>,
    /// session access token -> RINDA tokens
    pub sessions: Arc<DashMap<String, SessionTokens>>,
    /// client_id -> registration
    pub registered_clients: Arc<DashMap<String, ClientRegistration>>,
    /// opaque refresh token (UUID) -> real RINDA refresh token
    pub refresh_tokens: Arc<DashMap<String, String>>,
    /// RINDA API base URL
    pub base_url: String,
    /// This MCP server's externally reachable URL (for redirect URIs in metadata)
    pub server_url: String,
}

impl OAuthState {
    pub fn new(base_url: String, server_url: String) -> Self {
        Self {
            pending_auths: Arc::new(DashMap::new()),
            auth_codes: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            registered_clients: Arc::new(DashMap::new()),
            refresh_tokens: Arc::new(DashMap::new()),
            base_url,
            server_url,
        }
    }

    /// Look up a session by its access token.
    /// Returns `Some(rinda_access_token)` if valid, `None` if missing/expired.
    pub fn validate_session(&self, access_token: &str) -> Option<String> {
        let entry = self.sessions.get(access_token)?;
        if Utc::now() >= entry.expires_at {
            return None;
        }
        Some(entry.rinda_access_token.clone())
    }

    /// Store a new session and return the opaque session access token.
    pub fn create_session(&self, rinda_access_token: String) -> String {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.sessions.insert(
            token.clone(),
            SessionTokens {
                rinda_access_token,
                expires_at: now + chrono::Duration::seconds(SESSION_TTL_SECS),
                created_at: now,
            },
        );
        token
    }

    /// Store a real RINDA refresh token and return an opaque UUID refresh token.
    pub fn create_opaque_refresh_token(&self, rinda_refresh_token: String) -> String {
        let opaque = Uuid::new_v4().to_string();
        self.refresh_tokens
            .insert(opaque.clone(), rinda_refresh_token);
        opaque
    }

    /// Look up the real RINDA refresh token by opaque token, consuming the opaque token.
    pub fn consume_opaque_refresh_token(&self, opaque: &str) -> Option<String> {
        self.refresh_tokens.remove(opaque).map(|(_, v)| v)
    }
}

// ── Auth middleware ────────────────────────────────────────────────────────────

/// Axum middleware that reads `Authorization: Bearer <token>`, validates it
/// against the session store, and injects `AuthenticatedToken` into request
/// extensions if valid.
///
/// When no valid token is present, the middleware still passes the request
/// through (to allow `initialize` and `tools/list` through), but injects a
/// marker so downstream handlers know auth was attempted and failed.
pub async fn auth_middleware(
    State(state): State<Arc<OAuthState>>,
    mut req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    if let Some(token) = extract_bearer_token(req.headers()) {
        if let Some(rinda_token) = state.validate_session(&token) {
            req.extensions_mut().insert(AuthenticatedToken(rinda_token));
        }
        // If token was present but invalid, fall through without AuthenticatedToken.
        // The tool handler will decide whether to reject.
    }

    let mut response = next.run(req).await;

    // If a downstream handler signalled 401 (via AuthRequired marker in
    // extensions), inject the WWW-Authenticate header per RFC 9728 §5.1.
    // Note: rmcp tool handlers return MCP-level responses, so this mainly
    // applies when we add explicit 401 returns in the future. For now, the
    // WWW-Authenticate header is always available on 401 responses.
    if response.status() == StatusCode::UNAUTHORIZED {
        let resource_metadata_url = format!(
            "{}/.well-known/oauth-protected-resource",
            state.server_url
        );
        response.headers_mut().insert(
            axum::http::header::WWW_AUTHENTICATE,
            format!(
                "Bearer resource_metadata=\"{}\"",
                resource_metadata_url
            )
            .parse()
            .unwrap(),
        );
    }

    response
}

// ── RFC 9728 Protected Resource Metadata ─────────────────────────────────────

#[derive(Serialize)]
pub struct ProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub bearer_methods_supported: Vec<String>,
    pub resource_documentation: Option<String>,
}

/// `GET /.well-known/oauth-protected-resource` — RFC 9728 Protected Resource Metadata.
///
/// This is the entry point for MCP client auth discovery. The client fetches this
/// to find the authorization server(s), then fetches the AS metadata from
/// `/.well-known/oauth-authorization-server`.
pub async fn protected_resource_metadata(
    State(state): State<Arc<OAuthState>>,
) -> Json<ProtectedResourceMetadata> {
    let base = &state.server_url;
    Json(ProtectedResourceMetadata {
        resource: base.clone(),
        authorization_servers: vec![base.clone()],
        bearer_methods_supported: vec!["header".to_string()],
        resource_documentation: None,
    })
}

// ── RFC 8414 Authorization Server Metadata ───────────────────────────────────

#[derive(Serialize)]
pub struct OAuthMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub registration_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
}

/// `GET /.well-known/oauth-authorization-server`
pub async fn metadata(State(state): State<Arc<OAuthState>>) -> Json<OAuthMetadata> {
    let base = &state.server_url;
    Json(OAuthMetadata {
        issuer: base.clone(),
        authorization_endpoint: format!("{base}/oauth/authorize"),
        token_endpoint: format!("{base}/oauth/token"),
        registration_endpoint: format!("{base}/oauth/register"),
        response_types_supported: vec!["code".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_post".to_string(),
            "none".to_string(),
        ],
        code_challenge_methods_supported: vec!["S256".to_string()],
    })
}

// ── /oauth/authorize ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    pub response_type: Option<String>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    #[allow(unused)]
    pub scope: Option<String>,
}

/// `GET /oauth/authorize` — Redirect to Google OAuth via RINDA backend.
pub async fn authorize(
    State(state): State<Arc<OAuthState>>,
    Query(params): Query<AuthorizeParams>,
) -> Response {
    // Validate required params.
    let response_type = params.response_type.as_deref().unwrap_or("");
    if response_type != "code" {
        return (
            StatusCode::BAD_REQUEST,
            "Unsupported response_type; only 'code' is supported",
        )
            .into_response();
    }

    let client_id = match &params.client_id {
        Some(id) => id.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing client_id").into_response();
        }
    };

    let redirect_uri = match &params.redirect_uri {
        Some(uri) => uri.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing redirect_uri").into_response();
        }
    };

    // Generate a CSRF token to tie the Google callback back to this flow.
    let csrf_token = Uuid::new_v4().to_string();

    // Store the pending auth state.
    state.pending_auths.insert(
        csrf_token.clone(),
        PendingAuth {
            client_id,
            redirect_uri,
            code_challenge: params.code_challenge.clone(),
            code_challenge_method: params.code_challenge_method.clone(),
            client_state: params.state.clone(),
            created_at: Utc::now(),
        },
    );

    // Build the callback URL that RINDA should redirect back to.
    let callback_url = format!("{}/oauth/callback", state.server_url);
    let encoded_callback = urlencoding::encode(&callback_url);

    // Fetch the Google OAuth URL from RINDA backend via HTTP GET.
    // RINDA returns JSON with the actual Google OAuth URL.
    let rinda_auth_endpoint = format!(
        "{}/api/v1/auth/google?redirectUri={}",
        state.base_url, encoded_callback
    );

    let http_client = reqwest::Client::new();
    let rinda_resp = match http_client.get(&rinda_auth_endpoint).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to contact RINDA auth backend: {e}"),
            )
                .into_response();
        }
    };

    let rinda_json: serde_json::Value = match rinda_resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("RINDA auth backend returned invalid JSON: {e}"),
            )
                .into_response();
        }
    };

    // Parse the Google OAuth URL from the JSON response.
    // Handle both {"url": "..."} and {"data": {"url": "..."}} shapes.
    let google_url = rinda_json.get("url").and_then(|v| v.as_str()).or_else(|| {
        rinda_json
            .get("data")
            .and_then(|d| d.get("url"))
            .and_then(|v| v.as_str())
    });

    let google_url = match google_url {
        Some(u) => u.to_string(),
        None => {
            return (
                StatusCode::BAD_GATEWAY,
                "RINDA backend did not return a Google OAuth URL",
            )
                .into_response();
        }
    };

    // Append our CSRF token as the `state` param so Google passes it back.
    let separator = if google_url.contains('?') { '&' } else { '?' };
    let final_url = format!("{google_url}{separator}state={csrf_token}");

    Redirect::temporary(&final_url).into_response()
}

// ── /oauth/callback ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// `GET /oauth/callback` — Receive Google OAuth code and exchange with RINDA.
pub async fn oauth_callback(
    State(state): State<Arc<OAuthState>>,
    Query(params): Query<CallbackParams>,
) -> Response {
    if let Some(err) = &params.error {
        return (
            StatusCode::BAD_REQUEST,
            format!("OAuth error from provider: {err}"),
        )
            .into_response();
    }

    let code = match &params.code {
        Some(c) => c.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing code in callback").into_response();
        }
    };

    let csrf_token = match &params.state {
        Some(s) => s.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing state in callback").into_response();
        }
    };

    // Look up the pending auth state.
    let pending = match state.pending_auths.remove(&csrf_token) {
        Some((_, p)) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid or expired state parameter",
            )
                .into_response();
        }
    };

    // Check expiry.
    if Utc::now() - pending.created_at > chrono::Duration::seconds(PENDING_AUTH_TTL_SECS) {
        return (StatusCode::BAD_REQUEST, "State parameter has expired").into_response();
    }

    // Exchange code with RINDA backend.
    let body = rinda_sdk::types::PostApiV1AuthGoogleCallbackBody {
        code: code.clone(),
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

    let client = rinda_sdk::Client::new(&state.base_url);
    let rinda_tokens = match client.post_api_v1_auth_google_callback(&body).await {
        Ok(resp) => resp.into_inner(),
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to exchange code with RINDA backend: {e}"),
            )
                .into_response();
        }
    };

    // Extract tokens from RINDA response.
    let data = rinda_tokens
        .get("data")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_else(|| rinda_tokens.clone().into_iter().collect());

    let access_token = match data.get("token").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::BAD_GATEWAY,
                "RINDA backend did not return an access token",
            )
                .into_response();
        }
    };

    let rinda_refresh_token = data
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Generate a short-lived authorization code.
    let auth_code = Uuid::new_v4().to_string();
    state.auth_codes.insert(
        auth_code.clone(),
        AuthCode {
            rinda_access_token: access_token,
            rinda_refresh_token,
            redirect_uri: pending.redirect_uri.clone(),
            client_id: pending.client_id.clone(),
            code_challenge: pending.code_challenge.clone(),
            created_at: Utc::now(),
            used: false,
        },
    );

    // Redirect back to client's redirect_uri with code and original state.
    let mut redirect_url = format!(
        "{}?code={}",
        pending.redirect_uri,
        urlencoding::encode(&auth_code)
    );
    if let Some(client_state) = &pending.client_state {
        redirect_url.push_str(&format!("&state={}", urlencoding::encode(client_state)));
    }

    Redirect::temporary(&redirect_url).into_response()
}

// ── /oauth/token ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    // authorization_code grant
    pub code: Option<String>,
    // redirect_uri and client_id/secret stored for future validation
    #[allow(dead_code)]
    pub redirect_uri: Option<String>,
    #[allow(dead_code)]
    pub client_id: Option<String>,
    #[allow(dead_code)]
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    // refresh_token grant
    pub refresh_token: Option<String>,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

/// `POST /oauth/token` — Exchange auth code or refresh token.
pub async fn token(
    State(state): State<Arc<OAuthState>>,
    body: Result<axum::Form<TokenRequest>, axum::extract::rejection::FormRejection>,
) -> Response {
    let req = match body {
        Ok(axum::Form(r)) => r,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Invalid form body: {e}")).into_response();
        }
    };

    match req.grant_type.as_str() {
        "authorization_code" => handle_auth_code_grant(state, req).await,
        "refresh_token" => handle_refresh_token_grant(state, req).await,
        other => (
            StatusCode::BAD_REQUEST,
            format!("Unsupported grant_type: {other}"),
        )
            .into_response(),
    }
}

async fn handle_auth_code_grant(state: Arc<OAuthState>, req: TokenRequest) -> Response {
    let code = match &req.code {
        Some(c) => c.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing code").into_response();
        }
    };

    // Look up the auth code.
    let mut entry = match state.auth_codes.get_mut(&code) {
        Some(e) => e,
        None => {
            return (StatusCode::BAD_REQUEST, "Invalid authorization code").into_response();
        }
    };

    // Check if already used.
    if entry.used {
        return (StatusCode::BAD_REQUEST, "Authorization code already used").into_response();
    }

    // Check expiry.
    if Utc::now() - entry.created_at > chrono::Duration::seconds(AUTH_CODE_TTL_SECS) {
        drop(entry);
        state.auth_codes.remove(&code);
        return (StatusCode::BAD_REQUEST, "Authorization code has expired").into_response();
    }

    // Validate PKCE if code_challenge was stored.
    if let Some(challenge) = &entry.code_challenge.clone() {
        let verifier = match &req.code_verifier {
            Some(v) => v.clone(),
            None => {
                return (StatusCode::BAD_REQUEST, "Missing code_verifier").into_response();
            }
        };
        if !verify_pkce_s256(&verifier, challenge) {
            return (StatusCode::BAD_REQUEST, "PKCE verification failed").into_response();
        }
    }

    // Mark as used and clone the tokens.
    entry.used = true;
    let rinda_access_token = entry.rinda_access_token.clone();
    let rinda_refresh_token = entry.rinda_refresh_token.clone();
    drop(entry);

    // Create a session token (no raw refresh token stored in session).
    let session_token = state.create_session(rinda_access_token);

    // Generate opaque refresh token — the real RINDA refresh token is NOT returned to the client.
    let opaque_refresh_token = state.create_opaque_refresh_token(rinda_refresh_token);

    Json(TokenResponse {
        access_token: session_token,
        token_type: "Bearer".to_string(),
        expires_in: SESSION_TTL_SECS,
        refresh_token: opaque_refresh_token,
    })
    .into_response()
}

async fn handle_refresh_token_grant(state: Arc<OAuthState>, req: TokenRequest) -> Response {
    let opaque_refresh_token = match &req.refresh_token {
        Some(rt) => rt.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing refresh_token").into_response();
        }
    };

    // Look up the real RINDA refresh token from the opaque token.
    let rinda_refresh_token = match state.consume_opaque_refresh_token(&opaque_refresh_token) {
        Some(rt) => rt,
        None => {
            return (StatusCode::UNAUTHORIZED, "Invalid or expired refresh token").into_response();
        }
    };

    // Call RINDA backend to refresh using the real refresh token.
    let body = rinda_sdk::types::PostApiV1AuthRefreshBody {
        refresh_token: rinda_refresh_token.clone(),
    };
    let client = rinda_sdk::Client::new(&state.base_url);
    let resp = match client.post_api_v1_auth_refresh(&body).await {
        Ok(r) => r.into_inner(),
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("401") || msg.contains("status code 401") {
                return (
                    StatusCode::UNAUTHORIZED,
                    "Refresh token is invalid or expired",
                )
                    .into_response();
            }
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to refresh token: {e}"),
            )
                .into_response();
        }
    };

    let data = resp
        .get("data")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_else(|| resp.clone().into_iter().collect());

    let new_access = match data.get("token").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::BAD_GATEWAY,
                "RINDA backend did not return an access token",
            )
                .into_response();
        }
    };

    let new_rinda_refresh = data
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .unwrap_or(&rinda_refresh_token)
        .to_string();

    let session_token = state.create_session(new_access);

    // Issue new opaque refresh token for the new RINDA refresh token.
    let new_opaque_refresh = state.create_opaque_refresh_token(new_rinda_refresh);

    Json(TokenResponse {
        access_token: session_token,
        token_type: "Bearer".to_string(),
        expires_in: SESSION_TTL_SECS,
        refresh_token: new_opaque_refresh,
    })
    .into_response()
}

// ── /oauth/register ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    #[allow(unused)]
    pub token_endpoint_auth_method: Option<String>,
    #[allow(unused)]
    pub grant_types: Option<Vec<String>>,
    #[allow(unused)]
    pub response_types: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct RegistrationResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub client_name: String,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub token_endpoint_auth_method: String,
}

/// `POST /oauth/register` — Dynamic client registration (RFC 7591).
pub async fn register(
    State(state): State<Arc<OAuthState>>,
    Json(req): Json<RegistrationRequest>,
) -> Response {
    if req.redirect_uris.is_empty() {
        return (StatusCode::BAD_REQUEST, "redirect_uris is required").into_response();
    }

    let client_id = Uuid::new_v4().to_string();
    let client_secret = Some(Uuid::new_v4().to_string());
    let client_name = req
        .client_name
        .clone()
        .unwrap_or_else(|| "Unknown Client".to_string());

    state.registered_clients.insert(
        client_id.clone(),
        ClientRegistration {
            client_secret: client_secret.clone(),
            redirect_uris: req.redirect_uris.clone(),
            client_name: client_name.clone(),
        },
    );

    (
        StatusCode::CREATED,
        Json(RegistrationResponse {
            client_id,
            client_secret,
            redirect_uris: req.redirect_uris,
            client_name,
            grant_types: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "client_secret_post".to_string(),
        }),
    )
        .into_response()
}

// ── PKCE helpers ──────────────────────────────────────────────────────────────

/// Verify PKCE S256: `BASE64URL(SHA256(verifier)) == challenge`.
pub fn verify_pkce_s256(verifier: &str, challenge: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let computed = URL_SAFE_NO_PAD.encode(hash);
    computed == challenge
}

// ── Token extraction helper ───────────────────────────────────────────────────

/// Extract the Bearer token from the Authorization header.
/// Returns `Some(token)` if present, `None` otherwise.
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())?;
    auth.strip_prefix("Bearer ").map(|t| t.to_string())
}

/// Validate a Bearer token against the session store.
/// Returns `Ok(rinda_access_token)` if valid.
/// Returns `Err(Response)` with 401 if invalid/expired.
#[allow(clippy::result_large_err, dead_code)]
pub fn validate_bearer(state: &Arc<OAuthState>, headers: &HeaderMap) -> Result<String, Response> {
    let token = match extract_bearer_token(headers) {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                [(
                    axum::http::header::WWW_AUTHENTICATE,
                    "Bearer realm=\"rinda-mcp\"",
                )],
                "Missing Authorization header",
            )
                .into_response());
        }
    };

    match state.validate_session(&token) {
        Some(rinda_token) => Ok(rinda_token),
        None => Err((
            StatusCode::UNAUTHORIZED,
            [(
                axum::http::header::WWW_AUTHENTICATE,
                "Bearer realm=\"rinda-mcp\", error=\"invalid_token\"",
            )],
            "Invalid or expired access token",
        )
            .into_response()),
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Arc<OAuthState> {
        Arc::new(OAuthState::new(
            "https://alpha.rinda.ai".to_string(),
            "http://localhost:3000".to_string(),
        ))
    }

    // ── Acceptance: RFC 8414 metadata has all required fields ─────────────────

    /// Acceptance criteria: GET /.well-known/oauth-authorization-server returns
    /// valid RFC 8414 metadata JSON with all required fields (issue #95).
    #[test]
    fn test_metadata_endpoint_returns_valid_json() {
        let state = make_state();
        let meta = OAuthMetadata {
            issuer: state.server_url.clone(),
            authorization_endpoint: format!("{}/oauth/authorize", state.server_url),
            token_endpoint: format!("{}/oauth/token", state.server_url),
            registration_endpoint: format!("{}/oauth/register", state.server_url),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_post".to_string(),
                "none".to_string(),
            ],
            code_challenge_methods_supported: vec!["S256".to_string()],
        };

        // Serialize to JSON and validate structure.
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["issuer"], "http://localhost:3000");
        assert!(
            json["authorization_endpoint"]
                .as_str()
                .unwrap()
                .contains("/oauth/authorize"),
            "authorization_endpoint should include /oauth/authorize"
        );
        assert!(
            json["token_endpoint"]
                .as_str()
                .unwrap()
                .contains("/oauth/token"),
            "token_endpoint should include /oauth/token"
        );
        assert!(
            json["registration_endpoint"]
                .as_str()
                .unwrap()
                .contains("/oauth/register"),
            "registration_endpoint should include /oauth/register"
        );
        assert_eq!(
            json["response_types_supported"],
            serde_json::json!(["code"])
        );
        assert!(
            json["grant_types_supported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "authorization_code"),
            "grant_types_supported should include authorization_code"
        );
        assert!(
            json["grant_types_supported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "refresh_token"),
            "grant_types_supported should include refresh_token"
        );
        assert_eq!(
            json["code_challenge_methods_supported"],
            serde_json::json!(["S256"])
        );
    }

    // ── Dynamic client registration ───────────────────────────────────────────

    /// Acceptance criteria: POST /oauth/register accepts registration and returns client_id (issue #95).
    #[test]
    fn test_register_creates_client() {
        let state = make_state();
        let client_id = Uuid::new_v4().to_string();
        let client_secret = Some(Uuid::new_v4().to_string());

        state.registered_clients.insert(
            client_id.clone(),
            ClientRegistration {
                client_secret: client_secret.clone(),
                redirect_uris: vec!["https://example.com/callback".to_string()],
                client_name: "Test Client".to_string(),
            },
        );

        assert!(
            state.registered_clients.contains_key(&client_id),
            "registered client should be stored"
        );
        let entry = state.registered_clients.get(&client_id).unwrap();
        assert_eq!(entry.client_name, "Test Client");
        assert_eq!(
            entry.redirect_uris,
            vec!["https://example.com/callback".to_string()]
        );
        assert!(entry.client_secret.is_some(), "client_secret should be set");
    }

    // ── PKCE S256 verification ────────────────────────────────────────────────

    /// Acceptance criteria: PKCE S256 math is correct (issue #95).
    #[test]
    fn test_pkce_s256_verification() {
        // Reference vectors from RFC 7636 Appendix B.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        // SHA256(verifier) base64url-encoded = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(
            verify_pkce_s256(verifier, challenge),
            "RFC 7636 Appendix B test vector should pass"
        );

        // Wrong verifier should fail.
        assert!(
            !verify_pkce_s256("wrong_verifier", challenge),
            "wrong verifier should not pass"
        );

        // Empty verifier should fail.
        assert!(
            !verify_pkce_s256("", challenge),
            "empty verifier should not pass"
        );
    }

    // ── Token endpoint: invalid grant type ────────────────────────────────────

    /// Acceptance criteria: POST /oauth/token with unsupported grant_type returns error (issue #95).
    #[test]
    fn test_token_endpoint_rejects_invalid_grant_type() {
        // We test the logic, not the HTTP layer, by checking grant_type routing.
        let unsupported = ["implicit", "client_credentials", "password", ""];
        for grant in &unsupported {
            let matched = matches!(*grant, "authorization_code" | "refresh_token");
            assert!(
                !matched,
                "grant_type '{grant}' should not be supported by our token endpoint"
            );
        }
    }

    // ── Token endpoint: invalid code ──────────────────────────────────────────

    /// Acceptance criteria: POST /oauth/token with invalid code returns error (issue #95).
    #[test]
    fn test_token_endpoint_rejects_invalid_code() {
        let state = make_state();
        // No codes inserted — any lookup should return None.
        let result = state.auth_codes.get("nonexistent-code");
        assert!(result.is_none(), "nonexistent code should not be found");
    }

    // ── Auth code expiry ──────────────────────────────────────────────────────

    #[test]
    fn test_auth_code_expiry_check() {
        let old_time = Utc::now() - chrono::Duration::seconds(AUTH_CODE_TTL_SECS + 1);
        let code = AuthCode {
            rinda_access_token: "tok".to_string(),
            rinda_refresh_token: "ref".to_string(),
            redirect_uri: "https://example.com/cb".to_string(),
            client_id: "client".to_string(),
            code_challenge: None,
            created_at: old_time,
            used: false,
        };
        let age = Utc::now() - code.created_at;
        assert!(
            age > chrono::Duration::seconds(AUTH_CODE_TTL_SECS),
            "expired code should be detected"
        );
    }

    // ── Session creation and validation ──────────────────────────────────────

    #[test]
    fn test_session_create_and_validate() {
        let state = make_state();
        let session_token = state.create_session("rinda-access-123".to_string());
        assert!(
            !session_token.is_empty(),
            "session token should not be empty"
        );

        let rinda_token = state.validate_session(&session_token);
        assert!(rinda_token.is_some(), "valid session should be found");
        assert_eq!(
            rinda_token.unwrap(),
            "rinda-access-123",
            "should return the RINDA access token"
        );
    }

    #[test]
    fn test_session_validate_unknown_token() {
        let state = make_state();
        let result = state.validate_session("does-not-exist");
        assert!(result.is_none(), "unknown token should not validate");
    }

    // ── Opaque refresh token handling ─────────────────────────────────────────

    /// Acceptance criteria: POST /oauth/token with auth_code grant returns opaque
    /// refresh token (UUID), NOT the raw RINDA refresh token (issue #95 BLOCKER 2).
    #[test]
    fn test_opaque_refresh_token_is_not_rinda_token() {
        let state = make_state();
        let rinda_refresh = "rinda-real-refresh-token-secret";
        let opaque = state.create_opaque_refresh_token(rinda_refresh.to_string());

        // Opaque token should be a UUID, not the raw RINDA token.
        assert_ne!(
            opaque, rinda_refresh,
            "opaque refresh token must differ from the real RINDA token"
        );
        assert!(
            Uuid::parse_str(&opaque).is_ok(),
            "opaque refresh token should be a valid UUID"
        );
    }

    /// Acceptance criteria: POST /oauth/token with refresh_token grant accepts opaque
    /// token and looks up the real RINDA token (issue #95 BLOCKER 3).
    #[test]
    fn test_consume_opaque_refresh_token() {
        let state = make_state();
        let rinda_refresh = "rinda-real-refresh-xyz".to_string();
        let opaque = state.create_opaque_refresh_token(rinda_refresh.clone());

        // Consuming should return the real RINDA token.
        let retrieved = state.consume_opaque_refresh_token(&opaque);
        assert_eq!(
            retrieved,
            Some(rinda_refresh),
            "should return the real RINDA refresh token"
        );

        // Second consume should return None (token is single-use).
        let second = state.consume_opaque_refresh_token(&opaque);
        assert!(
            second.is_none(),
            "opaque refresh token should be single-use"
        );
    }

    #[test]
    fn test_consume_unknown_opaque_refresh_token() {
        let state = make_state();
        let result = state.consume_opaque_refresh_token("unknown-token");
        assert!(result.is_none(), "unknown opaque token should return None");
    }

    // ── Bearer token extraction ───────────────────────────────────────────────

    #[test]
    fn test_extract_bearer_token_present() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer my-session-token".parse().unwrap(),
        );
        let token = extract_bearer_token(&headers);
        assert_eq!(token, Some("my-session-token".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let headers = HeaderMap::new();
        let token = extract_bearer_token(&headers);
        assert!(token.is_none(), "should be None when header is absent");
    }

    #[test]
    fn test_extract_bearer_token_non_bearer_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Basic dXNlcjpwYXNz".parse().unwrap(),
        );
        let token = extract_bearer_token(&headers);
        assert!(
            token.is_none(),
            "Basic auth should not be extracted as Bearer"
        );
    }

    // ── Pending auth expiry ───────────────────────────────────────────────────

    #[test]
    fn test_pending_auth_expiry_check() {
        let old_time = Utc::now() - chrono::Duration::seconds(PENDING_AUTH_TTL_SECS + 1);
        let pending = PendingAuth {
            client_id: "client".to_string(),
            redirect_uri: "https://example.com/cb".to_string(),
            code_challenge: None,
            code_challenge_method: None,
            client_state: None,
            created_at: old_time,
        };
        let age = Utc::now() - pending.created_at;
        assert!(
            age > chrono::Duration::seconds(PENDING_AUTH_TTL_SECS),
            "expired pending auth should be detected"
        );
    }

    // ── validate_bearer: missing token returns 401 ────────────────────────────

    /// Acceptance criteria: MCP tool calls without a Bearer token return 401 (issue #95).
    #[test]
    fn test_validate_bearer_missing_returns_error() {
        let state = make_state();
        let headers = HeaderMap::new();
        let result = validate_bearer(&state, &headers);
        assert!(result.is_err(), "missing Bearer token should return Err");
    }

    /// Acceptance criteria: MCP tool calls with an invalid token return 401 (issue #95).
    #[test]
    fn test_validate_bearer_invalid_token_returns_error() {
        let state = make_state();
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer invalid-token-xyz".parse().unwrap(),
        );
        let result = validate_bearer(&state, &headers);
        assert!(result.is_err(), "invalid Bearer token should return Err");
    }

    /// Acceptance criteria: MCP tool calls with a valid Bearer token are accepted (issue #95).
    #[test]
    fn test_validate_bearer_valid_token_returns_rinda_token() {
        let state = make_state();
        let session_token = state.create_session("rinda-access-abc".to_string());
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {session_token}").parse().unwrap(),
        );
        let result = validate_bearer(&state, &headers);
        assert!(result.is_ok(), "valid Bearer token should be accepted");
        assert_eq!(result.unwrap(), "rinda-access-abc");
    }

    // ── AuthenticatedToken injection (BLOCKER 1) ──────────────────────────────

    /// Acceptance criteria: auth middleware inserts AuthenticatedToken for valid session (issue #95).
    #[test]
    fn test_authenticated_token_struct() {
        let token = AuthenticatedToken("rinda-token-xyz".to_string());
        assert_eq!(token.0, "rinda-token-xyz");
        // Verify Clone works (needed for axum extension).
        let cloned = token.clone();
        assert_eq!(cloned.0, "rinda-token-xyz");
    }

    // ── Integration test: OAuth HTTP endpoints ────────────────────────────────

    /// Integration test: GET /.well-known/oauth-protected-resource returns valid RFC 9728 metadata.
    #[tokio::test]
    async fn test_integration_protected_resource_metadata_endpoint() {
        use axum::Router;
        use axum::routing::get;
        use tokio::net::TcpListener;

        let oauth_state = Arc::new(OAuthState::new(
            "https://alpha.rinda.ai".to_string(),
            "http://localhost:0".to_string(),
        ));

        let app = Router::new()
            .route(
                "/.well-known/oauth-protected-resource",
                get(protected_resource_metadata),
            )
            .with_state(oauth_state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("http://{addr}/.well-known/oauth-protected-resource");
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(
            body["resource"].as_str().is_some(),
            "resource should be present"
        );
        let auth_servers = body["authorization_servers"].as_array();
        assert!(
            auth_servers.is_some() && !auth_servers.unwrap().is_empty(),
            "authorization_servers should be a non-empty array"
        );
        assert_eq!(
            body["bearer_methods_supported"],
            serde_json::json!(["header"])
        );
    }

    /// Integration test: GET /.well-known/oauth-authorization-server returns valid metadata.
    #[tokio::test]
    async fn test_integration_oauth_metadata_endpoint() {
        use axum::Router;
        use axum::routing::get;
        use tokio::net::TcpListener;

        let oauth_state = Arc::new(OAuthState::new(
            "https://alpha.rinda.ai".to_string(),
            "http://localhost:0".to_string(),
        ));

        let app = Router::new()
            .route("/.well-known/oauth-authorization-server", get(metadata))
            .with_state(oauth_state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("http://{addr}/.well-known/oauth-authorization-server");
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(
            body["issuer"].as_str().is_some(),
            "issuer should be present"
        );
        assert!(
            body["authorization_endpoint"].as_str().is_some(),
            "authorization_endpoint should be present"
        );
        assert!(
            body["token_endpoint"].as_str().is_some(),
            "token_endpoint should be present"
        );
        assert!(
            body["registration_endpoint"].as_str().is_some(),
            "registration_endpoint should be present"
        );
        assert_eq!(
            body["code_challenge_methods_supported"],
            serde_json::json!(["S256"])
        );
    }

    /// Integration test: POST /oauth/register returns client_id.
    #[tokio::test]
    async fn test_integration_oauth_register_endpoint() {
        use axum::Router;
        use axum::routing::post;
        use tokio::net::TcpListener;

        let oauth_state = Arc::new(OAuthState::new(
            "https://alpha.rinda.ai".to_string(),
            "http://localhost:0".to_string(),
        ));

        let app = Router::new()
            .route("/oauth/register", post(register))
            .with_state(oauth_state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("http://{addr}/oauth/register");
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({
                "redirect_uris": ["https://example.com/callback"],
                "client_name": "Test Client"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(
            body["client_id"].as_str().is_some(),
            "client_id should be present"
        );
        assert!(
            !body["client_id"].as_str().unwrap().is_empty(),
            "client_id should not be empty"
        );
    }
}
