# Changelog

## [0.2.0](https://github.com/factorial-io/twentycrm-mcp/compare/twentycrm-mcp-v0.1.0...twentycrm-mcp-v0.2.0) (2026-11-09)


### ⚠ BREAKING CHANGES

* ENCRYPTION_KEY is replaced by TOKEN_SIGNING_KEY. Existing sessions are invalid; users log in once more. Clients must connect to <BASE_URL>/mcp, the OAuth resource.

### Features

* add campaign CRUD tools and company-campaign linking ([4a8ef50](https://github.com/factorial-io/twentycrm-mcp/commit/4a8ef50f14d7d92a7a3ec3aebce8edd22d034ab5))
* add campaign CRUD tools and company-campaign linking ([20abdb9](https://github.com/factorial-io/twentycrm-mcp/commit/20abdb956ce9fe135d3d57ea23e7a8748ce35a9f))
* add messages/calendar tools, fix note/task target linking ([e86c1f5](https://github.com/factorial-io/twentycrm-mcp/commit/e86c1f5b9f9a92117307ed5efb3099bb54146da4))
* fix OAuth flow, add notes/timeline/delete tools, improve DX ([7981120](https://github.com/factorial-io/twentycrm-mcp/commit/7981120fdfe0681911a690b261e54109f59ac030))
* implement TwentyCRM MCP server (Phases 1-9 complete) ([0cd20ae](https://github.com/factorial-io/twentycrm-mcp/commit/0cd20ae83466a64d5111596984ce6c5a4e267f9c))
* replace email whitelist with OIDC group membership check ([788ff40](https://github.com/factorial-io/twentycrm-mcp/commit/788ff409691b9952a874f2848329c45b9825d7b7))
* replace hand-rolled OAuth with grantry ([001088d](https://github.com/factorial-io/twentycrm-mcp/commit/001088d29b822a86e865c68583957892f0653b83))


### Bug Fixes

* resolve rustfmt and clippy warnings ([c34824c](https://github.com/factorial-io/twentycrm-mcp/commit/c34824c03f662f3e1ba7c0f865f2e60400ca3b0a))
