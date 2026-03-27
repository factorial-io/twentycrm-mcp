# Feature Specification: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Created**: 2026-03-27
**Status**: Draft
**Input**: User description: "Build an MCP server for TwentyCRM that exposes CRM operations as tools"

## User Scenarios & Testing

### User Story 1 - Server Discovery and Authentication (Priority: P1)

A user wants to set up the MCP server for the first time. They provide their TwentyCRM API token, and the server exposes all available tools to the MCP client. The user can discover what operations are available without reading documentation.

**Why this priority**: Without authentication and tool discovery, no other functionality works. This is the foundation.

**Independent Test**: Can be fully tested by starting the server with a valid API token and verifying that the MCP client receives the list of available tools.

**Acceptance Scenarios**:

1. **Given** a user has a TwentyCRM API token, **When** they configure the MCP server with the token, **Then** the server starts and exposes available tools to the MCP client
2. **Given** a running MCP server, **When** the MCP client requests the tool list, **Then** all available CRM tools are returned with descriptions and parameter schemas
3. **Given** an invalid or expired API token, **When** the server attempts to connect, **Then** a clear authentication error is returned to the user

---

### User Story 2 - Manage Companies via AI Assistant (Priority: P1)

A user working in an AI-powered assistant (e.g., Claude Code, Cursor, or any MCP-compatible client) wants to search, view, create, and update company records in their TwentyCRM workspace without leaving their workflow. They connect the MCP server, authenticate once, and then use tool calls to interact with their CRM data.

**Why this priority**: Companies are a core CRM entity and the most common starting point for sales workflows. This validates the full end-to-end MCP integration pattern.

**Independent Test**: Can be fully tested by connecting the MCP server to an MCP client and performing CRUD operations on the companies endpoint. Delivers immediate value for users managing company pipelines.

**Acceptance Scenarios**:

1. **Given** a configured MCP server with valid TwentyCRM credentials, **When** the user requests to list companies, **Then** the server returns a paginated list of company records with key fields (name, domain, employees, etc.)
2. **Given** a configured MCP server, **When** the user requests to create a company with a name and domain, **Then** a new company record is created in TwentyCRM and the confirmation is returned
3. **Given** a configured MCP server, **When** the user requests to find a specific company by name, **Then** the server returns matching records using the filtering capabilities of the API
4. **Given** a configured MCP server, **When** the user requests to update a company field, **Then** the field is updated in TwentyCRM and the updated record is returned

---

### User Story 3 - Manage People and Contacts (Priority: P1)

A user wants to look up, create, and update contact/people records associated with companies. They can search contacts by name or email, view their details, and update information as conversations progress.

**Why this priority**: People/contacts are the second core CRM entity, tightly linked to companies. Together with companies, they form the minimum viable CRM workflow.

**Independent Test**: Can be fully tested by performing CRUD operations on the people endpoint, including filtering by email and linking to companies.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user requests to list people, **Then** the server returns a paginated list of contact records
2. **Given** a configured MCP server, **When** the user creates a new person with name and email, **Then** the person is created and linked to the specified company if provided
3. **Given** a configured MCP server, **When** the user searches for a person by email, **Then** matching contacts are returned

---

### User Story 4 - Track Opportunities and Deals (Priority: P2)

A user wants to view and manage sales opportunities. They can list open deals, create new opportunities linked to companies, and update deal stages as they progress through the pipeline.

**Why this priority**: Opportunities represent the sales pipeline, which is the primary revenue-generating workflow in a CRM. Depends on companies existing first.

**Independent Test**: Can be fully tested by performing CRUD operations on opportunities, including filtering by stage and linking to companies.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user lists opportunities, **Then** open deals are returned with stage, amount, and associated company
2. **Given** a configured MCP server, **When** the user creates an opportunity with a name, stage, and company link, **Then** the opportunity is created in TwentyCRM
3. **Given** a configured MCP server, **When** the user updates a deal stage, **Then** the stage change is persisted and the updated record is returned

---

### User Story 5 - Manage Notes and Tasks (Priority: P2)

A user wants to create notes against contacts or companies to track conversation history, and create tasks to follow up on action items. This keeps all CRM context within the AI assistant workflow.

**Why this priority**: Notes and tasks are essential for day-to-day CRM usage but depend on companies and people existing first.

**Independent Test**: Can be fully tested by creating notes/tasks and linking them to existing company or person records.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user creates a note with body text and a target (company or person), **Then** the note is created and linked to the target
2. **Given** a configured MCP server, **When** the user creates a task with a title and due date, **Then** the task is created in TwentyCRM
3. **Given** a configured MCP server, **When** the user lists tasks, **Then** open tasks are returned with their targets and due dates

---

### Edge Cases

- What happens when the API token expires mid-session? The server returns a clear authentication error prompting the user to re-authenticate.
- What happens when a requested record does not exist? The server returns a clear "not found" message rather than a generic error.
- What happens when the TwentyCRM API rate limit is hit? The server returns an informative message indicating the rate limit and suggesting the user wait.
- What happens when required fields are missing from a create/update request? The server returns validation errors indicating which fields are required.
- What happens when pagination reaches the end of results? The server indicates no more results are available via pagination metadata.
- What happens when the TwentyCRM instance is unreachable? The server returns a connection error with guidance on checking the server URL and network.

## Requirements

### Functional Requirements

- **FR-001**: System MUST authenticate with TwentyCRM using a bearer token provided via configuration
- **FR-002**: System MUST expose CRM operations as MCP tools following the Model Context Protocol specification
- **FR-003**: System MUST support CRUD operations (create, read, update, delete) for companies
- **FR-004**: System MUST support CRUD operations for people/contacts
- **FR-005**: System MUST support CRUD operations for opportunities
- **FR-006**: System MUST support CRUD operations for notes, including linking notes to target entities (companies, people)
- **FR-007**: System MUST support CRUD operations for tasks, including linking tasks to target entities
- **FR-008**: System MUST support filtering and searching records using the TwentyCRM filter syntax (e.g., by name, email, stage, date ranges)
- **FR-009**: System MUST support cursor-based pagination for list operations, allowing users to navigate through large result sets
- **FR-010**: System MUST return clear, human-readable error messages for authentication failures, validation errors, not-found conditions, and API errors
- **FR-011**: System MUST expose tool descriptions and parameter schemas so MCP clients can present available operations to users
- **FR-012**: System MUST support ordering/sorting results by standard fields (e.g., createdAt, name, updatedAt)
- **FR-013**: System MUST communicate over stdio transport as the primary MCP transport mechanism

### Key Entities

- **Company**: A business organization tracked in the CRM. Key attributes: name, domain, employee count, address, industry. Central entity linked to people, opportunities, and notes.
- **Person**: An individual contact. Key attributes: name (first/last), email, phone, job title, city. Linked to a company.
- **Opportunity**: A sales deal or potential revenue. Key attributes: name, stage, amount, close date, probability. Linked to a company and a point-of-contact person.
- **Note**: A free-text record of information. Key attributes: title, body, creation date. Linked to target entities (company, person) via note targets.
- **Task**: An action item or follow-up. Key attributes: title, body, due date, status, assignee. Linked to target entities via task targets.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can perform all core CRM operations (list, search, create, update, delete) for companies, people, opportunities, notes, and tasks without leaving their AI assistant
- **SC-002**: Users can discover and understand all available tools within 2 minutes of connecting the MCP server, without consulting external documentation
- **SC-003**: All operations return results within 5 seconds under normal network conditions
- **SC-004**: Error messages are specific enough for users to self-resolve 90% of issues (e.g., missing required fields, invalid token, record not found)
- **SC-005**: Users can filter and paginate through 1000+ records without performance degradation or data loss
- **SC-006**: Setup and first successful operation can be completed within 5 minutes using only the provided configuration instructions

## Assumptions

- Users have an active TwentyCRM workspace with API access enabled
- Users have generated a valid API token from TwentyCRM Settings
- The MCP client (e.g., Claude Code, Cursor) supports the MCP protocol and stdio transport
- The TwentyCRM REST API at api.twenty.com is available and follows the documented OpenAPI specification
- The initial release focuses on the five core entities (companies, people, opportunities, notes, tasks); additional entities (workflows, calendar events, messages, etc.) are out of scope for v1
- The server runs locally on the user's machine, connecting to TwentyCRM's cloud API
- Rate limits imposed by TwentyCRM are handled with informative user-facing messages, not automatic retries
