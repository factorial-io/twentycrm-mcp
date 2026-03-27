# Tasks: TwentyCRM MCP Server

**Input**: Design documents from `/specs/001-twentycrm-mcp-server/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/mcp-tools.md
**Blueprint**: [factorial-io/imap-mcp](https://github.com/factorial-io/imap-mcp)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, Cargo.toml, and scaffolding

- [x] T001 Initialize Rust project with `cargo init`, configure Cargo.toml with all dependencies from plan.md (axum 0.8, rmcp 1.2.0, openidconnect 3.5.0, reqwest, redis, serde, schemars, thiserror, anyhow, tracing, etc.)
- [x] T002 [P] Create .env.example with all environment variables (TWENTYCRM_API_TOKEN, TWENTYCRM_API_URL, ALLOWED_EMAILS, OIDC_ISSUER_URL, OIDC_CLIENT_ID, OIDC_CLIENT_SECRET, BASE_URL, REDIS_URL, ENCRYPTION_KEY, BIND_ADDR, RUST_LOG)
- [x] T003 [P] Create Dockerfile and docker-compose.yml / docker-compose.dev.yml following imap-mcp patterns
- [x] T004 [P] Create src/main.rs with env_var()/env_var_or() helpers, tracing setup, and Tokio runtime bootstrap (adapted from imap-mcp main.rs)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Implement AppError enum in src/error.rs with variants: Redis, Auth, SessionNotFound, TwentyCrmApi, NotFound, Validation, Encryption, Internal — each mapped to HTTP status codes, using thiserror (adapted from imap-mcp error.rs)
- [x] T006 [P] Implement Redis session storage with AES-256-GCM encryption in src/session.rs — session create, get, delete, TTL refresh (adapted from imap-mcp session.rs)
- [x] T007 [P] Implement OAuth/OIDC flow in src/auth.rs — well-known endpoints (oauth-protected-resource, oauth-authorization-server), dynamic client registration (POST /register), login initiation (GET /auth/login with PKCE S256), callback handler (GET /auth/callback with email whitelist check), token exchange (POST /auth/token) (adapted from imap-mcp auth.rs)
- [x] T008 [P] Define shared types in src/types.rs — Company, Person, Opportunity, Task entity structs with serde Deserialize, API response wrapper (ListResponse with data + pageInfo), common tool parameter structs (SearchParams, PaginationParams) with schemars JsonSchema derive
- [x] T009 Implement TwentyCRM REST API client in src/twentycrm.rs — TwentyCrmClient struct with reqwest::Client, base_url, api_token; methods: search(entity, filter, order_by, limit, cursor, fields), get(entity, id, fields), create(entity, body), update(entity, id, body); filter/pagination/ordering query string builders; error mapping from HTTP status codes to AppError
- [x] T010 Implement AppState, build_router(), and mcp_handler in src/lib.rs — AppState holding OIDC client, Redis pool, encryption key, TwentyCrmClient, allowed_emails; build_router() mounting auth routes + MCP endpoint at /mcp and /mcp/{path} via axum::routing::any(); mcp_handler extracting bearer token, looking up session in Redis, creating per-request TwentyCrmMcpServer, delegating to StreamableHttpService (adapted from imap-mcp lib.rs)
- [x] T011 Wire main.rs to call build_router() and start Axum server on BIND_ADDR in src/main.rs — construct AppState from env vars, initialize OIDC client, connect Redis, create TwentyCrmClient, bind and serve

**Checkpoint**: Foundation ready — OAuth flow works, Redis sessions work, TwentyCRM API client works, MCP endpoint returns 401 without token. User story implementation can now begin.

---

## Phase 3: User Story 1 — Authentication and Tool Discovery (Priority: P1) 🎯 MVP

**Goal**: Users can authenticate via OAuth OIDC, get verified against the email whitelist, and see all 16 MCP tools listed with descriptions and parameter schemas.

**Independent Test**: Start server, initiate OAuth flow, complete authentication, verify tool list is returned with 16 tools.

### Implementation for User Story 1

- [x] T012 [US1] Create TwentyCrmMcpServer struct in src/mcp.rs — holds TwentyCrmClient reference; implement rmcp server handler trait; register all 16 tools with names, descriptions, and parameter schemas (initially returning placeholder "not yet implemented" responses)
- [x] T013 [US1] Verify full OAuth flow end-to-end in src/auth.rs — ensure callback checks email against ALLOWED_EMAILS, denies access for non-whitelisted emails with clear error, creates session for whitelisted users
- [x] T014 [US1] Verify MCP tool listing in src/lib.rs — ensure authenticated requests to /mcp return all 16 tools with correct names (search_companies, view_company, create_company, update_company, etc.), descriptions, and JSON Schema parameter definitions

**Checkpoint**: User Story 1 complete — authenticated users see all 16 tools. Non-whitelisted users are denied. Tool calls return placeholder responses.

---

## Phase 4: User Story 2 — Search and View Companies (Priority: P1)

**Goal**: Users can search companies by name/domain/attributes with pagination, and view full company details by ID.

**Independent Test**: Search for companies by name, paginate results, view a single company's full details.

### Implementation for User Story 2

- [x] T015 [P] [US2] Implement search_companies tool in src/mcp.rs — accept SearchCompaniesParams (query, filter, order_by, limit, starting_after, fields via schemars); call TwentyCrmClient::search("companies", ...); return formatted results with pageInfo; handle empty results gracefully
- [x] T016 [P] [US2] Implement view_company tool in src/mcp.rs — accept ViewCompanyParams (id: String, fields: Option<Vec<String>>); call TwentyCrmClient::get("companies", id, fields); return full company record; handle not-found with clear error

**Checkpoint**: User Story 2 complete — company search with pagination and field selection works. Company detail view works.

---

## Phase 5: User Story 3 — Search and View Contacts (Priority: P1)

**Goal**: Users can search contacts by name/email/phone/company and view full contact details.

**Independent Test**: Search contacts by email, view a contact's details including company association.

### Implementation for User Story 3

- [x] T017 [P] [US3] Implement search_contacts tool in src/mcp.rs — accept SearchContactsParams (query, filter, order_by, limit, starting_after, fields); call TwentyCrmClient::search("people", ...); return formatted results with pageInfo
- [x] T018 [P] [US3] Implement view_contact tool in src/mcp.rs — accept ViewContactParams (id, fields); call TwentyCrmClient::get("people", id, fields); return full contact record including companyId association

**Checkpoint**: User Story 3 complete — contact search and detail view works independently.

---

## Phase 6: User Story 4 — Create New Records (Priority: P1)

**Goal**: Users can create new companies, contacts, opportunities, and tasks directly from the AI assistant.

**Independent Test**: Create a new company, verify it appears in subsequent searches.

### Implementation for User Story 4

- [x] T019 [P] [US4] Implement create_company tool in src/mcp.rs — accept CreateCompanyParams (name, domainName, employees, industry, address, etc. as Option fields with schemars); call TwentyCrmClient::create("companies", body); return created record
- [x] T020 [P] [US4] Implement create_contact tool in src/mcp.rs — accept CreateContactParams (firstName, lastName, email, phone, jobTitle, companyId, etc.); call TwentyCrmClient::create("people", body); return created record
- [x] T021 [P] [US4] Implement create_opportunity tool in src/mcp.rs — accept CreateOpportunityParams (name, stage, closeDate, amount, companyId, pointOfContactId, etc.); call TwentyCrmClient::create("opportunities", body); return created record
- [x] T022 [P] [US4] Implement create_task tool in src/mcp.rs — accept CreateTaskParams (title, status, dueAt, assigneeId, body, targetCompanyId, targetPersonId, targetOpportunityId); call TwentyCrmClient::create("tasks", body); if target IDs provided, also create TaskTarget records via separate API call; return created record

**Checkpoint**: User Story 4 complete — all four entity types can be created. Validation errors from TwentyCRM API are returned clearly.

---

## Phase 7: User Story 5 — Update Records (Priority: P1)

**Goal**: Users can update fields on existing companies, contacts, opportunities, and tasks.

**Independent Test**: Update a company's domain field, verify the change persists when viewing the record.

### Implementation for User Story 5

- [x] T023 [P] [US5] Implement update_company tool in src/mcp.rs — accept UpdateCompanyParams (id: String required, plus all writable company fields as Option); call TwentyCrmClient::update("companies", id, body); return updated record; handle not-found and validation errors
- [x] T024 [P] [US5] Implement update_contact tool in src/mcp.rs — accept UpdateContactParams (id required, plus writable contact fields as Option); call TwentyCrmClient::update("people", id, body); return updated record
- [x] T025 [P] [US5] Implement update_opportunity tool in src/mcp.rs — accept UpdateOpportunityParams (id required, plus writable opportunity fields as Option); call TwentyCrmClient::update("opportunities", id, body); return updated record
- [x] T026 [P] [US5] Implement update_task tool in src/mcp.rs — accept UpdateTaskParams (id required, plus writable task fields as Option); call TwentyCrmClient::update("tasks", id, body); return updated record

**Checkpoint**: User Story 5 complete — all four entity types can be updated. Not-found and validation errors are handled.

---

## Phase 8: User Story 6 — Search and View Opportunities (Priority: P2)

**Goal**: Users can search opportunities by stage/company/name and view opportunity details.

**Independent Test**: List opportunities filtered by stage, view a single opportunity's full details.

### Implementation for User Story 6

- [x] T027 [P] [US6] Implement search_opportunities tool in src/mcp.rs — accept SearchOpportunitiesParams (query, filter, order_by, limit, starting_after, fields); call TwentyCrmClient::search("opportunities", ...); return formatted results with pageInfo
- [x] T028 [P] [US6] Implement view_opportunity tool in src/mcp.rs — accept ViewOpportunityParams (id, fields); call TwentyCrmClient::get("opportunities", id, fields); return full opportunity record

**Checkpoint**: User Story 6 complete — opportunity search and detail view works.

---

## Phase 9: User Story 7 — Search and View Tasks (Priority: P2)

**Goal**: Users can search tasks by status/due date/assignee and view task details.

**Independent Test**: List tasks filtered by status, view a single task's details.

### Implementation for User Story 7

- [x] T029 [P] [US7] Implement search_tasks tool in src/mcp.rs — accept SearchTasksParams (query, filter, order_by, limit, starting_after, fields); call TwentyCrmClient::search("tasks", ...); return formatted results with pageInfo
- [x] T030 [P] [US7] Implement view_task tool in src/mcp.rs — accept ViewTaskParams (id, fields); call TwentyCrmClient::get("tasks", id, fields); return full task record including taskTargets

**Checkpoint**: User Story 7 complete — task search and detail view works.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, documentation, and hardening

- [ ] T031 [P] Create integration tests in tests/integration.rs — test OAuth well-known endpoints return correct JSON, test /register rejects empty redirect URIs, test /mcp returns 401 without bearer token, test /mcp returns 401 with invalid token, test tool parameter deserialization with defaults (adapted from imap-mcp integration tests)
- [x] T032 [P] Add unit tests in src/error.rs — test status code mapping for each AppError variant, test display messages
- [x] T033 [P] Add unit tests in src/session.rs — test encryption/decryption roundtrips, key validation, serialization roundtrips
- [x] T034 [P] Add unit tests in src/types.rs — test serde deserialization of TwentyCRM API responses, test schemars JSON Schema generation for tool params
- [ ] T035 Run cargo clippy and cargo fmt, fix all warnings
- [ ] T036 Validate quickstart.md — verify setup instructions work end-to-end (cargo build, env config, Docker compose)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3–9)**: All depend on Foundational phase completion
  - US1 (auth + tool discovery) should complete first as it validates the MCP endpoint
  - US2–US7 can proceed in parallel after US1
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Auth + Tool Discovery)**: Can start after Foundational — no dependencies on other stories. Delivers placeholder tool registrations.
- **US2 (Companies Search/View)**: Can start after US1 — needs tool registrations to be in place
- **US3 (Contacts Search/View)**: Can start after US1 — independent of US2
- **US4 (Create Records)**: Can start after US1 — independent of US2/US3
- **US5 (Update Records)**: Can start after US1 — independent of US2/US3/US4
- **US6 (Opportunities Search/View)**: Can start after US1 — independent of other stories
- **US7 (Tasks Search/View)**: Can start after US1 — independent of other stories

### Within Each User Story

- Tool parameter structs (types) → tool handler implementation → error handling
- All tools within a story that touch different entities can be parallelized [P]

### Parallel Opportunities

- T002, T003, T004 can run in parallel (Setup phase)
- T005, T006, T007, T008 can run in parallel (Foundational — different files)
- After US1 completes, US2–US7 can all run in parallel
- Within US4: T019, T020, T021, T022 can all run in parallel (different tool handlers)
- Within US5: T023, T024, T025, T026 can all run in parallel
- T031, T032, T033, T034 can all run in parallel (different test files)

---

## Parallel Example: User Story 4 (Create Records)

```text
# Launch all create tool implementations together (all [P] marked):
Task T019: "Implement create_company tool in src/mcp.rs"
Task T020: "Implement create_contact tool in src/mcp.rs"
Task T021: "Implement create_opportunity tool in src/mcp.rs"
Task T022: "Implement create_task tool in src/mcp.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (Cargo.toml, .env, Dockerfile, main.rs scaffold)
2. Complete Phase 2: Foundational (error.rs, session.rs, auth.rs, types.rs, twentycrm.rs, lib.rs)
3. Complete Phase 3: User Story 1 (auth flow + 16 placeholder tools)
4. Complete Phase 4: User Story 2 (search + view companies — first real tools)
5. **STOP and VALIDATE**: End-to-end test — authenticate, search companies, view company
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (Auth + Tool Discovery) → Test → Deploy (auth works, tools listed)
3. US2 (Companies) → Test → Deploy (first real CRM data accessible)
4. US3 (Contacts) → Test → Deploy (people lookup works)
5. US4 (Create) → Test → Deploy (write operations enabled)
6. US5 (Update) → Test → Deploy (full read/write CRM)
7. US6 (Opportunities) + US7 (Tasks) → Test → Deploy (complete feature set)
8. Polish → Final deploy

---

## Notes

- [P] tasks = different files or independent operations, no dependencies
- [Story] label maps task to specific user story for traceability
- All tool handlers live in src/mcp.rs (single file, following imap-mcp pattern)
- Entity types and tool param structs live in src/types.rs
- auth.rs, session.rs, error.rs are adapted from imap-mcp with minimal changes
- twentycrm.rs is the new domain-specific module (replaces imap.rs from blueprint)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
