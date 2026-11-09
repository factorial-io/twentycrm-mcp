// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Authorization for MCP clients.
//!
//! grantry is the OAuth 2.1 authorization server MCP clients talk to
//! (metadata, registration, authorize, token, refresh). Its consent step is
//! the upstream OIDC login (GitLab) with the required-group check: once the
//! user passes it, `callback` approves the pending authorization with the
//! user's email as the grant subject.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use grantry::{
    metadata::{AuthorizationServerMetadata, ProtectedResourceMetadata},
    routes::Routes,
    server::ApproveError,
    store::NoEvents,
    Algorithm, BackendError, Config, Grant, NoFetch, Recheck, ScopeSpec, Server, Signer, Subjects,
};
use grantry_redis::RedisStore;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::AppState;

/// The authorization server; the grant subject is the user's email.
pub type OAuth = Server<String>;

pub const MCP_PATH: &str = "/mcp";
const AUTHORIZE_PATH: &str = "/auth/authorize";
const TOKEN_PATH: &str = "/auth/token";
const REGISTER_PATH: &str = "/register";
const PROTECTED_RESOURCE_PATH: &str = "/.well-known/oauth-protected-resource";

/// Grants last 30 days; no refresh extends them.
const GRANT_DAYS: i64 = 30;

fn scopes() -> ScopeSpec {
    // offline_access by default, so clients get refresh tokens without asking.
    ScopeSpec::new(
        &["mcp", "offline_access"],
        &["mcp", "offline_access"],
        "offline_access",
    )
}

/// The protected resource: the MCP endpoint URL clients connect to.
pub fn resource(base_url: &str) -> String {
    format!("{base_url}{MCP_PATH}")
}

/// URL of the protected resource metadata for the MCP endpoint (RFC 9728).
pub fn protected_resource_metadata_url(base_url: &str) -> String {
    format!("{base_url}{PROTECTED_RESOURCE_PATH}{MCP_PATH}")
}

pub fn oauth_server(
    base_url: &str,
    signing_key: &str,
    redis: ConnectionManager,
) -> Result<OAuth, AppError> {
    let signer = Signer::hmac_base64(Algorithm::HS256, signing_key)
        .map_err(|e| AppError::Internal(format!("Invalid TOKEN_SIGNING_KEY: {e}")))?;
    let config = Config {
        grant_lifetime: chrono::Duration::days(GRANT_DAYS),
        ..Config::new(resource(base_url), "mcp", scopes(), signer)
    };
    Ok(Server::new(
        config,
        Arc::new(RedisStore::with_prefix(redis, "grantry:")),
        Arc::new(GroupMembers),
        Arc::new(NoEvents),
        // ponytail: no Client ID Metadata Documents (fetching client-given URLs
        // needs an egress policy); clients use dynamic registration.
        Arc::new(NoFetch),
    ))
}

/// The OAuth endpoints: grantry's metadata, authorize, token and register,
/// plus the protected resource metadata and the login callback.
pub fn oauth_router(state: Arc<AppState>) -> Router {
    let base = state.base_url.clone();
    let mut metadata = AuthorizationServerMetadata::new(
        &base,
        AUTHORIZE_PATH,
        TOKEN_PATH,
        Some(REGISTER_PATH),
        &scopes().known,
    );
    metadata.client_id_metadata_document_supported = false;

    let consent_base = base.clone();
    let routes = Routes {
        metadata,
        authorize_path: AUTHORIZE_PATH.into(),
        token_path: TOKEN_PATH.into(),
        register_path: Some(REGISTER_PATH.into()),
        consent_url: Arc::new(move |id| format!("{consent_base}/auth/login?pending={id}")),
        other_grant: None,
    };

    let protected = Router::new()
        .route(PROTECTED_RESOURCE_PATH, get(protected_resource))
        .route(
            &format!("{PROTECTED_RESOURCE_PATH}{MCP_PATH}"),
            get(protected_resource),
        )
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .with_state(state.clone());

    grantry::routes::router(state.oauth.clone(), routes).merge(protected)
}

async fn protected_resource(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(ProtectedResourceMetadata::new(
        &resource(&state.base_url),
        &state.base_url,
        &scopes().known,
        Some("TwentyCRM"),
    ))
}

/// Re-checked on every token issue and refresh.
struct GroupMembers;

#[async_trait::async_trait]
impl Subjects<String> for GroupMembers {
    async fn recheck(&self, grant: &Grant<String>) -> Result<Recheck, BackendError> {
        // ponytail: group membership is checked at login only, so a user
        // removed from the group keeps access until the grant expires
        // (GRANT_DAYS). Store the GitLab refresh token and re-query groups
        // here if that window matters.
        let mut claims = serde_json::Map::new();
        claims.insert("sub".into(), grant.subject.clone().into());
        Ok(Recheck::Allow(claims))
    }
}

// ─── Login (consent step: upstream OIDC) ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub pending: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthStateData {
    pending: Uuid,
    pkce_verifier: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LoginQuery>,
) -> Result<Response, AppError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let auth_state_id = Uuid::new_v4().to_string();
    let auth_state = AuthStateData {
        pending: params.pending,
        pkce_verifier: pkce_verifier.secret().to_string(),
    };
    let auth_state_json = serde_json::to_string(&auth_state)
        .map_err(|e| AppError::Internal(format!("Failed to serialize auth state: {e}")))?;

    let mut redis = state.redis.clone();
    redis::AsyncCommands::set_ex::<_, _, ()>(
        &mut redis,
        format!("auth_state:{auth_state_id}"),
        &auth_state_json,
        600, // as long as grantry keeps the pending authorization
    )
    .await?;

    let client = state
        .oidc_client
        .clone()
        .set_redirect_uri(callback_url(&state)?);
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

fn callback_url(state: &AppState) -> Result<RedirectUrl, AppError> {
    RedirectUrl::new(format!("{}/auth/callback", state.base_url))
        .map_err(|e| AppError::Internal(format!("Invalid callback URL: {e}")))
}

// ─── Callback (OIDC response, then approve) ─────────────────────────

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

pub async fn callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    // Retrieve and delete auth state (single-use)
    let mut redis = state.redis.clone();
    let key = format!("auth_state:{}", params.state);
    let auth_state_json: Option<String> = redis::AsyncCommands::get_del(&mut redis, &key).await?;
    let auth_state_json = auth_state_json.ok_or(AppError::Auth(
        "Authorization state expired or invalid".into(),
    ))?;
    let auth_state: AuthStateData = serde_json::from_str(&auth_state_json)
        .map_err(|e| AppError::Internal(format!("Failed to parse auth state: {e}")))?;

    // Exchange code for tokens with OIDC provider
    let client = state
        .oidc_client
        .clone()
        .set_redirect_uri(callback_url(&state)?);
    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(PkceCodeVerifier::new(auth_state.pkce_verifier))
        .request_async(openidconnect::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::Oidc(format!("Token exchange failed: {e}")))?;

    let id_token = token_response
        .id_token()
        .ok_or(AppError::Oidc("No ID token in response".into()))?;

    let nonce_verifier = |_: Option<&Nonce>| Ok(());
    let claims = id_token
        .claims(&state.oidc_client.id_token_verifier(), nonce_verifier)
        .map_err(|e| AppError::Oidc(format!("ID token verification failed: {e}")))?;

    // The email is the grant subject, so it must be present.
    let email = claims
        .email()
        .map(|e| e.to_string().to_lowercase())
        .ok_or(AppError::Oidc("No email claim in ID token".into()))?;

    // Fetch user groups from the OIDC userinfo endpoint using the access token.
    // GitLab includes a "groups" claim in the userinfo response but not in the ID token.
    let access_token = token_response.access_token().secret();
    let userinfo_url = format!(
        "{}/oauth/userinfo",
        state.oidc_issuer_url.trim_end_matches('/')
    );

    let userinfo_json: serde_json::Value = reqwest::Client::new()
        .get(&userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Oidc(format!("Userinfo request failed: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Oidc(format!("Userinfo parse failed: {e}")))?;

    tracing::debug!("Userinfo response: {}", userinfo_json);

    let user_groups: Vec<String> = userinfo_json
        .get("groups")
        .or_else(|| userinfo_json.get("groups_direct"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // User must be in at least one of the required groups
    let has_required_group = state
        .required_groups
        .iter()
        .any(|required| user_groups.iter().any(|user_group| user_group == required));

    if !has_required_group {
        tracing::warn!(
            "Access denied for {email}: user groups {:?} do not match required groups {:?}",
            user_groups,
            state.required_groups
        );
        return Err(AppError::Auth(format!(
            "User {email} is not a member of a required group"
        )));
    }

    tracing::info!("Authenticated user: {email} (groups: {:?})", user_groups);

    let approved = state
        .oauth
        .approve(auth_state.pending, &email, None)
        .await
        .map_err(|e| match e {
            ApproveError::NotFound => {
                AppError::Auth("Authorization request expired or invalid".into())
            }
            ApproveError::InvalidScope => AppError::Validation("Invalid scope".into()),
            ApproveError::Backend(e) => AppError::Internal(format!("OAuth store error: {e}")),
        })?;

    Ok(Redirect::temporary(&approved.redirect).into_response())
}

/// Initialize the OIDC client from the issuer URL.
pub async fn init_oidc_client(
    issuer_url: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<CoreClient, AppError> {
    let issuer = IssuerUrl::new(issuer_url.to_string())
        .map_err(|e| AppError::Oidc(format!("Invalid issuer URL: {e}")))?;

    let metadata =
        CoreProviderMetadata::discover_async(issuer, openidconnect::reqwest::async_http_client)
            .await
            .map_err(|e| AppError::Oidc(format!("OIDC discovery failed: {e}")))?;

    Ok(CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
    ))
}
