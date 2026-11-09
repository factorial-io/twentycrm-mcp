// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;
use tracing_subscriber::EnvFilter;

fn env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing required environment variable: {name}"))
}

fn env_var_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let bind_addr = env_var_or("BIND_ADDR", "0.0.0.0:8080");
    let base_url = env_var("BASE_URL");
    let oidc_issuer_url = env_var("OIDC_ISSUER_URL");
    let oidc_client_id = env_var("OIDC_CLIENT_ID");
    let oidc_client_secret = env_var("OIDC_CLIENT_SECRET");
    let redis_url = env_var("REDIS_URL");
    let token_signing_key = env_var("TOKEN_SIGNING_KEY");
    let twentycrm_api_token = env_var("TWENTYCRM_API_TOKEN");
    let twentycrm_api_url = env_var_or("TWENTYCRM_API_URL", "https://api.twenty.com/rest");
    let required_groups: Vec<String> = env_var("REQUIRED_GROUPS")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    tracing::info!("Starting TwentyCRM MCP server");
    tracing::info!("Required OIDC groups: {:?}", required_groups);

    let app_state = twentycrm_mcp::AppState::new(
        &base_url,
        &oidc_issuer_url,
        &oidc_client_id,
        &oidc_client_secret,
        &redis_url,
        &token_signing_key,
        &twentycrm_api_token,
        &twentycrm_api_url,
        required_groups,
    )
    .await;

    let router = twentycrm_mcp::build_router(app_state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {bind_addr}: {e}"));

    tracing::info!("Listening on {bind_addr}");

    axum::serve(listener, router).await.expect("Server error");
}
