# Research: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Date**: 2026-03-27

## Decision 1: Language & Runtime

**Decision**: TypeScript on Node.js
**Rationale**: The official MCP SDK is TypeScript-first. The `@modelcontextprotocol/sdk` package provides built-in support for server creation, tool registration, and streamable HTTP transport. TypeScript gives type safety for the 16 MCP tool schemas.
**Alternatives considered**:
- Python (official SDK exists but TypeScript has stronger MCP ecosystem tooling for remote servers)
- Go (no official MCP SDK)

## Decision 2: MCP SDK & Transport

**Decision**: `@modelcontextprotocol/sdk` v1.x with streamable HTTP transport
**Rationale**: v1.x is the recommended production version. Streamable HTTP transport has been supported since v1.10.0 (April 2025) and is the modern transport for remote servers. v2 is anticipated Q1 2026 but v1.x continues to receive bug fixes.
**Alternatives considered**:
- SDK v2 (not yet stable for production)
- SSE transport (deprecated in favor of streamable HTTP since MCP spec version 2025-03-26)
- stdio (only for local process-spawned servers, not remote hosted)

**Key packages**:
- `@modelcontextprotocol/sdk` — core MCP server
- `@modelcontextprotocol/node` — Node.js streamable HTTP transport wrapper
- `@modelcontextprotocol/express` — Express integration helpers
- `zod` — schema validation (required peer dependency)

## Decision 3: HTTP Framework

**Decision**: Express.js
**Rationale**: The MCP SDK provides `@modelcontextprotocol/express` helpers specifically for Express integration with `NodeStreamableHTTPServerTransport`. Express is the most documented path for remote MCP servers.
**Alternatives considered**:
- Hono (`@modelcontextprotocol/hono` available but less documented for this use case)
- Raw Node.js HTTP (`@modelcontextprotocol/node` available but Express simplifies middleware)

## Decision 4: Authentication Architecture

**Decision**: OAuth OIDC via MCP SDK `requireBearerAuth` middleware + email whitelist check
**Rationale**: The MCP SDK provides built-in `requireBearerAuth` middleware that verifies tokens and extracts user identity. The `authInfo` object is available in tool handler contexts. The server checks the verified email against the whitelist configured via environment variable. The TwentyCRM API token is stored server-side as an environment variable.
**Alternatives considered**:
- Custom middleware (unnecessary when SDK provides built-in auth support)
- `mcp-auth` library (adds dependency; SDK built-ins sufficient for email whitelist pattern)

**Auth flow**:
1. MCP client (claude.ai) authenticates user via OAuth OIDC
2. Client sends bearer token with each request
3. `requireBearerAuth` middleware verifies token, extracts user identity
4. Server checks user email from `authInfo` against `ALLOWED_EMAILS` env var
5. Server makes TwentyCRM API calls using `TWENTYCRM_API_TOKEN` env var

## Decision 5: TwentyCRM API Client

**Decision**: HTTP client (fetch or axios) calling TwentyCRM REST API directly
**Rationale**: TwentyCRM provides a well-documented REST API with OpenAPI spec. No official TwentyCRM SDK for Node.js exists. The REST endpoints follow consistent patterns (CRUD per entity, filter syntax, cursor pagination) that are straightforward to wrap.
**Alternatives considered**:
- GraphQL (TwentyCRM supports it but REST is simpler and documented in the OpenAPI specs we have)
- Auto-generated client from OpenAPI spec (adds complexity; the 4 entities and their endpoints are manageable manually)

## Decision 6: Testing Framework

**Decision**: Vitest
**Rationale**: Fast, TypeScript-native, compatible with Node.js. Supports mocking for HTTP calls to TwentyCRM API. Works well for both unit tests (tool handlers, auth logic) and integration tests (MCP protocol compliance).
**Alternatives considered**:
- Jest (heavier, requires more configuration for TypeScript)
- Node.js built-in test runner (less ecosystem support for mocking)

## Decision 7: Project Structure

**Decision**: Single project, flat src directory with domain-organized modules
**Rationale**: The server is a single deployable service with 16 tools, a shared auth layer, and a shared TwentyCRM API client. No need for monorepo or multi-package structure.

```
src/
├── server.ts              # MCP server setup, tool registration
├── auth/                  # OAuth middleware, whitelist check
├── tools/                 # 16 MCP tool handlers
│   ├── companies/         # search, view, create, update
│   ├── contacts/          # search, view, create, update
│   ├── opportunities/     # search, view, create, update
│   └── tasks/             # search, view, create, update
├── api/                   # TwentyCRM REST API client
│   └── client.ts          # HTTP client, auth, error handling
└── utils/                 # Shared utilities (pagination, filtering, field selection)
```

## Decision 8: Environment Configuration

**Decision**: Environment variables for all configuration
**Rationale**: Standard for hosted services. Keeps secrets out of code and config files.

| Variable | Purpose |
|----------|---------|
| `TWENTYCRM_API_TOKEN` | Bearer token for TwentyCRM API |
| `TWENTYCRM_API_URL` | TwentyCRM API base URL (default: `https://api.twenty.com/rest`) |
| `ALLOWED_EMAILS` | Comma-separated list of whitelisted user emails |
| `PORT` | Server port (default: 3000) |

## Decision 9: Stateless vs Stateful Transport

**Decision**: Stateless streamable HTTP
**Rationale**: The server acts as a stateless proxy to the TwentyCRM API. No session state needs to be maintained between requests. Stateless mode is simpler to deploy and scale.
**Alternatives considered**:
- Stateful with session IDs (unnecessary; no cross-request state to maintain)
