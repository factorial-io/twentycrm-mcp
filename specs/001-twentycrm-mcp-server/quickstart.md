# Quickstart: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Date**: 2026-03-27

## Prerequisites

- Node.js 20+
- A TwentyCRM workspace with API access
- A TwentyCRM API token (generated from Settings → Playground in TwentyCRM)

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TWENTYCRM_API_TOKEN` | Yes | Bearer token for TwentyCRM REST API |
| `TWENTYCRM_API_URL` | No | API base URL (default: `https://api.twenty.com/rest`) |
| `ALLOWED_EMAILS` | Yes | Comma-separated list of authorized user emails |
| `PORT` | No | Server port (default: 3000) |

## Setup

```bash
# Install dependencies
npm install

# Configure environment
cp .env.example .env
# Edit .env with your values

# Run in development
npm run dev

# Run in production
npm start
```

## Connecting from claude.ai

1. Navigate to claude.ai Settings → Integrations
2. Add a new MCP server with the hosted URL
3. Authenticate via OAuth OIDC when prompted
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
# Run tests
npm test

# Run tests in watch mode
npm run test:watch
```
