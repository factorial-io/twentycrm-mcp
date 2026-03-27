# Implementation Plan: TwentyCRM MCP Server

**Branch**: `001-twentycrm-mcp-server` | **Date**: 2026-03-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-twentycrm-mcp-server/spec.md`
**Blueprint**: [factorial-io/imap-mcp](https://github.com/factorial-io/imap-mcp)

## Summary

Build a remote MCP server in Rust that exposes TwentyCRM CRM operations as 16 MCP tools (search, view, create, update for companies, contacts, opportunities, and tasks). Uses Axum + `rmcp` for streamable HTTP transport, OAuth 2.0/OIDC for user authentication with an email whitelist, Redis for session storage, and `reqwest` for proxying requests to the TwentyCRM REST API. Architecture follows the proven imap-mcp blueprint.

## Technical Context

**Language/Version**: Rust (2021 edition), Tokio async runtime
**Primary Dependencies**: `rmcp` 1.2.0, `axum` 0.8.x, `openidconnect` 3.5.0, `reqwest`, `redis` 1.0.0, `schemars` 1.0.0, `serde`/`serde_json`
**Storage**: Redis (session storage only — AES-256-GCM encrypted)
**Testing**: Rust built-in `#[cfg(test)]` + `tests/integration.rs` with `tower::ServiceExt::oneshot()`
**Target Platform**: Linux server (Docker container, remote hosted)
**Project Type**: Web service (MCP server)
**Performance Goals**: <5s response time per tool call under normal network conditions
**Constraints**: Per-request MCP server instantiation; stateless MCP transport; stateful OAuth sessions in Redis
**Scale/Scope**: Team-sized usage (whitelisted users), 4 CRM entities, 16 MCP tools

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution is an unfilled template — no gates defined. No violations to check.

## Project Structure

### Documentation (this feature)

```text
specs/001-twentycrm-mcp-server/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: technology decisions
├── data-model.md        # Phase 1: entity schemas from TwentyCRM API
├── quickstart.md        # Phase 1: setup and usage guide
├── contracts/           # Phase 1: MCP tool contracts
│   └── mcp-tools.md     # 16 tool parameter/response schemas
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs                # Entrypoint: env config, tracing setup, server bootstrap
├── lib.rs                 # AppState, build_router(), mcp_handler, bearer token extraction
├── auth.rs                # OAuth/OIDC flow: well-known endpoints, registration,
│                          #   login, callback, token exchange (from imap-mcp)
├── session.rs             # Redis session storage, AES-256-GCM encryption (from imap-mcp)
├── error.rs               # Centralized AppError enum with thiserror (from imap-mcp)
├── mcp.rs                 # TwentyCrmMcpServer struct + 16 MCP tool implementations
├── twentycrm.rs           # TwentyCRM REST API client (reqwest-based)
│                          #   - search, get, create, update for each entity
│                          #   - filter/pagination/ordering query builders
│                          #   - field selection support
└── types.rs               # Entity structs (Company, Person, Opportunity, Task),
                           #   API response types, tool parameter structs with schemars

tests/
└── integration.rs         # Integration tests via tower::ServiceExt::oneshot()
                           #   - OAuth well-known endpoints
                           #   - MCP auth requirements (401 without token)
                           #   - Tool parameter deserialization
                           #   - Dynamic client registration

Cargo.toml
Dockerfile
docker-compose.yml
docker-compose.dev.yml
.env.example
CLAUDE.md
```

**Structure Decision**: Single Rust crate following the imap-mcp module layout. Key modules mirror the blueprint (`auth.rs`, `session.rs`, `error.rs`, `lib.rs`, `main.rs`). The `mcp.rs` module contains all 16 tool handlers (vs 8 in imap-mcp). The `twentycrm.rs` module replaces `imap.rs` as the domain-specific client. The `types.rs` module holds shared entity/param structs.

### Key Dependencies (Cargo.toml)

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.8.x (features: macros) | Web framework |
| `tower-http` | 0.6.x (features: cors, trace) | HTTP middleware |
| `tower` | 0.5.x | Service abstractions |
| `tokio` | 1.x (features: full) | Async runtime |
| `rmcp` | 1.2.0 (features: server, transport-streamable-http-server) | MCP protocol |
| `openidconnect` | 3.5.0 | OIDC client |
| `reqwest` | latest (features: json, rustls-tls) | HTTP client for TwentyCRM API |
| `redis` | 1.0.0 (features: tokio-comp, aio) | Session storage |
| `aes-gcm` | 0.10.x | Session encryption |
| `serde` | 1.x (features: derive) | Serialization |
| `serde_json` | 1.x | JSON |
| `schemars` | 1.0.0 | JSON Schema for MCP tool params |
| `thiserror` | 2.x | Typed error derivation |
| `anyhow` | 1.x | Ad-hoc error context |
| `tracing` | 0.1.x | Structured logging |
| `tracing-subscriber` | 0.3.x (features: env-filter) | Log subscriber |
| `sha2` | 0.11.x | PKCE verification |
| `secrecy` | 0.10.x (features: serde) | Secret string handling |
| `uuid` | 1.x (features: v4) | Bearer token generation |
| `chrono` | 0.4.x (features: serde) | Timestamps |
| `base64` | 0.22.x | Base64 encoding |
| `url` | 2.x | URL parsing |
| `http` | 1.x | HTTP types |
| `futures` | 0.3.x | Async utilities |

## Complexity Tracking

No constitution gates defined — no violations to justify.
