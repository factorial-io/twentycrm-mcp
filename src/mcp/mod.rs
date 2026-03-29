mod helpers;

use helpers::{extract_id, format_error, format_list_result, format_record, strip_nulls};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
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

    #[tool(
        description = "Search for companies in TwentyCRM by name, domain, industry, or other attributes. Supports filtering, pagination, and field selection."
    )]
    async fn search_companies(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self
            .client
            .search(
                "companies",
                p.query.as_deref(),
                p.filter.as_deref(),
                p.order_by.as_deref(),
                p.limit,
                p.starting_after.as_deref(),
                p.fields.as_deref(),
            )
            .await
        {
            Ok(resp) => format_list_result("companies", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific company by ID.")]
    async fn view_company(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self
            .client
            .get("companies", &p.id, p.fields.as_deref())
            .await
        {
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
        if let Some(obj) = body.as_object_mut() {
            obj.remove("id");
        }
        match self
            .client
            .update("companies", &id, strip_nulls(body))
            .await
        {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Delete a company record from TwentyCRM by ID.")]
    async fn delete_company(&self, params: Parameters<DeleteParams>) -> String {
        match self.client.delete("companies", &params.0.id).await {
            Ok(()) => format!("Company {} deleted.", params.0.id),
            Err(e) => format_error(e),
        }
    }

    // ─── Contacts (People) ──────────────────────────────────────────

    #[tool(
        description = "Search for contacts/people in TwentyCRM. The query parameter searches across first name, last name, and primary email. Use the filter parameter for precise field matching (e.g. emails.primaryEmail[eq]:user@example.com). Supports pagination and field selection."
    )]
    async fn search_contacts(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self
            .client
            .search(
                "people",
                p.query.as_deref(),
                p.filter.as_deref(),
                p.order_by.as_deref(),
                p.limit,
                p.starting_after.as_deref(),
                p.fields.as_deref(),
            )
            .await
        {
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
            body["name"] =
                json!({"firstName": first, "lastName": p.last_name.as_deref().unwrap_or("")});
        } else if let Some(last) = &p.last_name {
            body["name"] = json!({"firstName": "", "lastName": last});
        }
        if p.email.is_some() || p.additional_emails.is_some() {
            let mut emails = json!({});
            if let Some(email) = &p.email {
                emails["primaryEmail"] = json!(email);
            }
            if let Some(additional) = &p.additional_emails {
                emails["additionalEmails"] = json!(additional);
            }
            body["emails"] = emails;
        }
        if let Some(phone) = &p.phone {
            body["phones"] = json!({"primaryPhoneNumber": phone});
        }
        if let Some(jt) = &p.job_title {
            body["jobTitle"] = json!(jt);
        }
        if let Some(cid) = &p.company_id {
            body["companyId"] = json!(cid);
        }
        if let Some(city) = &p.city {
            body["city"] = json!(city);
        }
        match self.client.create("people", body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(
        description = "Update an existing contact/person record in TwentyCRM. You can update firstName or lastName independently — the other will be preserved. Use additionalEmails to add secondary email addresses (e.g. for email consolidation)."
    )]
    async fn update_contact(&self, params: Parameters<UpdateContactParams>) -> String {
        let p = params.0;
        let id = p.id.clone();
        let mut body = json!({});

        // Name is a composite object in Twenty — sending one field without the other
        // zeros it out. Read-then-merge so callers can update either independently.
        if p.first_name.is_some() || p.last_name.is_some() {
            let (existing_first, existing_last) = match self.client.get("people", &id, None).await {
                Ok(record) => {
                    let name = record
                        .pointer("/data/person/name")
                        .or_else(|| record.get("name"));
                    let first = name
                        .and_then(|n| n.get("firstName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let last = name
                        .and_then(|n| n.get("lastName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    (first, last)
                }
                Err(e) => return format_error(e),
            };

            body["name"] = json!({
                "firstName": p.first_name.as_deref().unwrap_or(&existing_first),
                "lastName": p.last_name.as_deref().unwrap_or(&existing_last),
            });
        }

        if p.email.is_some() || p.additional_emails.is_some() {
            let mut emails = json!({});
            if let Some(email) = &p.email {
                emails["primaryEmail"] = json!(email);
            }
            if let Some(additional) = &p.additional_emails {
                emails["additionalEmails"] = json!(additional);
            }
            body["emails"] = emails;
        }
        if let Some(phone) = &p.phone {
            body["phones"] = json!({"primaryPhoneNumber": phone});
        }
        if let Some(jt) = &p.job_title {
            body["jobTitle"] = json!(jt);
        }
        if let Some(cid) = &p.company_id {
            body["companyId"] = json!(cid);
        }
        if let Some(city) = &p.city {
            body["city"] = json!(city);
        }
        match self.client.update("people", &id, body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Delete a contact/person record from TwentyCRM by ID.")]
    async fn delete_contact(&self, params: Parameters<DeleteParams>) -> String {
        match self.client.delete("people", &params.0.id).await {
            Ok(()) => format!("Contact {} deleted.", params.0.id),
            Err(e) => format_error(e),
        }
    }

    // ─── Opportunities ──────────────────────────────────────────────

    #[tool(
        description = "Search for opportunities/deals in TwentyCRM by name, stage, company, or other attributes."
    )]
    async fn search_opportunities(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self
            .client
            .search(
                "opportunities",
                p.query.as_deref(),
                p.filter.as_deref(),
                p.order_by.as_deref(),
                p.limit,
                p.starting_after.as_deref(),
                p.fields.as_deref(),
            )
            .await
        {
            Ok(resp) => format_list_result("opportunities", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific opportunity/deal by ID.")]
    async fn view_opportunity(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self
            .client
            .get("opportunities", &p.id, p.fields.as_deref())
            .await
        {
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
        if let Some(obj) = body.as_object_mut() {
            obj.remove("id");
        }
        match self
            .client
            .update("opportunities", &id, strip_nulls(body))
            .await
        {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Delete an opportunity/deal record from TwentyCRM by ID.")]
    async fn delete_opportunity(&self, params: Parameters<DeleteParams>) -> String {
        match self.client.delete("opportunities", &params.0.id).await {
            Ok(()) => format!("Opportunity {} deleted.", params.0.id),
            Err(e) => format_error(e),
        }
    }

    // ─── Tasks ──────────────────────────────────────────────────────

    #[tool(
        description = "Search for tasks in TwentyCRM by title, status, due date, assignee, or linked entity."
    )]
    async fn search_tasks(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self
            .client
            .search(
                "tasks",
                p.query.as_deref(),
                p.filter.as_deref(),
                p.order_by.as_deref(),
                p.limit,
                p.starting_after.as_deref(),
                p.fields.as_deref(),
            )
            .await
        {
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

    #[tool(
        description = "Create a new task in TwentyCRM. Optionally link to a company, person, or opportunity."
    )]
    async fn create_task(&self, params: Parameters<CreateTaskParams>) -> String {
        let p = params.0;
        let mut body = json!({});
        if let Some(title) = &p.title {
            body["title"] = json!(title);
        }
        if let Some(status) = &p.status {
            body["status"] = json!(status);
        }
        if let Some(due_at) = &p.due_at {
            body["dueAt"] = json!(due_at);
        }
        if let Some(assignee) = &p.assignee_id {
            body["assigneeId"] = json!(assignee);
        }
        if let Some(text) = &p.body {
            body["bodyV2"] = json!({"markdown": text});
        }

        let record = match self.client.create("tasks", body).await {
            Ok(r) => r,
            Err(e) => return format_error(e),
        };

        // Create task targets if linked entities provided
        let task_id = extract_id(&record).unwrap_or("");
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
        if let Some(title) = &p.title {
            body["title"] = json!(title);
        }
        if let Some(status) = &p.status {
            body["status"] = json!(status);
        }
        if let Some(due_at) = &p.due_at {
            body["dueAt"] = json!(due_at);
        }
        if let Some(assignee) = &p.assignee_id {
            body["assigneeId"] = json!(assignee);
        }
        if let Some(text) = &p.body {
            body["bodyV2"] = json!({"markdown": text});
        }
        match self.client.update("tasks", &id, body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Delete a task from TwentyCRM by ID.")]
    async fn delete_task(&self, params: Parameters<DeleteParams>) -> String {
        match self.client.delete("tasks", &params.0.id).await {
            Ok(()) => format!("Task {} deleted.", params.0.id),
            Err(e) => format_error(e),
        }
    }

    // ─── Notes ──────────────────────────────────────────────────────

    #[tool(
        description = "Search for notes in TwentyCRM. Use query to search by title, or filter for precise matching. Supports pagination."
    )]
    async fn search_notes(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self
            .client
            .search(
                "notes",
                p.query.as_deref(),
                p.filter.as_deref(),
                p.order_by.as_deref(),
                p.limit,
                p.starting_after.as_deref(),
                p.fields.as_deref(),
            )
            .await
        {
            Ok(resp) => format_list_result("notes", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View full details of a specific note by ID.")]
    async fn view_note(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self.client.get("notes", &p.id, p.fields.as_deref()).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(
        description = "Create a new note in TwentyCRM. Optionally link to a company, person, or opportunity via target IDs."
    )]
    async fn create_note(&self, params: Parameters<CreateNoteParams>) -> String {
        let p = params.0;
        let mut body = json!({});
        if let Some(title) = &p.title {
            body["title"] = json!(title);
        }
        if let Some(text) = &p.body {
            body["bodyV2"] = json!({"markdown": text});
        }

        let record = match self.client.create("notes", body).await {
            Ok(r) => r,
            Err(e) => return format_error(e),
        };

        // Create noteTargets to link the note to entities
        let note_id = extract_id(&record).unwrap_or("");
        if !note_id.is_empty() {
            for (field, value) in [
                ("targetCompanyId", &p.target_company_id),
                ("targetPersonId", &p.target_person_id),
                ("targetOpportunityId", &p.target_opportunity_id),
            ] {
                if let Some(val) = value {
                    let target_body = json!({ "noteId": note_id, field: val });
                    if let Err(e) = self.client.create("noteTargets", target_body).await {
                        tracing::warn!("Failed to create note target: {e}");
                    }
                }
            }
        }

        format_record(record)
    }

    #[tool(description = "Update an existing note in TwentyCRM.")]
    async fn update_note(&self, params: Parameters<UpdateNoteParams>) -> String {
        let p = params.0;
        let id = p.id.clone();
        let mut body = json!({});
        if let Some(title) = &p.title {
            body["title"] = json!(title);
        }
        if let Some(text) = &p.body {
            body["bodyV2"] = json!({"markdown": text});
        }
        match self.client.update("notes", &id, body).await {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "Delete a note from TwentyCRM by ID.")]
    async fn delete_note(&self, params: Parameters<DeleteParams>) -> String {
        match self.client.delete("notes", &params.0.id).await {
            Ok(()) => format!("Note {} deleted.", params.0.id),
            Err(e) => format_error(e),
        }
    }

    // ─── Timeline ───────────────────────────────────────────────────

    #[tool(
        description = "Get the timeline of activities for an entity. Shows events like creation, updates, notes, tasks, and other changes. entity_type must be one of: company, contact, opportunity, task."
    )]
    async fn get_timeline(&self, params: Parameters<TimelineParams>) -> String {
        let p = params.0;

        let filter_field = match p.entity_type.as_str() {
            "company" => "targetCompanyId",
            "contact" | "person" => "targetPersonId",
            "opportunity" => "targetOpportunityId",
            "task" => "targetTaskId",
            other => {
                return format!(
                    "Error: Unknown entity_type '{other}'. Must be one of: company, contact, opportunity, task."
                );
            }
        };

        let filter = format!("{filter_field}[eq]:{}", p.entity_id);
        match self
            .client
            .search(
                "timelineActivities",
                None,
                Some(&filter),
                Some("happensAt[DescNullsLast]"),
                p.limit,
                p.starting_after.as_deref(),
                None,
            )
            .await
        {
            Ok(resp) => format_list_result("timelineActivities", resp),
            Err(e) => format_error(e),
        }
    }

    // ─── Workspace Members ──────────────────────────────────────────

    #[tool(
        description = "List workspace members in TwentyCRM. Returns member IDs, names, and emails. Use this to look up member IDs for task assignment or ownership."
    )]
    async fn list_workspace_members(&self, params: Parameters<SearchParams>) -> String {
        let p = params.0;
        match self
            .client
            .search(
                "workspaceMembers",
                p.query.as_deref(),
                p.filter.as_deref(),
                p.order_by.as_deref(),
                p.limit,
                p.starting_after.as_deref(),
                p.fields.as_deref(),
            )
            .await
        {
            Ok(resp) => format_list_result("workspaceMembers", resp),
            Err(e) => format_error(e),
        }
    }

    // ─── Messages ───────────────────────────────────────────────────

    #[tool(
        description = "Search for email messages in TwentyCRM. Filter by linked contact (personId) or search by subject. Messages are synced from connected email accounts."
    )]
    async fn search_messages(&self, params: Parameters<SearchMessagesParams>) -> String {
        let p = params.0;

        // If personId is provided, first find messageParticipants for that person,
        // then look up the messages. Otherwise search messages directly.
        if let Some(person_id) = &p.person_id {
            let filter = format!("personId[eq]:{person_id}");
            let participants = match self
                .client
                .search(
                    "messageParticipants",
                    None,
                    Some(&filter),
                    None,
                    Some(50),
                    None,
                    None,
                )
                .await
            {
                Ok(resp) => resp,
                Err(e) => return format_error(e),
            };

            // Extract unique messageIds from participants
            let message_ids: Vec<String> = participants
                .data
                .as_object()
                .and_then(|obj| obj.values().next())
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| p.get("messageId").and_then(|v| v.as_str()))
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            if message_ids.is_empty() {
                return format_list_result(
                    "messages",
                    crate::types::ListResponse {
                        data: json!([]),
                        page_info: None,
                    },
                );
            }

            // Fetch messages by IDs (up to limit)
            let ids: Vec<&str> = message_ids
                .iter()
                .take(p.limit.unwrap_or(20) as usize)
                .map(|s| s.as_str())
                .collect();
            let id_filter = format!(
                "id[in]:[{}]",
                ids.iter()
                    .map(|id| format!("\"{id}\""))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            match self
                .client
                .search(
                    "messages",
                    None,
                    Some(&id_filter),
                    Some("receivedAt[DescNullsLast]"),
                    p.limit,
                    None,
                    None,
                )
                .await
            {
                Ok(resp) => format_list_result("messages", resp),
                Err(e) => format_error(e),
            }
        } else {
            let mut filter_parts: Vec<String> = Vec::new();
            if let Some(subject) = &p.subject {
                filter_parts.push(format!("subject[ilike]:%{subject}%"));
            }
            if let Some(f) = &p.filter {
                filter_parts.push(f.clone());
            }
            let filter = if filter_parts.is_empty() {
                None
            } else {
                Some(filter_parts.join(","))
            };
            match self
                .client
                .search(
                    "messages",
                    None,
                    filter.as_deref(),
                    Some("receivedAt[DescNullsLast]"),
                    p.limit,
                    p.starting_after.as_deref(),
                    None,
                )
                .await
            {
                Ok(resp) => format_list_result("messages", resp),
                Err(e) => format_error(e),
            }
        }
    }

    #[tool(
        description = "View a specific email message by ID, including subject, text, and participants."
    )]
    async fn view_message(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self
            .client
            .get("messages", &p.id, p.fields.as_deref())
            .await
        {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    #[tool(description = "View all messages in a message thread by thread ID.")]
    async fn view_message_thread(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self
            .client
            .get("messageThreads", &p.id, p.fields.as_deref())
            .await
        {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }

    // ─── Calendar Events ────────────────────────────────────────────

    #[tool(
        description = "Search for calendar events in TwentyCRM. Filter by date range (startsAfter/startsBefore) or search by title. Events are synced from connected calendar accounts."
    )]
    async fn search_calendar_events(
        &self,
        params: Parameters<SearchCalendarEventsParams>,
    ) -> String {
        let p = params.0;
        let mut filter_parts: Vec<String> = Vec::new();
        if let Some(after) = &p.starts_after {
            filter_parts.push(format!("startsAt[gte]:{after}"));
        }
        if let Some(before) = &p.starts_before {
            filter_parts.push(format!("startsAt[lte]:{before}"));
        }
        if let Some(title) = &p.title {
            filter_parts.push(format!("title[ilike]:%{title}%"));
        }
        if let Some(f) = &p.filter {
            filter_parts.push(f.clone());
        }
        let filter = if filter_parts.is_empty() {
            None
        } else {
            Some(filter_parts.join(","))
        };
        match self
            .client
            .search(
                "calendarEvents",
                None,
                filter.as_deref(),
                Some("startsAt[AscNullsLast]"),
                p.limit,
                p.starting_after.as_deref(),
                None,
            )
            .await
        {
            Ok(resp) => format_list_result("calendarEvents", resp),
            Err(e) => format_error(e),
        }
    }

    #[tool(
        description = "View a specific calendar event by ID, including title, times, location, description, and participants."
    )]
    async fn view_calendar_event(&self, params: Parameters<ViewParams>) -> String {
        let p = params.0;
        match self
            .client
            .get("calendarEvents", &p.id, p.fields.as_deref())
            .await
        {
            Ok(record) => format_record(record),
            Err(e) => format_error(e),
        }
    }
}

#[tool_handler]
impl ServerHandler for TwentyCrmMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.instructions = Some(
            "TwentyCRM MCP Server. Manage companies, contacts, opportunities, tasks, and notes. Search email messages and calendar events. View entity timelines for activity history.".into()
        );
        info
    }
}
