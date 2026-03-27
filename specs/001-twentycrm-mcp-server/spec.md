# Feature Specification: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Created**: 2026-03-27
**Status**: Draft
**Input**: User description: "The TwentyCRM MCP server should expose the most important endpoints of the TwentyCRM API via MCP. The user should be able to search for contacts, companies, opportunities, tasks, view their respective details and update the information. Auth against TwentyCRM via API token. App access controlled by a whitelist of allowed email addresses verified via OAuth OIDC from the MCP client. Create operations included, delete out of scope."

## Clarifications

### Session 2026-03-27

- Q: Deployment & transport model — stdio (local) or streamable HTTP (remote hosted)? → A: Remote hosted server with streamable HTTP transport
- Q: Shared API token scope — single workspace, per-user tokens, or multi-tenant? → A: Single shared workspace — one API token, all whitelisted users see the same data
- Q: MCP tool granularity — fine-grained (16 tools), per-entity (4), or per-operation (4)? → A: Fine-grained — one tool per operation per entity (16 tools total)
- Q: Field exposure strategy — all fields always, summary for search / full for view, or configurable? → A: Configurable — users can specify which fields to return via a parameter

## User Scenarios & Testing

### User Story 1 - Authentication and Tool Discovery (Priority: P1)

A user wants to connect their AI assistant to their TwentyCRM workspace. The MCP server uses two layers of authentication: (1) the MCP client (e.g., claude.ai) authenticates the user via OAuth OIDC and the server checks their email against a whitelist of allowed users, and (2) the server uses a pre-configured TwentyCRM API token to make requests to the TwentyCRM API. Once the user is verified and the API connection is established, all available CRM tools are immediately exposed. The user can see what operations are available — searching, viewing, creating, and updating records — without needing to consult external documentation.

**Why this priority**: This is the foundation. Without successful authentication and tool discovery, no CRM operations are possible. The two-layer approach separates user access control (who can use the MCP server) from API access (how the server talks to TwentyCRM).

**Independent Test**: Can be fully tested by authenticating via the MCP client's OAuth flow, verifying the user's email is on the whitelist, and confirming the MCP client receives the complete list of available tools.

**Acceptance Scenarios**:

1. **Given** a user adds the TwentyCRM MCP server to their MCP client, **When** the client authenticates them via OAuth OIDC, **Then** the server verifies their email address against the configured whitelist
2. **Given** a user whose email is on the whitelist, **When** they complete authentication, **Then** the server connects to TwentyCRM using the configured API token and exposes all available CRM tools
3. **Given** a running MCP server, **When** the MCP client requests the tool list, **Then** tools for searching, viewing, creating, and updating contacts, companies, opportunities, and tasks are returned with clear descriptions and parameter definitions
4. **Given** a user whose email is NOT on the whitelist, **When** they attempt to authenticate, **Then** access is denied with a clear message explaining they are not authorized
5. **Given** an invalid or expired TwentyCRM API token, **When** the server attempts to make a request, **Then** a clear error message is returned explaining the API authentication failure

---

### User Story 2 - Search and View Companies (Priority: P1)

A user wants to find specific companies in their CRM. They search by company name, domain, or other attributes, browse through results, and view the full details of any company record — including linked contacts and opportunities.

**Why this priority**: Companies are the foundational CRM entity. Sales workflows start with identifying and understanding companies. This validates the core search-and-view pattern that all other entities follow.

**Independent Test**: Can be fully tested by searching for companies by name, paginating through results, and viewing a single company's complete details.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user searches for companies by name, **Then** matching company records are returned with key fields (name, domain, employee count, address)
2. **Given** a configured MCP server, **When** the user requests full details of a specific company, **Then** all company attributes are returned including associated metadata
3. **Given** a search that returns many results, **When** the user paginates through them, **Then** results are returned in pages with clear indicators of remaining results
4. **Given** a search with no matches, **When** results are returned, **Then** the user receives a clear empty result rather than an error

---

### User Story 3 - Search and View Contacts (Priority: P1)

A user wants to find people/contacts in the CRM. They search by name, email, or phone number, and view contact details including which company the person belongs to, their job title, and contact information.

**Why this priority**: Contacts are the second most important CRM entity. Together with companies, they form the minimum viable CRM lookup workflow that users need daily.

**Independent Test**: Can be fully tested by searching for contacts by email, viewing a contact's details, and verifying the company association is visible.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user searches for contacts by name or email, **Then** matching contact records are returned with key fields (name, email, phone, job title, company)
2. **Given** a configured MCP server, **When** the user requests full details of a specific contact, **Then** all contact attributes are returned including their associated company
3. **Given** a configured MCP server, **When** the user searches for contacts filtered by company, **Then** only contacts belonging to that company are returned

---

### User Story 4 - Create New Records (Priority: P1)

A user wants to add new data to the CRM directly from their AI assistant. After a prospecting call, they create a new company, add a new contact linked to that company, log a new opportunity, or create a follow-up task — all without leaving their workflow.

**Why this priority**: Creating records is essential for CRM adoption. If users must switch to the CRM UI to add data, the AI assistant becomes a read-only tool with limited value.

**Independent Test**: Can be fully tested by creating a new company record with required fields and verifying it appears in subsequent searches.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user creates a new company with a name and domain, **Then** the company is created in TwentyCRM and the full new record is returned as confirmation
2. **Given** a configured MCP server, **When** the user creates a new contact with name and email, **Then** the contact is created and optionally linked to a specified company
3. **Given** a configured MCP server, **When** the user creates a new opportunity with name, stage, and company, **Then** the opportunity is created in TwentyCRM
4. **Given** a configured MCP server, **When** the user creates a new task with a title and due date, **Then** the task is created in TwentyCRM
5. **Given** a configured MCP server, **When** the user omits a required field during creation, **Then** a validation error is returned listing the missing required fields

---

### User Story 5 - Update Records (Priority: P1)

A user has just had a meeting and wants to update CRM records with new information — a company's new address, a contact's updated job title, a deal's stage progression, or a task marked as completed. They locate the record and update specific fields without affecting other data.

**Why this priority**: Keeping CRM data current is critical for sales teams. The ability to update records directly from the AI assistant eliminates context-switching and ensures data is captured in the moment.

**Independent Test**: Can be fully tested by updating a field on a company record (e.g., changing the domain) and verifying the change persists when viewing the record again.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user updates a company's field (e.g., address, domain, employee count), **Then** the field is updated and the full updated record is returned as confirmation
2. **Given** a configured MCP server, **When** the user updates a contact's field (e.g., job title, email, phone), **Then** the field is updated and the full updated record is returned
3. **Given** a configured MCP server, **When** the user updates an opportunity's stage or amount, **Then** the change is persisted and the updated record is returned
4. **Given** a configured MCP server, **When** the user updates a task's status or details, **Then** the change is persisted and the updated record is returned
5. **Given** a configured MCP server, **When** the user attempts to update a record that does not exist, **Then** a clear "not found" error is returned
6. **Given** a configured MCP server, **When** the user provides an invalid value for a field, **Then** a validation error is returned explaining what values are acceptable

---

### User Story 6 - Search and View Opportunities (Priority: P2)

A user wants to review their sales pipeline. They search for opportunities by stage, company, or name, and view deal details including amount, stage, close date, and the associated company.

**Why this priority**: Opportunities represent the sales pipeline and revenue tracking. Important for pipeline reviews but depends on companies being accessible first.

**Independent Test**: Can be fully tested by listing opportunities, filtering by stage, and viewing a single opportunity's complete details.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user searches for opportunities, **Then** matching deals are returned with key fields (name, stage, amount, close date, company)
2. **Given** a configured MCP server, **When** the user requests full details of a specific opportunity, **Then** all opportunity attributes are returned including the associated company

---

### User Story 7 - Search and View Tasks (Priority: P2)

A user wants to review their CRM tasks — follow-ups, action items, and reminders. They list tasks, filter by status or due date, and view task details.

**Why this priority**: Tasks drive daily CRM activity. Accessing them from the AI assistant keeps users focused and productive without switching to the CRM interface.

**Independent Test**: Can be fully tested by listing tasks, filtering by status, and viewing a task's details.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user lists tasks, **Then** tasks are returned with key fields (title, status, due date, assignee, linked entity)
2. **Given** a configured MCP server, **When** the user filters tasks by status or due date, **Then** only matching tasks are returned

---

### Edge Cases

- What happens when the TwentyCRM API token becomes invalid mid-session? The server returns a clear API authentication error to the administrator.
- What happens when a user authenticates via OAuth OIDC but their email is not on the whitelist? Access is denied with a clear "not authorized" message.
- What happens when the OAuth OIDC provider is unreachable? The server returns an error explaining that user identity could not be verified.
- What happens when a requested record does not exist? The server returns a clear "not found" message with the record identifier, rather than a generic error.
- What happens when the TwentyCRM API rate limit is hit? The server returns an informative message indicating the rate limit has been reached.
- What happens when a create or update includes invalid field values? The server returns validation errors indicating which fields failed and what values are acceptable.
- What happens when required fields are missing during record creation? The server returns a clear error listing all required fields that were not provided.
- What happens when pagination reaches the end of results? The server clearly indicates no more results are available.
- What happens when the TwentyCRM instance is unreachable? The server returns a connection error with guidance on checking network connectivity.
- What happens when a search returns zero results? The server returns an empty result set with a clear message, not an error.
- What happens when the user attempts to update a read-only field? The server returns an error explaining which fields cannot be modified.

## Requirements

### Functional Requirements

- **FR-001**: System MUST verify user identity via OAuth OIDC provided by the MCP client (e.g., claude.ai) and check the user's email against a configurable whitelist of allowed email addresses
- **FR-002**: System MUST authenticate with the TwentyCRM API using a single pre-configured API token shared across all whitelisted users, providing access to one TwentyCRM workspace
- **FR-003**: System MUST deny access to users whose email is not on the whitelist, returning a clear authorization error
- **FR-004**: System MUST expose CRM operations as MCP tools following the Model Context Protocol specification
- **FR-005**: System MUST support searching companies by name, domain, and other key attributes
- **FR-006**: System MUST support viewing full details of a specific company record
- **FR-007**: System MUST support creating new company records with required fields
- **FR-008**: System MUST support updating company record fields
- **FR-009**: System MUST support searching contacts/people by name, email, phone, and company association
- **FR-010**: System MUST support viewing full details of a specific contact record, including their company association
- **FR-011**: System MUST support creating new contact records, optionally linking them to a company
- **FR-012**: System MUST support updating contact record fields
- **FR-013**: System MUST support searching opportunities by name, stage, company, and other key attributes
- **FR-014**: System MUST support viewing full details of a specific opportunity record
- **FR-015**: System MUST support creating new opportunity records linked to a company
- **FR-016**: System MUST support updating opportunity record fields (e.g., stage, amount, close date)
- **FR-017**: System MUST support searching tasks by status, due date, assignee, and linked entity
- **FR-018**: System MUST support viewing full details of a specific task record
- **FR-019**: System MUST support creating new task records
- **FR-020**: System MUST support updating task record fields (e.g., status, due date, title)
- **FR-021**: System MUST support an optional fields parameter on search and view tools, allowing users to specify which fields to return; when omitted, a sensible default set of key fields is returned
- **FR-022**: System MUST support cursor-based pagination for all search/list operations
- **FR-023**: System MUST support ordering/sorting results by standard fields (e.g., createdAt, name, updatedAt)
- **FR-024**: System MUST return clear, human-readable error messages for authentication failures, authorization denials, validation errors, not-found conditions, and connectivity issues
- **FR-025**: System MUST expose one dedicated tool per operation per entity (16 tools total: search, view, create, update for each of companies, contacts, opportunities, tasks), each with its own description and parameter schema
- **FR-026**: System MUST communicate over streamable HTTP transport as the primary MCP transport mechanism, suitable for remote hosted deployment

### Key Entities

- **Company**: A business organization tracked in the CRM. Key attributes: name, domain, employee count, address, industry. Central entity linked to contacts and opportunities.
- **Contact (Person)**: An individual person in the CRM. Key attributes: first name, last name, email, phone, job title, city. Linked to a company.
- **Opportunity**: A sales deal or potential revenue. Key attributes: name, stage, amount, close date, probability. Linked to a company.
- **Task**: An action item or follow-up. Key attributes: title, body, due date, status, assignee. Linked to target entities (companies, contacts).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can search for and find any contact, company, opportunity, or task within 3 interactions (search, filter, view detail) without leaving their AI assistant
- **SC-002**: Users can create a new record and see the confirmed creation within a single interaction
- **SC-003**: Users can update any editable field on a record and see the confirmed change within a single interaction
- **SC-004**: Users can discover and understand all available tools within 2 minutes of connecting the MCP server, without consulting external documentation
- **SC-005**: All search, view, create, and update operations return results within 5 seconds under normal network conditions
- **SC-006**: Error messages are specific enough for users to self-resolve 90% of issues (e.g., missing required fields, invalid field values, record not found, authentication expired)
- **SC-007**: Users can paginate through 1000+ records without data loss or degraded experience
- **SC-008**: User authentication (OAuth OIDC verification + whitelist check) completes in under 30 seconds from initiation to first available tool

## Assumptions

- Users have an active TwentyCRM workspace with API access enabled
- A single TwentyCRM API token has been pre-configured for the MCP server by an administrator; all whitelisted users share access to the same workspace
- A whitelist of allowed user email addresses is configured on the MCP server
- The MCP client (e.g., claude.ai) supports OAuth OIDC and provides verified user email to the MCP server
- The MCP client supports the MCP protocol and streamable HTTP transport for remote servers
- The TwentyCRM REST API is available and follows the documented OpenAPI specification
- The initial release focuses on four core entities: companies, contacts, opportunities, and tasks. Additional entities (notes, workflows, calendar events, messages, etc.) are out of scope for v1
- The server supports search, view, create, and update operations. Delete operations are out of scope for v1
- The server is deployed as a remote hosted service, accessible by MCP clients over streamable HTTP
- Rate limits imposed by TwentyCRM are handled with informative user-facing messages, not automatic retries
- Users have appropriate permissions in TwentyCRM to read, create, and update the records they interact with
