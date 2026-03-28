# Research: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Date**: 2026-03-27
**Blueprint**: [factorial-io/imap-mcp](https://github.com/factorial-io/imap-mcp) (Rust MCP server)

## Decision 1: Language & Runtime

**Decision**: Rust with Tokio async runtime
**Rationale**: The existing imap-mcp project at factorial-io provides a proven Rust MCP server blueprint. Rust gives memory safety, strong typing, and excellent performance for a remote hosted service. The `rmcp` crate provides full MCP protocol support including streamable HTTP transport.
**Alternatives considered**:
- TypeScript (official MCP SDK, but Rust is the established pattern at factorial-io)
- Go (no mature MCP SDK)

## Decision 2: MCP SDK & Transport

**Decision**: `rmcp` crate v1.2.0 with `transport-streamable-http-server` feature
**Rationale**: Proven in the imap-mcp blueprint. `rmcp` provides the full MCP protocol implementation including tool registration, JSON Schema generation via `schemars`, and streamable HTTP transport via Tower services. Stateless mode with JSON responses.
**Alternatives considered**:
- TypeScript `@modelcontextprotocol/sdk` (would break consistency with imap-mcp blueprint)
- Custom MCP implementation (unnecessary when `rmcp` is mature)

**Key crate features**:
- `rmcp` with features `server`, `transport-streamable-http-server`
- `StreamableHttpService` for per-request MCP server instantiation
- `LocalSessionManager` for session management
- `schemars` for JSON Schema generation from Rust types

## Decision 3: HTTP Framework

**Decision**: Axum 0.8.x
**Rationale**: Used in imap-mcp blueprint. Axum provides ergonomic routing, Tower middleware integration, and native async support. The `rmcp` streamable HTTP transport integrates with Axum via `axum::routing::any()` for the MCP endpoint.
**Alternatives considered**:
- Actix-web (heavier, different ecosystem than Tower)
- Warp (less maintained than Axum)

**Middleware stack** (from blueprint):
- `tower-http::TraceLayer` — HTTP request/response tracing
- `tower-http::CorsLayer` — CORS (any origin, GET/POST/DELETE/OPTIONS)

## Decision 4: Authentication Architecture

**Decision**: OAuth 2.0 + OIDC flow with `openidconnect` crate, following imap-mcp patterns
**Rationale**: The imap-mcp blueprint implements a complete OAuth/OIDC flow compatible with claude.ai, including Dynamic Client Registration (RFC 7591), PKCE (S256), well-known metadata endpoints, and bearer token validation. This same pattern applies directly.

**Key difference from imap-mcp**: After OAuth authentication, instead of collecting IMAP credentials, the server checks the user's email against a whitelist. The TwentyCRM API token is server-side configuration, not per-user.

**Auth flow** (adapted from imap-mcp):
1. Claude.ai discovers server via `/.well-known/oauth-protected-resource`
2. Claude registers as OAuth client via `POST /register` (Dynamic Client Registration)
3. `GET /auth/login` initiates OIDC authorization with PKCE (S256)
4. User authenticates with OIDC provider in browser
5. `GET /auth/callback` handles OIDC response, verifies ID token, extracts email
6. Server checks email against `ALLOWED_EMAILS` whitelist
7. If whitelisted, creates session in Redis with email, returns authorization code
8. `POST /auth/token` exchanges code for bearer token (UUID v4), verifies PKCE
9. Claude sends subsequent requests with `Authorization: Bearer <token>`
10. MCP handler extracts bearer token, looks up session in Redis, creates per-request MCP server

**Required endpoints** (same as imap-mcp):
- `/.well-known/oauth-protected-resource` — Protected Resource Metadata
- `/.well-known/oauth-authorization-server` — Authorization Server Metadata
- `POST /register` — Dynamic Client Registration
- `GET /auth/login` — OIDC authorization initiation
- `GET /auth/callback` — OIDC callback handler
- `POST /auth/token` — Token exchange
- `/mcp` and `/mcp/{path}` — Streamable HTTP MCP endpoint

**OIDC provider**: Any provider supporting standard OIDC (GitLab, Keycloak, Auth0, etc.)

## Decision 5: Session Storage

**Decision**: Redis with AES-256-GCM encryption (following imap-mcp pattern)
**Rationale**: The imap-mcp blueprint uses Redis for session storage with encrypted secrets. For this project, sessions store the authenticated user's email and bearer token mapping. Redis provides TTL-based session expiry.
**Alternatives considered**:
- In-memory sessions (not durable across restarts)
- No sessions / stateless JWT (would require larger tokens and no revocation)

**Key crates**: `redis` (with `tokio-comp`, `aio` features), `aes-gcm`

## Decision 6: TwentyCRM API Client

**Decision**: `reqwest` HTTP client calling TwentyCRM REST API directly
**Rationale**: `reqwest` is the standard async HTTP client in Rust. TwentyCRM provides a well-documented REST API with OpenAPI spec. The 4 entities and their CRUD endpoints are manageable with a thin client wrapper.
**Alternatives considered**:
- `hyper` directly (lower-level than needed)
- Auto-generated client from OpenAPI spec (adds code generation complexity)

## Decision 7: Testing Framework

**Decision**: Rust built-in `#[cfg(test)]` + `tests/integration.rs` following imap-mcp patterns
**Rationale**: The imap-mcp blueprint uses embedded unit tests per module and a separate integration test file that uses `tower::ServiceExt::oneshot()` to test the full Axum router without a running server.

**Testing approach**:
- **Unit tests**: Embedded `#[cfg(test)] mod tests` in each module — error mapping, parameter deserialization, whitelist parsing, API response parsing
- **Integration tests**: `tests/integration.rs` using `tower::ServiceExt::oneshot()` on the Axum router — OAuth endpoints, MCP auth requirements, tool parameter deserialization

## Decision 8: Error Handling

**Decision**: Centralized `AppError` enum with `thiserror`, following imap-mcp pattern
**Rationale**: The blueprint uses a typed error enum that maps each variant to an HTTP status code. Sensitive details are logged server-side; clients receive generic messages. Errors propagated with `?` operator.

**Error variants** (adapted for this project):
- `AppError::Redis` → 500 (logged, generic message)
- `AppError::Auth` → 401 Unauthorized
- `AppError::SessionNotFound` → 401 Unauthorized
- `AppError::TwentyCrmApi` → 502 Bad Gateway (upstream API error)
- `AppError::NotFound` → 404 (record not found)
- `AppError::Validation` → 400 (invalid input)
- `AppError::Encryption` → 500 (logged, generic message)
- `AppError::Internal` → 500 (logged, generic message)

## Decision 9: Project Structure

**Decision**: Single crate, flat src directory following imap-mcp module layout

```
src/
├── main.rs            # Entrypoint, env config, server bootstrap
├── lib.rs             # AppState, build_router(), MCP handler, bearer token extraction
├── auth.rs            # OAuth/OIDC flow, dynamic client registration, token exchange
├── session.rs         # Redis session storage, AES-256-GCM encryption
├── error.rs           # Unified AppError enum with thiserror
├── mcp.rs             # TwentyCrmMcpServer struct + all 16 MCP tool implementations
├── twentycrm.rs       # TwentyCRM REST API client (reqwest-based)
└── types.rs           # Shared types: entity structs, API response types, tool params

tests/
└── integration.rs     # Integration tests using tower::ServiceExt::oneshot()
```

## Decision 10: Environment Configuration

**Decision**: Environment variables, panic on missing required vars (same as imap-mcp)
**Rationale**: Simple `env_var()` / `env_var_or()` helpers that panic at startup if required vars are missing. No config file parsing — purely environment-driven for container deployments.

| Variable | Required | Purpose |
|----------|----------|---------|
| `TWENTYCRM_API_TOKEN` | Yes | Bearer token for TwentyCRM API |
| `TWENTYCRM_API_URL` | No | API base URL (default: `https://api.twenty.com/rest`) |
| `ALLOWED_EMAILS` | Yes | Comma-separated list of whitelisted user emails |
| `OIDC_ISSUER_URL` | Yes | OIDC provider URL |
| `OIDC_CLIENT_ID` | Yes | OIDC client ID |
| `OIDC_CLIENT_SECRET` | Yes | OIDC client secret |
| `BASE_URL` | Yes | Public URL of the service |
| `REDIS_URL` | Yes | Redis connection URL |
| `ENCRYPTION_KEY` | Yes | 32-byte AES-256 key, base64-encoded |
| `BIND_ADDR` | No | Listen address (default: `0.0.0.0:8080`) |
| `RUST_LOG` | No | Log level filter (default: `info`) |

## Decision 11: Stateless MCP, Stateful Auth

**Decision**: Stateless MCP transport, stateful OAuth sessions in Redis
**Rationale**: MCP requests are stateless (no cross-request MCP state). However, the OAuth flow requires session persistence for bearer token → user email mapping. Redis handles session storage with TTL-based expiry (30 days, refreshed on access). This matches the imap-mcp pattern exactly.
