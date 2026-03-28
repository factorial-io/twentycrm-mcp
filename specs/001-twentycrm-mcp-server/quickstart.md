# Quickstart: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Date**: 2026-03-27
**Blueprint**: [factorial-io/imap-mcp](https://github.com/factorial-io/imap-mcp)

## Prerequisites

- Rust toolchain (stable, 2021 edition)
- Redis instance
- A TwentyCRM workspace with API access
- A TwentyCRM API token (generated from Settings → Playground in TwentyCRM)
- An OIDC provider (GitLab, Keycloak, Auth0, or similar)

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TWENTYCRM_API_TOKEN` | Yes | Bearer token for TwentyCRM REST API |
| `TWENTYCRM_API_URL` | No | API base URL (default: `https://api.twenty.com/rest`) |
| `ALLOWED_EMAILS` | Yes | Comma-separated list of authorized user emails |
| `OIDC_ISSUER_URL` | Yes | OIDC provider URL |
| `OIDC_CLIENT_ID` | Yes | OIDC client ID |
| `OIDC_CLIENT_SECRET` | Yes | OIDC client secret |
| `BASE_URL` | Yes | Public URL of the service |
| `REDIS_URL` | Yes | Redis connection URL |
| `ENCRYPTION_KEY` | Yes | 32-byte AES-256 key, base64-encoded |
| `BIND_ADDR` | No | Listen address (default: `0.0.0.0:8080`) |
| `RUST_LOG` | No | Log level filter (default: `info`) |

## Setup

```bash
# Clone and build
cargo build --release

# Configure environment
cp .env.example .env
# Edit .env with your values

# Start Redis (via Docker)
docker compose -f docker-compose.dev.yml up -d redis

# Run in development
cargo run

# Run in production
./target/release/twentycrm-mcp
```

## Docker

```bash
# Build and run with Docker Compose
docker compose up -d
```

## Connecting from claude.ai

1. Navigate to claude.ai Settings → Integrations
2. Add a new MCP server with the hosted URL (your `BASE_URL`)
3. Authenticate via your OIDC provider when prompted
4. The 16 CRM tools will appear in the tool list

## Available Tools

| Entity | Search | View | Create | Update |
|--------|--------|------|--------|--------|
| Companies | `search_companies` | `view_company` | `create_company` | `update_company` |
| Contacts | `search_contacts` | `view_contact` | `create_contact` | `update_contact` |
| Opportunities | `search_opportunities` | `view_opportunity` | `create_opportunity` | `update_opportunity` |
| Tasks | `search_tasks` | `view_task` | `create_task` | `update_task` |

## Example Usage

After connecting, you can ask the AI assistant:
- "Search for companies in the tech industry"
- "Show me the details for company X"
- "Create a new contact: John Smith, john@example.com, CTO at Acme Corp"
- "Update the opportunity stage to Closed Won"
- "List all tasks due this week"

## Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run integration tests only
cargo test --test integration
```
