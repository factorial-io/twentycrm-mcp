# twentycrm-mcp Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-27

## Active Technologies

- Rust (2021 edition) + axum 0.8.x + rmcp 1.2.0 + reqwest + redis + openidconnect (001-twentycrm-mcp-server)

## Blueprint

This project follows the architecture of [factorial-io/imap-mcp](https://github.com/factorial-io/imap-mcp).

## Project Structure

```text
src/
├── main.rs          # Entrypoint, env config, server bootstrap
├── lib.rs           # AppState, build_router(), mcp_handler
├── auth.rs          # OAuth/OIDC flow (from imap-mcp)
├── session.rs       # Redis sessions, AES-256-GCM encryption (from imap-mcp)
├── error.rs         # AppError enum (from imap-mcp)
├── mcp.rs           # MCP tool implementations
├── twentycrm.rs     # TwentyCRM REST API client
└── types.rs         # Entity structs, tool params (schemars)
tests/
└── integration.rs
```

## Commands

```bash
cargo build          # Build
cargo test           # Run all tests
cargo run            # Run dev server
cargo clippy         # Lint
cargo fmt            # Format
```

## Code Style

- Rust 2021 edition: Follow standard Rust conventions
- Never suppress errors with `.unwrap()`, `.expect()`, or silent `let _ =`
- Propagate errors using `?` and return meaningful errors
- Use `thiserror` for typed errors, `anyhow` for ad-hoc context
- Tool params use `serde` + `schemars` for deserialization and JSON Schema

## Recent Changes

- 001-twentycrm-mcp-server: Rust MCP server with axum + rmcp, following imap-mcp blueprint

## TwentyCRM API Reference

The OpenAPI specs live in `docs/` but are git-ignored; fetch them from a Twenty instance (see README):
- `docs/core.json` — Core entity endpoints (companies, people, notes, tasks, opportunities, timelineActivities, noteTargets, etc.)
- `docs/metadata.json` — Metadata/schema endpoints
