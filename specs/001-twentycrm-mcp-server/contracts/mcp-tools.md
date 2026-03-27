# MCP Tool Contracts: TwentyCRM MCP Server

**Feature Branch**: `001-twentycrm-mcp-server`
**Date**: 2026-03-27
**Transport**: Streamable HTTP (stateless)

## Overview

16 MCP tools exposed, organized as 4 operations x 4 entities.

**Naming convention**: `{operation}_{entity}` (e.g., `search_companies`, `view_company`)

**Common parameters** across tools:
- `fields` (optional string[]): Specific fields to return. When omitted, returns a default set of key fields.

---

## Companies

### search_companies

Search and filter company records.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | string | no | Search term (matched against name via ilike) |
| filter | object | no | Advanced filters: `{ field, comparator, value }[]` |
| order_by | string | no | Sort field and direction (e.g., `createdAt[DescNullsLast]`) |
| limit | integer | no | Results per page (default: 20, max: 200) |
| starting_after | string | no | Cursor for next page |
| fields | string[] | no | Fields to include in response |

**Returns**: `{ data: Company[], pageInfo: { hasNextPage, startCursor, endCursor }, totalCount? }`

**Default fields**: id, name, domainName.primaryLinkUrl, employees, industry, lifecycleStage, createdAt

---

### view_company

Get full details of a specific company.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Company record ID |
| fields | string[] | no | Fields to include in response |

**Returns**: `{ data: Company }` with all fields by default

---

### create_company

Create a new company record.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| name | string | no | Company name |
| domainName | string | no | Primary website URL |
| employees | integer | no | Number of employees |
| industry | string | no | Industry classification |
| address | object | no | Address fields |
| phones | object | no | Phone numbers |
| description | string | no | Description |
| *(additional fields)* | various | no | Any writable company field |

**Returns**: `{ data: Company }` — the created record

---

### update_company

Update fields on an existing company.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Company record ID |
| *(fields to update)* | various | no | Any writable company field with new value |

**Returns**: `{ data: Company }` — the updated record

---

## Contacts (People)

### search_contacts

Search and filter contact/people records.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | string | no | Search term (matched against firstName, lastName, primaryEmail via ilike) |
| filter | object | no | Advanced filters |
| order_by | string | no | Sort field and direction |
| limit | integer | no | Results per page (default: 20, max: 200) |
| starting_after | string | no | Cursor for next page |
| fields | string[] | no | Fields to include in response |

**Returns**: `{ data: Person[], pageInfo: { hasNextPage, startCursor, endCursor } }`

**Default fields**: id, name.firstName, name.lastName, emails.primaryEmail, phones.primaryPhoneNumber, jobTitle, companyId, city, createdAt

---

### view_contact

Get full details of a specific contact.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Person record ID |
| fields | string[] | no | Fields to include |

**Returns**: `{ data: Person }` with all fields by default

---

### create_contact

Create a new contact/person record.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| firstName | string | no | First name |
| lastName | string | no | Last name |
| email | string | no | Primary email |
| phone | string | no | Primary phone |
| jobTitle | string | no | Job title |
| companyId | string (uuid) | no | Link to company |
| *(additional fields)* | various | no | Any writable person field |

**Returns**: `{ data: Person }` — the created record

---

### update_contact

Update fields on an existing contact.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Person record ID |
| *(fields to update)* | various | no | Any writable person field |

**Returns**: `{ data: Person }` — the updated record

---

## Opportunities

### search_opportunities

Search and filter opportunity records.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | string | no | Search term (matched against name via ilike) |
| filter | object | no | Advanced filters (e.g., by stage, companyId) |
| order_by | string | no | Sort field and direction |
| limit | integer | no | Results per page (default: 20, max: 200) |
| starting_after | string | no | Cursor for next page |
| fields | string[] | no | Fields to include |

**Returns**: `{ data: Opportunity[], pageInfo: { hasNextPage, startCursor, endCursor } }`

**Default fields**: id, name, stage, amount, closeDate, companyId, createdAt

---

### view_opportunity

Get full details of a specific opportunity.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Opportunity record ID |
| fields | string[] | no | Fields to include |

**Returns**: `{ data: Opportunity }` with all fields by default

---

### create_opportunity

Create a new opportunity record.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| name | string | no | Opportunity name |
| stage | string | no | Pipeline stage |
| closeDate | string (date-time) | no | Expected close date |
| amount | object | no | `{ amountMicros, currencyCode }` |
| companyId | string (uuid) | no | Link to company |
| pointOfContactId | string (uuid) | no | Link to contact person |
| *(additional fields)* | various | no | Any writable opportunity field |

**Returns**: `{ data: Opportunity }` — the created record

---

### update_opportunity

Update fields on an existing opportunity.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Opportunity record ID |
| *(fields to update)* | various | no | Any writable opportunity field |

**Returns**: `{ data: Opportunity }` — the updated record

---

## Tasks

### search_tasks

Search and filter task records.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | string | no | Search term (matched against title via ilike) |
| filter | object | no | Advanced filters (e.g., by status, assigneeId, dueAt) |
| order_by | string | no | Sort field and direction |
| limit | integer | no | Results per page (default: 20, max: 200) |
| starting_after | string | no | Cursor for next page |
| fields | string[] | no | Fields to include |

**Returns**: `{ data: Task[], pageInfo: { hasNextPage, startCursor, endCursor } }`

**Default fields**: id, title, status, dueAt, assigneeId, createdAt

---

### view_task

Get full details of a specific task.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Task record ID |
| fields | string[] | no | Fields to include |

**Returns**: `{ data: Task }` with all fields by default, including taskTargets

---

### create_task

Create a new task record, optionally linked to entities.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| title | string | no | Task title |
| status | string | no | TODO, IN_PROGRESS, or DONE (default: TODO) |
| dueAt | string (date-time) | no | Due date |
| assigneeId | string (uuid) | no | Assigned workspace member |
| body | string | no | Task description (markdown) |
| targetCompanyId | string (uuid) | no | Link task to a company |
| targetPersonId | string (uuid) | no | Link task to a person |
| targetOpportunityId | string (uuid) | no | Link task to an opportunity |

**Returns**: `{ data: Task }` — the created record

**Note**: If target IDs are provided, the server creates TaskTarget records transparently after creating the task.

---

### update_task

Update fields on an existing task.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | string (uuid) | yes | Task record ID |
| *(fields to update)* | various | no | Any writable task field |

**Returns**: `{ data: Task }` — the updated record

---

## Error Responses

All tools return errors in a consistent format:

```json
{
  "isError": true,
  "content": [
    {
      "type": "text",
      "text": "Error: [human-readable message]\n\nDetails: [specific field errors or guidance]"
    }
  ]
}
```

**Error categories**:
- **Authentication error**: Invalid/expired TwentyCRM API token
- **Authorization error**: User email not on whitelist
- **Not found**: Record ID does not exist
- **Validation error**: Invalid field values or missing required fields
- **Rate limit**: TwentyCRM API rate limit exceeded
- **Connectivity error**: TwentyCRM API unreachable
