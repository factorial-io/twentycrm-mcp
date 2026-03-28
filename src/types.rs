use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── API Response Wrappers ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "startCursor")]
    pub start_cursor: Option<String>,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListResponse {
    pub data: serde_json::Value,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

// ─── Common Tool Parameters ─────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Free-text search query (matched via ilike on key fields).
    #[serde(default)]
    pub query: Option<String>,

    /// Filter expression in TwentyCRM format, e.g. "name[ilike]:%Acme%".
    #[serde(default)]
    pub filter: Option<String>,

    /// Ordering expression, e.g. "createdAt[DescNullsLast]".
    #[serde(default)]
    pub order_by: Option<String>,

    /// Maximum number of results to return (default: 20, max: 200).
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,

    /// Cursor for the next page of results.
    #[serde(default)]
    pub starting_after: Option<String>,

    /// Specific fields to include in the response. If omitted, returns a default set.
    #[serde(default)]
    pub fields: Option<Vec<String>>,
}

fn default_limit() -> Option<i64> {
    Some(20)
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ViewParams {
    /// The unique ID of the record to retrieve.
    pub id: String,

    /// Specific fields to include in the response. If omitted, returns all fields.
    #[serde(default)]
    pub fields: Option<Vec<String>>,
}

// ─── Company Tool Parameters ────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateCompanyParams {
    /// Company name.
    #[serde(default)]
    pub name: Option<String>,
    /// Primary website URL.
    #[serde(default, rename = "domainName")]
    pub domain_name: Option<serde_json::Value>,
    /// Number of employees.
    #[serde(default)]
    pub employees: Option<i64>,
    /// Industry classification.
    #[serde(default)]
    pub industry: Option<String>,
    /// Company address.
    #[serde(default)]
    pub address: Option<serde_json::Value>,
    /// Phone numbers.
    #[serde(default)]
    pub phones: Option<serde_json::Value>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCompanyParams {
    /// The unique ID of the company to update.
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "domainName")]
    pub domain_name: Option<serde_json::Value>,
    #[serde(default)]
    pub employees: Option<i64>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub address: Option<serde_json::Value>,
    #[serde(default)]
    pub phones: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
}

// ─── Contact (Person) Tool Parameters ───────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateContactParams {
    /// First name.
    #[serde(default, rename = "firstName")]
    pub first_name: Option<String>,
    /// Last name.
    #[serde(default, rename = "lastName")]
    pub last_name: Option<String>,
    /// Primary email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Primary phone number.
    #[serde(default)]
    pub phone: Option<String>,
    /// Job title.
    #[serde(default, rename = "jobTitle")]
    pub job_title: Option<String>,
    /// Company ID to link to.
    #[serde(default, rename = "companyId")]
    pub company_id: Option<String>,
    /// City.
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateContactParams {
    /// The unique ID of the contact to update.
    pub id: String,
    #[serde(default, rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(default, rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default, rename = "jobTitle")]
    pub job_title: Option<String>,
    #[serde(default, rename = "companyId")]
    pub company_id: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
}

// ─── Opportunity Tool Parameters ────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateOpportunityParams {
    /// Opportunity name.
    #[serde(default)]
    pub name: Option<String>,
    /// Pipeline stage.
    #[serde(default)]
    pub stage: Option<String>,
    /// Expected close date (ISO 8601).
    #[serde(default, rename = "closeDate")]
    pub close_date: Option<String>,
    /// Deal amount as object { amountMicros, currencyCode }.
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    /// Associated company ID.
    #[serde(default, rename = "companyId")]
    pub company_id: Option<String>,
    /// Point of contact person ID.
    #[serde(default, rename = "pointOfContactId")]
    pub point_of_contact_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateOpportunityParams {
    /// The unique ID of the opportunity to update.
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default, rename = "closeDate")]
    pub close_date: Option<String>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    #[serde(default, rename = "companyId")]
    pub company_id: Option<String>,
    #[serde(default, rename = "pointOfContactId")]
    pub point_of_contact_id: Option<String>,
}

// ─── Task Tool Parameters ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskParams {
    /// Task title.
    #[serde(default)]
    pub title: Option<String>,
    /// Status: TODO, IN_PROGRESS, or DONE.
    #[serde(default)]
    pub status: Option<String>,
    /// Due date (ISO 8601).
    #[serde(default, rename = "dueAt")]
    pub due_at: Option<String>,
    /// Assigned workspace member ID.
    #[serde(default, rename = "assigneeId")]
    pub assignee_id: Option<String>,
    /// Task description (markdown).
    #[serde(default)]
    pub body: Option<String>,
    /// Link task to a company.
    #[serde(default, rename = "targetCompanyId")]
    pub target_company_id: Option<String>,
    /// Link task to a person.
    #[serde(default, rename = "targetPersonId")]
    pub target_person_id: Option<String>,
    /// Link task to an opportunity.
    #[serde(default, rename = "targetOpportunityId")]
    pub target_opportunity_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTaskParams {
    /// The unique ID of the task to update.
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "dueAt")]
    pub due_at: Option<String>,
    #[serde(default, rename = "assigneeId")]
    pub assignee_id: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_defaults() {
        let json = r#"{}"#;
        let params: SearchParams = serde_json::from_str(json).unwrap();
        assert!(params.query.is_none());
        assert_eq!(params.limit, Some(20));
    }

    #[test]
    fn test_search_params_with_values() {
        let json = r#"{"query": "Acme", "limit": 50}"#;
        let params: SearchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.query.as_deref(), Some("Acme"));
        assert_eq!(params.limit, Some(50));
    }

    #[test]
    fn test_create_company_params_serialization() {
        let params = CreateCompanyParams {
            name: Some("Test Corp".into()),
            domain_name: None,
            employees: Some(100),
            industry: None,
            address: None,
            phones: None,
            description: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["name"], "Test Corp");
        assert_eq!(json["employees"], 100);
    }

    #[test]
    fn test_view_params_requires_id() {
        let json = r#"{"id": "abc-123"}"#;
        let params: ViewParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.id, "abc-123");
    }

    #[test]
    fn test_list_response_deserialization() {
        let json = r#"{"data": [{"id": "1"}], "pageInfo": {"hasNextPage": true, "startCursor": "a", "endCursor": "b"}}"#;
        let resp: ListResponse = serde_json::from_str(json).unwrap();
        assert!(resp.page_info.unwrap().has_next_page);
    }
}
