# Feature Specification: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Created**: 2026-03-27
**Status**: Draft
**Input**: User description: "The TwentyCRM MCP server should expose the most important endpoints of the TwentyCRM API via MCP. The user should be able to search for contacts, companies, opportunities, tasks, view their respective details and update the information."

## User Scenarios & Testing

### User Story 1 - Server Setup and Tool Discovery (Priority: P1)

A user wants to connect their AI assistant to their TwentyCRM workspace. They configure the MCP server with their API token, and the server immediately exposes all available CRM tools. The user can see what operations are available — searching, viewing, and updating records — without needing to consult external documentation.

**Why this priority**: This is the foundation. Without successful authentication and tool discovery, no CRM operations are possible.

**Independent Test**: Can be fully tested by starting the server with a valid API token and verifying the MCP client receives the complete list of available tools with descriptions.

**Acceptance Scenarios**:

1. **Given** a user has a TwentyCRM API token, **When** they configure the MCP server with the token, **Then** the server starts successfully and exposes all available CRM tools to the MCP client
2. **Given** a running MCP server, **When** the MCP client requests the tool list, **Then** tools for searching, viewing, and updating contacts, companies, opportunities, and tasks are returned with clear descriptions and parameter definitions
3. **Given** an invalid or expired API token, **When** the server attempts to authenticate, **Then** a clear error message is returned explaining the authentication failure

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

### User Story 4 - Update Company and Contact Information (Priority: P1)

A user has just had a meeting and wants to update CRM records with new information — a company's new address, a contact's updated job title, or a corrected email address. They locate the record and update specific fields without affecting other data.

**Why this priority**: Keeping CRM data current is critical for sales teams. The ability to update records directly from the AI assistant eliminates context-switching and ensures data is captured in the moment.

**Independent Test**: Can be fully tested by updating a field on a company record (e.g., changing the domain) and verifying the change persists when viewing the record again.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user updates a company's field (e.g., address, domain, employee count), **Then** the field is updated and the full updated record is returned as confirmation
2. **Given** a configured MCP server, **When** the user updates a contact's field (e.g., job title, email, phone), **Then** the field is updated and the full updated record is returned
3. **Given** a configured MCP server, **When** the user attempts to update a record that does not exist, **Then** a clear "not found" error is returned
4. **Given** a configured MCP server, **When** the user provides an invalid value for a field, **Then** a validation error is returned explaining what values are acceptable

---

### User Story 5 - Search and View Opportunities (Priority: P2)

A user wants to review their sales pipeline. They search for opportunities by stage, company, or name, and view deal details including amount, stage, close date, and the associated company.

**Why this priority**: Opportunities represent the sales pipeline and revenue tracking. Important for pipeline reviews but depends on companies being accessible first.

**Independent Test**: Can be fully tested by listing opportunities, filtering by stage, and viewing a single opportunity's complete details.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user searches for opportunities, **Then** matching deals are returned with key fields (name, stage, amount, close date, company)
2. **Given** a configured MCP server, **When** the user requests full details of a specific opportunity, **Then** all opportunity attributes are returned including the associated company
3. **Given** a configured MCP server, **When** the user updates an opportunity's stage or amount, **Then** the change is persisted and the updated record is returned

---

### User Story 6 - Search and View Tasks (Priority: P2)

A user wants to review their CRM tasks — follow-ups, action items, and reminders. They list tasks, filter by status or due date, view task details, and mark tasks as completed or update their information.

**Why this priority**: Tasks drive daily CRM activity. Accessing them from the AI assistant keeps users focused and productive without switching to the CRM interface.

**Independent Test**: Can be fully tested by listing tasks, filtering by status, viewing a task's details, and updating its status.

**Acceptance Scenarios**:

1. **Given** a configured MCP server, **When** the user lists tasks, **Then** tasks are returned with key fields (title, status, due date, assignee, linked entity)
2. **Given** a configured MCP server, **When** the user filters tasks by status or due date, **Then** only matching tasks are returned
3. **Given** a configured MCP server, **When** the user updates a task's status or details, **Then** the change is persisted and the updated record is returned

---

### Edge Cases

- What happens when the API token expires mid-session? The server returns a clear authentication error prompting the user to reconfigure their token.
- What happens when a requested record does not exist? The server returns a clear "not found" message with the record identifier, rather than a generic error.
- What happens when the TwentyCRM API rate limit is hit? The server returns an informative message indicating the rate limit has been reached.
- What happens when an update includes invalid field values? The server returns validation errors indicating which fields failed and what values are acceptable.
- What happens when pagination reaches the end of results? The server clearly indicates no more results are available.
- What happens when the TwentyCRM instance is unreachable? The server returns a connection error with guidance on checking network connectivity.
- What happens when a search returns zero results? The server returns an empty result set with a clear message, not an error.
- What happens when the user attempts to update a read-only field? The server returns an error explaining which fields cannot be modified.

## Requirements

### Functional Requirements

- **FR-001**: System MUST authenticate with TwentyCRM using a bearer token provided via configuration
- **FR-002**: System MUST expose CRM operations as MCP tools following the Model Context Protocol specification
- **FR-003**: System MUST support searching companies by name, domain, and other key attributes
- **FR-004**: System MUST support viewing full details of a specific company record
- **FR-005**: System MUST support updating company record fields
- **FR-006**: System MUST support searching contacts/people by name, email, phone, and company association
- **FR-007**: System MUST support viewing full details of a specific contact record, including their company association
- **FR-008**: System MUST support updating contact record fields
- **FR-009**: System MUST support searching opportunities by name, stage, company, and other key attributes
- **FR-010**: System MUST support viewing full details of a specific opportunity record
- **FR-011**: System MUST support updating opportunity record fields (e.g., stage, amount, close date)
- **FR-012**: System MUST support searching tasks by status, due date, assignee, and linked entity
- **FR-013**: System MUST support viewing full details of a specific task record
- **FR-014**: System MUST support updating task record fields (e.g., status, due date, title)
- **FR-015**: System MUST support cursor-based pagination for all search/list operations
- **FR-016**: System MUST support ordering/sorting results by standard fields (e.g., createdAt, name, updatedAt)
- **FR-017**: System MUST return clear, human-readable error messages for authentication failures, validation errors, not-found conditions, and connectivity issues
- **FR-018**: System MUST expose tool descriptions and parameter schemas so MCP clients can present available operations to users
- **FR-019**: System MUST communicate over stdio transport as the primary MCP transport mechanism

### Key Entities

- **Company**: A business organization tracked in the CRM. Key attributes: name, domain, employee count, address, industry. Central entity linked to contacts and opportunities.
- **Contact (Person)**: An individual person in the CRM. Key attributes: first name, last name, email, phone, job title, city. Linked to a company.
- **Opportunity**: A sales deal or potential revenue. Key attributes: name, stage, amount, close date, probability. Linked to a company.
- **Task**: An action item or follow-up. Key attributes: title, body, due date, status, assignee. Linked to target entities (companies, contacts).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can search for and find any contact, company, opportunity, or task within 3 interactions (search, filter, view detail) without leaving their AI assistant
- **SC-002**: Users can update any editable field on a record and see the confirmed change within a single interaction
- **SC-003**: Users can discover and understand all available tools within 2 minutes of connecting the MCP server, without consulting external documentation
- **SC-004**: All search and view operations return results within 5 seconds under normal network conditions
- **SC-005**: Error messages are specific enough for users to self-resolve 90% of issues (e.g., invalid field values, record not found, authentication expired)
- **SC-006**: Users can paginate through 1000+ records without data loss or degraded experience
- **SC-007**: Setup and first successful search can be completed within 5 minutes using only the provided configuration instructions

## Assumptions

- Users have an active TwentyCRM workspace with API access enabled
- Users have generated a valid API token from TwentyCRM Settings
- The MCP client (e.g., Claude Code, Cursor) supports the MCP protocol and stdio transport
- The TwentyCRM REST API is available and follows the documented OpenAPI specification
- The initial release focuses on four core entities: companies, contacts, opportunities, and tasks. Additional entities (notes, workflows, calendar events, messages, etc.) are out of scope for v1
- The server focuses on read (search, view) and update operations. Create and delete operations are out of scope for v1
- The server runs locally on the user's machine, connecting to TwentyCRM's cloud API
- Rate limits imposed by TwentyCRM are handled with informative user-facing messages, not automatic retries
- Users have appropriate permissions in TwentyCRM to read and update the records they interact with
