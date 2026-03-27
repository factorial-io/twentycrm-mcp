use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use base64::Engine;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::session::{self, Session};
use crate::AppState;

// ─── Well-Known Endpoints ───────────────────────────────────────────

pub async fn oauth_protected_resource(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(json!({
        "resource": state.base_url,
        "authorization_servers": [format!("{}/auth", state.base_url)],
        "bearer_methods_supported": ["header"],
    }))
}

pub async fn oauth_authorization_server(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(json!({
        "issuer": format!("{}/auth", state.base_url),
        "authorization_endpoint": format!("{}/auth/login", state.base_url),
        "token_endpoint": format!("{}/auth/token", state.base_url),
        "registration_endpoint": format!("{}/register", state.base_url),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "none"],
    }))
}

// ─── Dynamic Client Registration ────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
}

pub async fn register(
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.redirect_uris.is_empty() {
        return Err(AppError::Validation(
            "redirect_uris must not be empty".into(),
        ));
    }

    // Validate all redirect URIs use HTTPS
    for uri in &body.redirect_uris {
        if !uri.starts_with("https://") && !uri.starts_with("http://localhost") {
            return Err(AppError::Validation(format!(
                "redirect_uri must use HTTPS: {uri}"
            )));
        }
    }

    let client_id = Uuid::new_v4().to_string();

    Ok(Json(json!({
        "client_id": client_id,
        "client_name": body.client_name.unwrap_or_else(|| "MCP Client".into()),
        "redirect_uris": body.redirect_uris,
    })))
}

// ─── Login (OIDC Authorization Initiation) ──────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub redirect_uri: String,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub client_id: Option<String>,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LoginQuery>,
) -> Result<Response, AppError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Store auth state in Redis for callback
    let auth_state_id = Uuid::new_v4().to_string();
    let auth_state = AuthStateData {
        redirect_uri: params.redirect_uri.clone(),
        client_state: params.state.clone(),
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: params.code_challenge_method.clone(),
        pkce_verifier: pkce_verifier.secret().to_string(),
        client_id: params.client_id.clone(),
    };

    let auth_state_json = serde_json::to_string(&auth_state)
        .map_err(|e| AppError::Internal(format!("Failed to serialize auth state: {e}")))?;

    let mut redis = state.redis.clone();
    redis::AsyncCommands::set_ex::<_, _, ()>(
        &mut redis,
        format!("auth_state:{auth_state_id}"),
        &auth_state_json,
        300, // 5 minute TTL
    )
    .await?;

    // Build OIDC authorization URL
    let callback_url = format!("{}/auth/callback", state.base_url);
    let redirect_url = RedirectUrl::new(callback_url)
        .map_err(|e| AppError::Internal(format!("Invalid callback URL: {e}")))?;

    let client = state.oidc_client.clone().set_redirect_uri(redirect_url);

    let (auth_url, _csrf, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            || CsrfToken::new(auth_state_id),
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok(Redirect::temporary(auth_url.as_str()).into_response())
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthStateData {
    redirect_uri: String,
    client_state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    pkce_verifier: String,
    client_id: Option<String>,
}

// ─── Callback (OIDC Response Handler) ───────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

pub async fn callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    // Retrieve and delete auth state
    let mut redis = state.redis.clone();
    let auth_state_json: Option<String> = redis::AsyncCommands::get(
        &mut redis,
        format!("auth_state:{}", params.state),
    )
    .await?;

    let auth_state_json = auth_state_json.ok_or(AppError::Auth(
        "Authorization state expired or invalid".into(),
    ))?;

    // Delete the auth state (single-use)
    let _: () = redis::AsyncCommands::del(
        &mut redis,
        format!("auth_state:{}", params.state),
    )
    .await?;

    let auth_state: AuthStateData = serde_json::from_str(&auth_state_json)
        .map_err(|e| AppError::Internal(format!("Failed to parse auth state: {e}")))?;

    // Exchange code for tokens with OIDC provider
    let callback_url = format!("{}/auth/callback", state.base_url);
    let redirect_url = RedirectUrl::new(callback_url)
        .map_err(|e| AppError::Internal(format!("Invalid callback URL: {e}")))?;

    let client = state.oidc_client.clone().set_redirect_uri(redirect_url);

    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(PkceCodeVerifier::new(auth_state.pkce_verifier))
        .request_async(openidconnect::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::Oidc(format!("Token exchange failed: {e}")))?;

    // Extract email from ID token claims
    let id_token = token_response
        .id_token()
        .ok_or(AppError::Oidc("No ID token in response".into()))?;

    let nonce_verifier = |_: Option<&Nonce>| Ok(());
    let claims = id_token
        .claims(&state.oidc_client.id_token_verifier(), nonce_verifier)
        .map_err(|e| AppError::Oidc(format!("ID token verification failed: {e}")))?;

    let email = claims
        .email()
        .map(|e| e.to_string())
        .ok_or(AppError::Oidc("No email claim in ID token".into()))?
        .to_lowercase();

    // Check email whitelist
    if !state.allowed_emails.contains(&email) {
        tracing::warn!("Access denied for email: {email}");
        return Err(AppError::Auth(format!(
            "User {email} is not authorized to access this server"
        )));
    }

    tracing::info!("Authenticated user: {email}");

    // Create authorization code for the MCP client
    let auth_code = Uuid::new_v4().to_string();
    let auth_code_data = json!({
        "email": email,
        "code_challenge": auth_state.code_challenge,
        "code_challenge_method": auth_state.code_challenge_method,
        "client_id": auth_state.client_id,
    });

    redis::AsyncCommands::set_ex::<_, _, ()>(
        &mut redis,
        format!("auth_code:{auth_code}"),
        serde_json::to_string(&auth_code_data)
            .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))?,
        300, // 5 minute TTL
    )
    .await?;

    // Redirect back to the MCP client with the authorization code
    let mut redirect = auth_state.redirect_uri;
    redirect.push_str(&format!("?code={auth_code}"));
    if let Some(client_state) = auth_state.client_state {
        redirect.push_str(&format!("&state={client_state}"));
    }

    Ok(Redirect::temporary(&redirect).into_response())
}

// ─── Token Exchange ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub code_verifier: Option<String>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
}

pub async fn token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.grant_type != "authorization_code" {
        return Err(AppError::Validation(
            "Unsupported grant_type. Must be authorization_code.".into(),
        ));
    }

    // Retrieve and delete auth code (single-use)
    let mut redis = state.redis.clone();
    let code_data_json: Option<String> = redis::AsyncCommands::get(
        &mut redis,
        format!("auth_code:{}", body.code),
    )
    .await?;

    let code_data_json = code_data_json.ok_or(AppError::Auth(
        "Authorization code expired or invalid".into(),
    ))?;

    let _: () = redis::AsyncCommands::del(
        &mut redis,
        format!("auth_code:{}", body.code),
    )
    .await?;

    let code_data: serde_json::Value = serde_json::from_str(&code_data_json)
        .map_err(|e| AppError::Internal(format!("Failed to parse auth code data: {e}")))?;

    let email = code_data["email"]
        .as_str()
        .ok_or(AppError::Internal("Missing email in auth code".into()))?
        .to_string();

    // Verify PKCE code_verifier if code_challenge was provided
    if let Some(expected_challenge) = code_data["code_challenge"].as_str() {
        let verifier = body.code_verifier.as_deref().ok_or(AppError::Auth(
            "code_verifier required for PKCE".into(),
        ))?;

        let method = code_data["code_challenge_method"]
            .as_str()
            .unwrap_or("S256");

        if method == "S256" {
            let mut hasher = Sha256::new();
            hasher.update(verifier.as_bytes());
            let computed = base64::prelude::BASE64_URL_SAFE_NO_PAD
                .encode(hasher.finalize());

            if computed != expected_challenge {
                return Err(AppError::Auth("PKCE verification failed".into()));
            }
        }
    }

    // Create bearer token and session
    let bearer_token = Uuid::new_v4().to_string();
    let sess = Session { email };

    session::create_session(&mut redis, &bearer_token, &sess, &state.encryption_key).await?;

    Ok(Json(json!({
        "access_token": bearer_token,
        "token_type": "bearer",
    })))
}

/// Initialize the OIDC client from the issuer URL.
pub async fn init_oidc_client(
    issuer_url: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<CoreClient, AppError> {
    let issuer = IssuerUrl::new(issuer_url.to_string())
        .map_err(|e| AppError::Oidc(format!("Invalid issuer URL: {e}")))?;

    let metadata = CoreProviderMetadata::discover_async(
        issuer,
        openidconnect::reqwest::async_http_client,
    )
    .await
    .map_err(|e| AppError::Oidc(format!("OIDC discovery failed: {e}")))?;

    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
    );

    Ok(client)
}
