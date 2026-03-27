use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ServerInfo;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use serde_json::json;

use crate::twentycrm::TwentyCrmClient;
use crate::types::*;

/// Per-request MCP server holding the authenticated user's context.
#[derive(Clone)]
pub struct TwentyCrmMcpServer {
    pub client: TwentyCrmClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TwentyCrmMcpServer {
    pub fn new(client: TwentyCrmClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    // ─── Companies ──────────────────────────────────────────────────

    #[tool(description = "Search for companies in TwentyCRM by name, domain, industry, or other attributes. Supports filtering, pagination, and field selection.")]
    async fn search_companies(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self.client.search(
            "companies", p.query.as_deref(), p.filter.as_deref(),
            p.order_by.as_deref(), p.limit, p.starting_after.as_deref(), p.fields.as_deref(),
        ).await {
            Ok(resp) => format_list_result("companies", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific company by ID.")]
    async fn view_company(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self.client.get("companies", &p.id, p.fields.as_deref()).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Create a new company record in TwentyCRM.")]
    async fn create_company(&self, params: Parameters<CreateCompanyParams>) -> String {
        let body = serde_json::to_value(&params.0).unwrap_or_default();
        match self.client.create("companies", strip_nulls(body)).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Update an existing company record in TwentyCRM.")]
    async fn update_company(&self, params: Parameters<UpdateCompanyParams>) -> String {
        let id = params.0.id.clone();
        let mut body = serde_json::to_value(&params.0).unwrap_or_default();
        if let Some(obj) = body.as_object_mut() { obj.remove("id"); }
        match self.client.update("companies", &id, strip_nulls(body)).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    // ─── Contacts (People) ──────────────────────────────────────────

    #[tool(description = "Search for contacts/people in TwentyCRM by name, email, phone, or company. Supports filtering, pagination, and field selection.")]
    async fn search_contacts(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self.client.search(
            "people", p.query.as_deref(), p.filter.as_deref(),
            p.order_by.as_deref(), p.limit, p.starting_after.as_deref(), p.fields.as_deref(),
        ).await {
            Ok(resp) => format_list_result("contacts", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific contact/person by ID.")]
    async fn view_contact(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self.client.get("people", &p.id, p.fields.as_deref()).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Create a new contact/person record in TwentyCRM.")]
    async fn create_contact(&self, params: Parameters<CreateContactParams>) -> String {
        let p = params.0;
        let mut body = json!({});
        if let Some(first) = &p.first_name {
            body["name"] = json!({"firstName": first, "lastName": p.last_name.as_deref().unwrap_or("")});
        } else if let Some(last) = &p.last_name {
            body["name"] = json!({"firstName": "", "lastName": last});
        }
        if let Some(email) = &p.email { body["emails"] = json!({"primaryEmail": email}); }
        if let Some(phone) = &p.phone { body["phones"] = json!({"primaryPhoneNumber": phone}); }
        if let Some(jt) = &p.job_title { body["jobTitle"] = json!(jt); }
        if let Some(cid) = &p.company_id { body["companyId"] = json!(cid); }
        if let Some(city) = &p.city { body["city"] = json!(city); }
        match self.client.create("people", body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Update an existing contact/person record in TwentyCRM.")]
    async fn update_contact(&self, params: Parameters<UpdateContactParams>) -> String {
        let p = params.0;
        let id = p.id.clone();
        let mut body = json!({});
        if p.first_name.is_some() || p.last_name.is_some() {
            body["name"] = json!({
                "firstName": p.first_name.as_deref().unwrap_or(""),
                "lastName": p.last_name.as_deref().unwrap_or("")
            });
        }
        if let Some(email) = &p.email { body["emails"] = json!({"primaryEmail": email}); }
        if let Some(phone) = &p.phone { body["phones"] = json!({"primaryPhoneNumber": phone}); }
        if let Some(jt) = &p.job_title { body["jobTitle"] = json!(jt); }
        if let Some(cid) = &p.company_id { body["companyId"] = json!(cid); }
        if let Some(city) = &p.city { body["city"] = json!(city); }
        match self.client.update("people", &id, body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    // ─── Opportunities ──────────────────────────────────────────────

    #[tool(description = "Search for opportunities/deals in TwentyCRM by name, stage, company, or other attributes.")]
    async fn search_opportunities(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self.client.search(
            "opportunities", p.query.as_deref(), p.filter.as_deref(),
            p.order_by.as_deref(), p.limit, p.starting_after.as_deref(), p.fields.as_deref(),
        ).await {
            Ok(resp) => format_list_result("opportunities", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific opportunity/deal by ID.")]
    async fn view_opportunity(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self.client.get("opportunities", &p.id, p.fields.as_deref()).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Create a new opportunity/deal record in TwentyCRM.")]
    async fn create_opportunity(&self, params: Parameters<CreateOpportunityParams>) -> String {
        let body = serde_json::to_value(&params.0).unwrap_or_default();
        match self.client.create("opportunities", strip_nulls(body)).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Update an existing opportunity/deal record in TwentyCRM.")]
    async fn update_opportunity(&self, params: Parameters<UpdateOpportunityParams>) -> String {
        let id = params.0.id.clone();
        let mut body = serde_json::to_value(&params.0).unwrap_or_default();
        if let Some(obj) = body.as_object_mut() { obj.remove("id"); }
        match self.client.update("opportunities", &id, strip_nulls(body)).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    // ─── Tasks ──────────────────────────────────────────────────────

    #[tool(description = "Search for tasks in TwentyCRM by title, status, due date, assignee, or linked entity.")]
    async fn search_tasks(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self.client.search(
            "tasks", p.query.as_deref(), p.filter.as_deref(),
            p.order_by.as_deref(), p.limit, p.starting_after.as_deref(), p.fields.as_deref(),
        ).await {
            Ok(resp) => format_list_result("tasks", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific task by ID.")]
    async fn view_task(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self.client.get("tasks", &p.id, p.fields.as_deref()).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Create a new task in TwentyCRM. Optionally link to a company, person, or opportunity.")]
    async fn create_task(&self, params: Parameters<CreateTaskParams>) -> String {
        let p = params.0;
        let mut body = json!({});
        if let Some(title) = &p.title { body["title"] = json!(title); }
        if let Some(status) = &p.status { body["status"] = json!(status); }
        if let Some(due_at) = &p.due_at { body["dueAt"] = json!(due_at); }
        if let Some(assignee) = &p.assignee_id { body["assigneeId"] = json!(assignee); }
        if let Some(text) = &p.body { body["bodyV2"] = json!({"markdown": text}); }

        let record = match self.client.create("tasks", body).await {
            Ok(r) => r,
            Err(e) => return format_error(e),
        };

        // Create task targets if linked entities provided
        let task_id = record.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if !task_id.is_empty() {
            for (field, value) in [
                ("targetCompanyId", &p.target_company_id),
                ("targetPersonId", &p.target_person_id),
                ("targetOpportunityId", &p.target_opportunity_id),
            ] {
                if let Some(val) = value {
                    let target_body = json!({ "taskId": task_id, field: val });
                    if let Err(e) = self.client.create("taskTargets", target_body).await {
                        tracing::warn!("Failed to create task target: {e}");
                    }
                }
            }
        }

        format_record(record)
    }

    #[tool(description = "Update an existing task in TwentyCRM.")]
    async fn update_task(&self, params: Parameters<UpdateTaskParams>) -> String {
        let p = params.0;
        let id = p.id.clone();
        let mut body = json!({});
        if let Some(title) = &p.title { body["title"] = json!(title); }
        if let Some(status) = &p.status { body["status"] = json!(status); }
        if let Some(due_at) = &p.due_at { body["dueAt"] = json!(due_at); }
        if let Some(assignee) = &p.assignee_id { body["assigneeId"] = json!(assignee); }
        if let Some(text) = &p.body { body["bodyV2"] = json!({"markdown": text}); }
        match self.client.update("tasks", &id, body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }
}

#[tool_handler]
impl ServerHandler for TwentyCrmMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.instructions = Some(
            "TwentyCRM MCP Server. Search, view, create, and update companies, contacts, opportunities, and tasks in your TwentyCRM workspace.".into()
        );
        info
    }
}

// ─── Helpers ────────────────────────────────────────────────────────

fn format_list_result(entity: &str, resp: crate::types::ListResponse) -> String {
    let mut result = json!({ "entity": entity, "data": resp.data });
    if let Some(pi) = resp.page_info {
        result["pageInfo"] = json!({
            "hasNextPage": pi.has_next_page,
            "startCursor": pi.start_cursor,
            "endCursor": pi.end_cursor,
        });
    }
    serde_json::to_string_pretty(&result).unwrap_or_default()
}

fn format_record(record: serde_json::Value) -> String {
    serde_json::to_string_pretty(&record).unwrap_or_default()
}

fn format_error(err: crate::error::AppError) -> String {
    format!("Error: {}", err.client_message())
}

fn strip_nulls(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let filtered: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect();
            serde_json::Value::Object(filtered)
        }
        other => other,
    }
}
