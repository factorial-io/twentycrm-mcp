use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request},
    response::Response,
    routing::{any, get, post},
    Router,
};
use http_body_util::BodyExt;
use openidconnect::core::CoreClient;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    tower::{StreamableHttpServerConfig, StreamableHttpService},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod error;
pub mod mcp;
pub mod session;
pub mod twentycrm;
pub mod types;

use crate::error::AppError;
use crate::mcp::TwentyCrmMcpServer;
use crate::twentycrm::TwentyCrmClient;

pub struct AppState {
    pub base_url: String,
    pub oidc_client: CoreClient,
    pub redis: redis::aio::MultiplexedConnection,
    pub encryption_key: [u8; 32],
    pub twentycrm_client: TwentyCrmClient,
    pub required_groups: Vec<String>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        base_url: &str,
        oidc_issuer_url: &str,
        oidc_client_id: &str,
        oidc_client_secret: &str,
        redis_url: &str,
        encryption_key: &str,
        twentycrm_api_token: &str,
        twentycrm_api_url: &str,
        required_groups: Vec<String>,
    ) -> Arc<Self> {
        let oidc_client =
            auth::init_oidc_client(oidc_issuer_url, oidc_client_id, oidc_client_secret)
                .await
                .expect("Failed to initialize OIDC client");

        let redis_client = redis::Client::open(redis_url).expect("Invalid Redis URL");
        let redis = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");

        let encryption_key =
            session::decode_encryption_key(encryption_key).expect("Invalid encryption key");

        let twentycrm_client = TwentyCrmClient::new(twentycrm_api_url, twentycrm_api_token);

        Arc::new(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            oidc_client,
            redis,
            encryption_key,
            twentycrm_client,
            required_groups,
        })
    }
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::DELETE,
            http::Method::OPTIONS,
        ])
        .allow_headers([
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
        ]);

    Router::new()
        // Well-known OAuth endpoints
        .route(
            "/.well-known/oauth-protected-resource",
            get(auth::oauth_protected_resource),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(auth::oauth_authorization_server),
        )
        // Dynamic client registration
        .route("/register", post(auth::register))
        // OAuth flow
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/token", post(auth::token))
        // MCP endpoint (all methods)
        .route("/mcp", any(mcp_handler))
        .route("/mcp/{*path}", any(mcp_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// MCP handler that authenticates the bearer token and creates a per-request MCP server.
async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<Response, AppError> {
    // Extract bearer token
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| AppError::Auth("Bearer token required".into()))?;

    // Look up session in Redis
    let mut redis = state.redis.clone();
    let _session = session::get_session(&mut redis, &token, &state.encryption_key).await?;

    // Create per-request MCP server
    let mcp_server = TwentyCrmMcpServer::new(state.twentycrm_client.clone());

    let config = StreamableHttpServerConfig::default()
        .with_stateful_mode(false)
        .with_json_response(true);

    let session_manager = Arc::new(LocalSessionManager::default());

    let service =
        StreamableHttpService::new(move || Ok(mcp_server.clone()), session_manager, config);

    // Delegate to the MCP transport service
    let response = service.handle(request).await;

    // Convert BoxBody response to axum Body
    let (parts, body) = response.into_parts();
    let bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Internal(format!("Body collect error: {e}")))?
        .to_bytes();
    let axum_body = Body::from(bytes);
    Ok(Response::from_parts(parts, axum_body))
}

/// Extract bearer token from Authorization header.
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get(http::header::AUTHORIZATION)?;
    let auth_str = auth.to_str().ok()?;

    auth_str
        .strip_prefix("Bearer ")
        .map(|token| token.to_string())
}
