# Implementation Plan: TwentyCRM MCP Server

**Branch**: `001-twentycrm-mcp-server` | **Date**: 2026-03-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-twentycrm-mcp-server/spec.md`

## Summary

Build a remote MCP server that exposes TwentyCRM CRM operations as 16 MCP tools (search, view, create, update for companies, contacts, opportunities, and tasks). The server uses streamable HTTP transport, authenticates users via OAuth OIDC with an email whitelist, and proxies requests to the TwentyCRM REST API using a shared API token. Built with TypeScript, the official MCP SDK, and Express.

## Technical Context

**Language/Version**: TypeScript 5.x on Node.js 20+
**Primary Dependencies**: `@modelcontextprotocol/sdk`, `@modelcontextprotocol/express`, `express`, `zod`
**Storage**: N/A (stateless proxy to TwentyCRM API)
**Testing**: Vitest
**Target Platform**: Linux server (remote hosted, accessible via streamable HTTP)
**Project Type**: Web service (MCP server)
**Performance Goals**: <5s response time per tool call under normal network conditions
**Constraints**: Stateless — no server-side session state; single shared TwentyCRM workspace
**Scale/Scope**: Team-sized usage (whitelisted users), 4 CRM entities, 16 MCP tools

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution is an unfilled template — no gates defined. No violations to check.

**Post-Phase 1 re-check**: N/A — no constitution gates active.

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
├── index.ts                    # Entry point — Express server + MCP setup
├── server.ts                   # McpServer creation, tool registration
├── auth/
│   ├── middleware.ts            # OAuth bearer auth + email whitelist check
│   └── whitelist.ts            # Parse ALLOWED_EMAILS env var
├── api/
│   ├── client.ts               # TwentyCRM REST API HTTP client
│   ├── types.ts                # TwentyCRM API response types
│   └── errors.ts               # API error mapping to MCP error responses
├── tools/
│   ├── companies/
│   │   ├── search.ts           # search_companies tool
│   │   ├── view.ts             # view_company tool
│   │   ├── create.ts           # create_company tool
│   │   └── update.ts           # update_company tool
│   ├── contacts/
│   │   ├── search.ts           # search_contacts tool
│   │   ├── view.ts             # view_contact tool
│   │   ├── create.ts           # create_contact tool
│   │   └── update.ts           # update_contact tool
│   ├── opportunities/
│   │   ├── search.ts           # search_opportunities tool
│   │   ├── view.ts             # view_opportunity tool
│   │   ├── create.ts           # create_opportunity tool
│   │   └── update.ts           # update_opportunity tool
│   └── tasks/
│       ├── search.ts           # search_tasks tool
│       ├── view.ts             # view_task tool
│       ├── create.ts           # create_task tool
│       └── update.ts           # update_task tool
└── utils/
    ├── pagination.ts           # Cursor-based pagination helpers
    ├── filtering.ts            # Filter query builder
    ├── fields.ts               # Field selection (default + custom)
    └── ordering.ts             # Order_by query builder

tests/
├── unit/
│   ├── auth/                   # Whitelist parsing, middleware logic
│   ├── api/                    # API client, error mapping
│   ├── tools/                  # Individual tool handler tests
│   └── utils/                  # Pagination, filtering, field selection
└── integration/
    ├── server.test.ts          # MCP protocol compliance
    └── tools.test.ts           # End-to-end tool calls with mocked API
```

**Structure Decision**: Single project with domain-organized tools directory. Each entity gets its own subdirectory under `tools/` with one file per operation. Shared concerns (API client, auth, utilities) are extracted into their own modules. This keeps each tool handler focused and independently testable.

## Complexity Tracking

No constitution gates defined — no violations to justify.
