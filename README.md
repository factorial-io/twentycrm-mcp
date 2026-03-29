# TwentyCRM MCP Server

MCP server for [TwentyCRM](https://twenty.com/) that exposes CRM operations as tools via the [Model Context Protocol](https://modelcontextprotocol.io/). Built with Rust, axum, and [rmcp](https://github.com/anthropics/rmcp).

Access is controlled via OIDC group membership (e.g. GitLab groups). The server uses a shared TwentyCRM API token for all API requests.

## Prerequisites

- Rust toolchain (stable, 2021 edition)
- Docker and Docker Compose
- A TwentyCRM workspace with an API token (Settings > Developers > API keys)
- An OIDC provider (GitLab, Keycloak, Auth0, or similar)
- [1Password CLI](https://developer.1password.com/docs/cli/) (`op`) for secret management (optional)

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TWENTYCRM_API_TOKEN` | Yes | Bearer token for the TwentyCRM REST API |
| `TWENTYCRM_API_URL` | No | API base URL (default: `https://api.twenty.com/rest`) |
| `REQUIRED_GROUPS` | Yes | Comma-separated OIDC group names for access control |
| `OIDC_ISSUER_URL` | Yes | OIDC provider URL (e.g. `https://gitlab.example.com`) |
| `OIDC_CLIENT_ID` | Yes | OIDC client ID |
| `OIDC_CLIENT_SECRET` | Yes | OIDC client secret |
| `BASE_URL` | Yes | Public URL of the service |
| `REDIS_URL` | Yes | Redis connection URL |
| `ENCRYPTION_KEY` | Yes | 32-byte AES-256 key, base64-encoded |
| `BIND_ADDR` | No | Listen address (default: `0.0.0.0:8080`) |
| `RUST_LOG` | No | Log level filter (default: `info`) |

Copy `.env.example` and fill in your values:

```bash
cp .env.example .env
```

## Local Development (Docker Compose + 1Password)

The dev stack runs the MCP server, Redis, and an ngrok tunnel. Secrets are injected at runtime via `op run`.

Create `.env.1password` with `op://` references for secrets (see `.env.example` for the template), then:

```bash
op run --env-file=.env.1password -- docker compose -f docker-compose.dev.yml up -d --build
```

Additional env vars for the dev stack:

| Variable | Description |
|----------|-------------|
| `NGROK_AUTHTOKEN` | ngrok auth token |
| `NGROK_DOMAIN` | Optional: reserved ngrok domain for a stable URL |

Check the ngrok tunnel URL:

```bash
# If using a reserved domain, it's your NGROK_DOMAIN value.
# Otherwise, check the ngrok dashboard:
curl -s http://localhost:4040/api/tunnels | jq '.tunnels[0].public_url'
```

View logs:

```bash
docker compose -f docker-compose.dev.yml logs -f twentycrm-mcp
```

## Local Development (Cargo)

```bash
# Start Redis
docker compose -f docker-compose.dev.yml up -d redis

# Run the server
cargo run
```

## Available Tools

| Entity | Search | View | Create | Update |
|--------|--------|------|--------|--------|
| Companies | `search_companies` | `view_company` | `create_company` | `update_company` |
| Contacts | `search_contacts` | `view_contact` | `create_contact` | `update_contact` |
| Opportunities | `search_opportunities` | `view_opportunity` | `create_opportunity` | `update_opportunity` |
| Tasks | `search_tasks` | `view_task` | `create_task` | `update_task` |

## Connecting from claude.ai

1. Go to claude.ai Settings > Integrations
2. Add a new MCP server with your `BASE_URL`
3. Authenticate via your OIDC provider when prompted
4. The 16 CRM tools will appear in the tool list

## Testing

```bash
cargo test
cargo test --test integration
RUST_LOG=debug cargo test -- --nocapture
```

## Commands

```bash
cargo build          # Build
cargo clippy         # Lint
cargo fmt            # Format
```
