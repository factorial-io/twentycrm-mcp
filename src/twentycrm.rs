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

        // Build filter: combine query (ilike on name) with explicit filter
        let mut filter_parts: Vec<String> = Vec::new();

        if let Some(q) = query {
            let search_field = match entity {
                "people" => "name.firstName[ilike]",
                _ => "name[ilike]",
            };
            filter_parts.push(format!("{search_field}:%{q}%"));
        }

        if let Some(f) = filter {
            filter_parts.push(f.to_string());
        }

        if !filter_parts.is_empty() {
            req = req.query(&[("filter", filter_parts.join(","))]);
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
fn extract_error_message(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<Value>(body) {
        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            return err.to_string();
        }
    }
    body.chars().take(200).collect()
}
