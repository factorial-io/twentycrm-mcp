# Data Model: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Date**: 2026-03-27
**Source**: TwentyCRM OpenAPI Specification (core.json)

## Entities

### Company

The central CRM entity representing a business organization.

**Fields exposed via MCP tools:**

| Field | Type | Notes |
|-------|------|-------|
| id | uuid | Read-only, auto-generated |
| name | string | Company name |
| domainName | object | `{ primaryLinkUrl, primaryLinkLabel, secondaryLinks[] }` |
| employees | integer | Number of employees |
| industry | string | Industry classification |
| address | object | `{ addressStreet1, addressStreet2, addressCity, addressPostcode, addressState, addressCountry, addressLat, addressLng }` |
| phones | object | `{ primaryPhoneNumber, primaryPhoneCountryCode, primaryPhoneCallingCode, additionalPhones[] }` |
| annualRevenue | object | `{ amountMicros, currencyCode }` |
| annualRecurringRevenue | object | `{ amountMicros, currencyCode }` |
| linkedinLink | object | `{ primaryLinkUrl, primaryLinkLabel, secondaryLinks[] }` |
| xLink | object | X/Twitter link |
| facebook | object | Facebook link |
| description | string | Company description |
| yearFounded | integer | |
| timezone | string | |
| isPublic | boolean | |
| idealCustomerProfile | boolean | |
| lifecycleStage | enum | LEAD, KUNDE, OPPORTUNITY |
| accountOwnerId | uuid | Owner workspace member |
| position | number | UI sort order |
| createdAt | date-time | Read-only |
| updatedAt | date-time | Read-only |

**Relationships:**
- `people` → Person[] (contacts at this company)
- `opportunities` → Opportunity[] (deals linked to this company)
- `taskTargets` → TaskTarget[] (tasks linked to this company)

**Creation:** No required fields. All fields are optional.

**Searchable by:** name (ilike), domainName.primaryLinkUrl, industry, lifecycleStage, createdAt range

---

### Contact (Person)

An individual contact in the CRM. The TwentyCRM API entity name is `people`.

**Fields exposed via MCP tools:**

| Field | Type | Notes |
|-------|------|-------|
| id | uuid | Read-only, auto-generated |
| name | object | `{ firstName, lastName }` |
| emails | object | `{ primaryEmail, additionalEmails[] }` |
| phones | object | `{ primaryPhoneNumber, primaryPhoneCountryCode, primaryPhoneCallingCode, additionalPhones[] }` |
| mobilePhones | object | Same structure as phones |
| jobTitle | string | |
| city | string | |
| town | string | |
| country | string | |
| contactAddress | object | Full address object |
| companyId | uuid | Link to associated company |
| ownerId | uuid | Owner workspace member |
| linkedinLink | object | LinkedIn URL |
| xLink | object | X/Twitter URL |
| seniority | enum | C_SUITE, DIRECTOR, FOUNDER, MANAGER, NON_MANAGER |
| lifecycleStage | enum | LEAD, KUNDE, OPPORTUNITY, SALES_QUALIFIED_LEAD, etc. |
| leadSource | enum | INBOUND, OUTBOUND_OTHER, SALES_TEAM, etc. |
| leadStatus | enum | NEW, OPEN, IN_PROGRESS, CONNECTED, etc. |
| numberOfTimesContacted | integer | |
| lastActivityDate | date-time | |
| position | number | UI sort order |
| createdAt | date-time | Read-only |
| updatedAt | date-time | Read-only |

**Relationships:**
- `company` → Company (the company this person belongs to)
- `pointOfContactForOpportunities` → Opportunity[] (deals where this person is POC)
- `taskTargets` → TaskTarget[] (tasks linked to this person)

**Creation:** No required fields. All fields optional. Link to company via `companyId`.

**Searchable by:** name.firstName/lastName (ilike), emails.primaryEmail, jobTitle, companyId, city, lifecycleStage, leadStatus

---

### Opportunity

A sales deal or potential revenue.

**Fields exposed via MCP tools:**

| Field | Type | Notes |
|-------|------|-------|
| id | uuid | Read-only, auto-generated |
| name | string | Opportunity/deal name |
| stage | enum | PROSPECT, DISCOVERY_CALL, OPT1_CHEMISTRY_MEETING, OPT2ND_NEGOTIATING, REBRIEFING_OFFERING, CLOSED_WON, NOT_RELEVANT_RIGHT_NOW, CLOSED_LOST |
| closeDate | date-time | Expected close date |
| amount | object | `{ amountMicros, currencyCode }` |
| companyId | uuid | Associated company |
| pointOfContactId | uuid | Associated person |
| ownerId | uuid | Opportunity owner |
| position | number | UI sort order |
| createdAt | date-time | Read-only |
| updatedAt | date-time | Read-only |

**Relationships:**
- `company` → Company
- `pointOfContact` → Person
- `taskTargets` → TaskTarget[]

**Creation:** No required fields. All fields optional. Link to company via `companyId`, to contact via `pointOfContactId`.

**Searchable by:** name (ilike), stage, companyId, pointOfContactId, closeDate range, createdAt range

---

### Task

An action item or follow-up.

**Fields exposed via MCP tools:**

| Field | Type | Notes |
|-------|------|-------|
| id | uuid | Read-only, auto-generated |
| title | string | Task title |
| status | enum | TODO, IN_PROGRESS, DONE |
| dueAt | date-time | Task due date |
| assigneeId | uuid | Assigned workspace member |
| bodyV2 | object | `{ blocknote, markdown }` — task description |
| position | number | UI sort order |
| createdAt | date-time | Read-only |
| updatedAt | date-time | Read-only |

**Relationships:**
- `assignee` → WorkspaceMember
- `taskTargets` → TaskTarget[] (linked companies, people, opportunities)

**Creation:** No required fields. All fields optional.

**Searchable by:** title (ilike), status, assigneeId, dueAt range, createdAt range

---

### TaskTarget (Linking Entity)

Links a task to one or more entities. Not directly exposed as MCP tools but used when creating/viewing tasks with linked entities.

| Field | Type | Notes |
|-------|------|-------|
| id | uuid | Read-only |
| taskId | uuid | Required — the task being linked |
| targetCompanyId | uuid | Link to company |
| targetPersonId | uuid | Link to person |
| targetOpportunityId | uuid | Link to opportunity |

**Note:** Creating a task does NOT automatically create TaskTargets. They must be created separately via the `/taskTargets` endpoint. The MCP server should handle this transparently when creating tasks with linked entities.

## API Patterns

### Pagination
- Cursor-based: `limit` (default 60, max 200), `starting_after`, `ending_before`
- Response includes `pageInfo: { hasNextPage, startCursor, endCursor }`

### Filtering
- Format: `filter=field[comparator]:value`
- Multiple filters: `filter=field1[eq]:val1,field2[gte]:val2` (AND)
- Advanced: `or(...)`, `and(...)`, `not(...)`
- Nested fields: `emails.primaryEmail[eq]:foo@example.com`
- Comparators: eq, neq, in, is, gt, gte, lt, lte, startsWith, like, ilike, containsAny

### Ordering
- Format: `order_by=field1,field2[Direction]`
- Directions: AscNullsFirst (default), AscNullsLast, DescNullsFirst, DescNullsLast

### Authentication
- Bearer token: `Authorization: Bearer <token>`
- Token is workspace-scoped

### Key API Behaviors
- All entity fields are optional on creation
- Soft deletes via `deletedAt` timestamp
- `depth` parameter controls relationship expansion in responses
- `createdBy.source` / `updatedBy.source` track record provenance
