// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reqwest::Client;
use serde_json::Value;

use crate::error::AppError;
use crate::types::ListResponse;

#[derive(Clone)]
pub struct TwentyCrmClient {
    client: Client,
    base_url: String,
    api_token: String,
}

impl TwentyCrmClient {
    pub fn new(base_url: &str, api_token: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token: api_token.to_string(),
        }
    }

    /// Search/list records for an entity with filtering, pagination, and ordering.
    #[allow(clippy::too_many_arguments)]
    pub async fn search(
        &self,
        entity: &str,
        query: Option<&str>,
        filter: Option<&str>,
        order_by: Option<&str>,
        limit: Option<i64>,
        starting_after: Option<&str>,
        _fields: Option<&[String]>,
    ) -> Result<ListResponse, AppError> {
        let url = format!("{}/{entity}", self.base_url);

        let mut req = self.client.get(&url).bearer_auth(&self.api_token);

        // Build filter: combine query with explicit filter.
        // For people, the query searches across name and email using OR.
        // For other entities, it searches by name.
        if let Some(q) = query {
            match entity {
                "people" => {
                    // TwentyCRM OR filter: search name and email
                    let name_filter = format!("name.firstName[ilike]:%{q}%");
                    let last_filter = format!("name.lastName[ilike]:%{q}%");
                    let email_filter = format!("emails.primaryEmail[ilike]:%{q}%");
                    req = req.query(&[(
                        "filter",
                        format!("or({name_filter},{last_filter},{email_filter})"),
                    )]);
                }
                _ => {
                    req = req.query(&[("filter", format!("name[ilike]:%{q}%"))]);
                }
            }
        }

        if let Some(f) = filter {
            req = req.query(&[("filter", f.to_string())]);
        }

        if let Some(ob) = order_by {
            req = req.query(&[("order_by", ob)]);
        }

        if let Some(lim) = limit {
            req = req.query(&[("limit", lim.to_string())]);
        }

        if let Some(cursor) = starting_after {
            req = req.query(&[("starting_after", cursor)]);
        }

        let response = req.send().await.map_err(|e| AppError::TwentyCrmApi {
            status: 502,
            message: format!("Failed to connect to TwentyCRM: {e}"),
        })?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body = response.text().await.unwrap_or_default();
            return Err(map_api_error(status, &body));
        }

        let list: ListResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse TwentyCRM response: {e}")))?;

        Ok(list)
    }

    /// Get a single record by ID.
    pub async fn get(
        &self,
        entity: &str,
        id: &str,
        _fields: Option<&[String]>,
    ) -> Result<Value, AppError> {
        let url = format!("{}/{entity}/{id}", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .map_err(|e| AppError::TwentyCrmApi {
                status: 502,
                message: format!("Failed to connect to TwentyCRM: {e}"),
            })?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body = response.text().await.unwrap_or_default();
            return Err(map_api_error(status, &body));
        }

        let record: Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse TwentyCRM response: {e}")))?;

        Ok(record)
    }

    /// Create a new record.
    pub async fn create(&self, entity: &str, body: Value) -> Result<Value, AppError> {
        let url = format!("{}/{entity}", self.base_url);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::TwentyCrmApi {
                status: 502,
                message: format!("Failed to connect to TwentyCRM: {e}"),
            })?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body = response.text().await.unwrap_or_default();
            return Err(map_api_error(status, &body));
        }

        let record: Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse TwentyCRM response: {e}")))?;

        Ok(record)
    }

    /// Update an existing record by ID (PATCH).
    pub async fn update(&self, entity: &str, id: &str, body: Value) -> Result<Value, AppError> {
        let url = format!("{}/{entity}/{id}", self.base_url);

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.api_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::TwentyCrmApi {
                status: 502,
                message: format!("Failed to connect to TwentyCRM: {e}"),
            })?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body = response.text().await.unwrap_or_default();
            return Err(map_api_error(status, &body));
        }

        let record: Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse TwentyCRM response: {e}")))?;

        Ok(record)
    }

    /// Delete a record by ID.
    pub async fn delete(&self, entity: &str, id: &str) -> Result<(), AppError> {
        let url = format!("{}/{entity}/{id}", self.base_url);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .map_err(|e| AppError::TwentyCrmApi {
                status: 502,
                message: format!("Failed to connect to TwentyCRM: {e}"),
            })?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body = response.text().await.unwrap_or_default();
            return Err(map_api_error(status, &body));
        }

        Ok(())
    }
}

/// Map TwentyCRM API HTTP status codes to AppError.
fn map_api_error(status: u16, body: &str) -> AppError {
    let message = extract_error_message(body);

    match status {
        401 => AppError::TwentyCrmApi {
            status,
            message: format!("TwentyCRM API authentication failed: {message}"),
        },
        404 => AppError::NotFound(format!("Record not found: {message}")),
        400 => AppError::Validation(format!("Invalid request: {message}")),
        429 => AppError::TwentyCrmApi {
            status,
            message: "TwentyCRM API rate limit exceeded. Please try again later.".into(),
        },
        _ => AppError::TwentyCrmApi { status, message },
    }
}

/// Try to extract a useful error message from the API response body.
/// TwentyCRM returns various shapes: {"message": "..."}, {"error": "...", "messages": [...]},
/// or {"message": ["..."]}, so we try to combine all useful info.
fn extract_error_message(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<Value>(body) {
        let mut parts: Vec<String> = Vec::new();

        // "error" field (e.g. "BadRequestException")
        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            parts.push(err.to_string());
        }

        // "message" field — can be a string or array of strings
        match json.get("message") {
            Some(Value::String(s)) => parts.push(s.clone()),
            Some(Value::Array(arr)) => {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        parts.push(s.to_string());
                    }
                }
            }
            _ => {}
        }

        // "messages" field (array)
        if let Some(Value::Array(arr)) = json.get("messages") {
            for item in arr {
                if let Some(s) = item.as_str() {
                    parts.push(s.to_string());
                }
            }
        }

        if !parts.is_empty() {
            return parts.join(": ");
        }
    }
    body.chars().take(500).collect()
}
