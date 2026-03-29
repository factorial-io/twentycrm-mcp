use serde_json::json;

use crate::types::ListResponse;

pub fn format_list_result(entity: &str, resp: ListResponse) -> String {
    // The TwentyCRM list response has `data: {"people": [...]}` or `data: {"companies": [...]}`
    // Unwrap the inner array for a consistent shape
    let data = resp
        .data
        .as_object()
        .and_then(|obj| {
            if obj.len() == 1 {
                obj.values().next().cloned()
            } else {
                None
            }
        })
        .unwrap_or(resp.data);

    let mut result = json!({ "entity": entity, "data": data });
    if let Some(pi) = resp.page_info {
        result["pageInfo"] = json!({
            "hasNextPage": pi.has_next_page,
            "startCursor": pi.start_cursor,
            "endCursor": pi.end_cursor,
        });
    }
    serde_json::to_string_pretty(&result).unwrap_or_default()
}

/// Unwrap the TwentyCRM single-record response shape.
/// The API returns `{"data": {"person": {...}}}` — extract the inner object
/// so the tool returns a flat record regardless of entity type.
pub fn format_record(record: serde_json::Value) -> String {
    let unwrapped = record
        .get("data")
        .and_then(|data| data.as_object())
        .and_then(|obj| {
            if obj.len() == 1 {
                obj.values().next().cloned()
            } else {
                None
            }
        })
        .unwrap_or(record);

    serde_json::to_string_pretty(&unwrapped).unwrap_or_default()
}

pub fn format_error(err: crate::error::AppError) -> String {
    format!("Error: {}", err.client_message())
}

pub fn strip_nulls(value: serde_json::Value) -> serde_json::Value {
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
