// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteParams {
    /// The unique ID of the record to delete.
    pub id: String,
}

// ─── Shared Field Types ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DomainName {
    /// Primary website URL (e.g. "https://example.com").
    #[serde(default, rename = "primaryLinkUrl")]
    pub primary_link_url: Option<String>,
    /// Display label for the link (e.g. "Website").
    #[serde(default, rename = "primaryLinkLabel")]
    pub primary_link_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Address {
    /// Street address line 1.
    #[serde(default, rename = "addressStreet1")]
    pub street1: Option<String>,
    /// Street address line 2.
    #[serde(default, rename = "addressStreet2")]
    pub street2: Option<String>,
    /// City.
    #[serde(default, rename = "addressCity")]
    pub city: Option<String>,
    /// State or region.
    #[serde(default, rename = "addressState")]
    pub state: Option<String>,
    /// Postal code.
    #[serde(default, rename = "addressPostcode")]
    pub postcode: Option<String>,
    /// Country.
    #[serde(default, rename = "addressCountry")]
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Phones {
    /// Primary phone number.
    #[serde(default, rename = "primaryPhoneNumber")]
    pub primary_phone_number: Option<String>,
    /// Primary phone country code (e.g. "+49").
    #[serde(default, rename = "primaryPhoneCountryCode")]
    pub primary_phone_country_code: Option<String>,
}

// ─── Company Tool Parameters ────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateCompanyParams {
    /// Company name.
    #[serde(default)]
    pub name: Option<String>,
    /// Company website domain.
    #[serde(default, rename = "domainName")]
    pub domain_name: Option<DomainName>,
    /// Number of employees.
    #[serde(default)]
    pub employees: Option<i64>,
    /// Industry classification.
    #[serde(default)]
    pub industry: Option<String>,
    /// Company address.
    #[serde(default)]
    pub address: Option<Address>,
    /// Phone numbers.
    #[serde(default)]
    pub phones: Option<Phones>,
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
    /// Company website domain.
    #[serde(default, rename = "domainName")]
    pub domain_name: Option<DomainName>,
    #[serde(default)]
    pub employees: Option<i64>,
    #[serde(default)]
    pub industry: Option<String>,
    /// Company address.
    #[serde(default)]
    pub address: Option<Address>,
    /// Phone numbers.
    #[serde(default)]
    pub phones: Option<Phones>,
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
    /// Additional email addresses.
    #[serde(default, rename = "additionalEmails")]
    pub additional_emails: Option<Vec<String>>,
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
    /// Additional email addresses (replaces existing list).
    #[serde(default, rename = "additionalEmails")]
    pub additional_emails: Option<Vec<String>>,
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
pub struct Amount {
    /// Amount in micros (e.g. 50000000000 = 50,000.00).
    #[serde(default, rename = "amountMicros")]
    pub amount_micros: Option<i64>,
    /// ISO 4217 currency code (e.g. "EUR", "USD").
    #[serde(default, rename = "currencyCode")]
    pub currency_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateOpportunityParams {
    /// Opportunity name.
    #[serde(default)]
    pub name: Option<String>,
    /// Pipeline stage. Valid values: PROSPECT, DISCOVERY_CALL, OPT1_CHEMISTRY_MEETING, OPT2ND_NEGOTIATING, REBRIEFING_OFFERING, CLOSED_WON, NOT_RELEVANT_RIGHT_NOW, CLOSED_LOST.
    #[serde(default)]
    pub stage: Option<String>,
    /// Expected close date (ISO 8601).
    #[serde(default, rename = "closeDate")]
    pub close_date: Option<String>,
    /// Deal amount.
    #[serde(default)]
    pub amount: Option<Amount>,
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
    /// Pipeline stage. Valid values: PROSPECT, DISCOVERY_CALL, OPT1_CHEMISTRY_MEETING, OPT2ND_NEGOTIATING, REBRIEFING_OFFERING, CLOSED_WON, NOT_RELEVANT_RIGHT_NOW, CLOSED_LOST.
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default, rename = "closeDate")]
    pub close_date: Option<String>,
    /// Deal amount.
    #[serde(default)]
    pub amount: Option<Amount>,
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
    /// Link task to a campaign.
    #[serde(default, rename = "targetCampaignId")]
    pub target_campaign_id: Option<String>,
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

// ─── Note Tool Parameters ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateNoteParams {
    /// Note title.
    #[serde(default)]
    pub title: Option<String>,
    /// Note body (markdown).
    #[serde(default)]
    pub body: Option<String>,
    /// Link note to a company.
    #[serde(default, rename = "targetCompanyId")]
    pub target_company_id: Option<String>,
    /// Link note to a person/contact.
    #[serde(default, rename = "targetPersonId")]
    pub target_person_id: Option<String>,
    /// Link note to an opportunity.
    #[serde(default, rename = "targetOpportunityId")]
    pub target_opportunity_id: Option<String>,
    /// Link note to a campaign.
    #[serde(default, rename = "targetCampaignId")]
    pub target_campaign_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateNoteParams {
    /// The unique ID of the note to update.
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    /// Note body (markdown).
    #[serde(default)]
    pub body: Option<String>,
}

// ─── Campaign Tool Parameters ──────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateCampaignParams {
    /// Campaign name.
    pub name: String,
    /// Position (display order).
    #[serde(default)]
    pub position: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCampaignParams {
    /// The unique ID of the campaign to update.
    pub id: String,
    /// Campaign name.
    #[serde(default)]
    pub name: Option<String>,
    /// Position (display order).
    #[serde(default)]
    pub position: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CampaignCompanyParams {
    /// The unique ID of the campaign.
    pub campaign_id: String,
    /// The unique ID of the company to add or remove.
    pub company_id: String,
}

// ─── Timeline Tool Parameters ──────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimelineParams {
    /// The entity type: "company", "contact", "opportunity", or "task".
    pub entity_type: String,
    /// The unique ID of the entity whose timeline to retrieve.
    pub entity_id: String,
    /// Maximum number of results to return (default: 20).
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,
    /// Cursor for the next page of results.
    #[serde(default)]
    pub starting_after: Option<String>,
}

// ─── Message Tool Parameters ───────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchMessagesParams {
    /// Filter by participant person ID (contact linked to message).
    #[serde(default, rename = "personId")]
    pub person_id: Option<String>,
    /// Free-text search on subject.
    #[serde(default)]
    pub subject: Option<String>,
    /// Additional filter expression in TwentyCRM format.
    #[serde(default)]
    pub filter: Option<String>,
    /// Maximum number of results (default: 20).
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,
    /// Cursor for the next page.
    #[serde(default)]
    pub starting_after: Option<String>,
}

// ─── Calendar Tool Parameters ──────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchCalendarEventsParams {
    /// Filter by start date (ISO 8601, e.g. "2026-03-01T00:00:00Z"). Returns events starting on or after this date.
    #[serde(default, rename = "startsAfter")]
    pub starts_after: Option<String>,
    /// Filter by end date (ISO 8601). Returns events starting before this date.
    #[serde(default, rename = "startsBefore")]
    pub starts_before: Option<String>,
    /// Free-text search on title.
    #[serde(default)]
    pub title: Option<String>,
    /// Additional filter expression in TwentyCRM format.
    #[serde(default)]
    pub filter: Option<String>,
    /// Maximum number of results (default: 20).
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,
    /// Cursor for the next page.
    #[serde(default)]
    pub starting_after: Option<String>,
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
