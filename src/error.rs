// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Authentication required")]
    Auth(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("TwentyCRM API error: {status} - {message}")]
    TwentyCrmApi { status: u16, message: String },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("OIDC error: {0}")]
    Oidc(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::SessionNotFound => StatusCode::UNAUTHORIZED,
            AppError::TwentyCrmApi { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Encryption(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Oidc(_) => StatusCode::BAD_GATEWAY,
            AppError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns a safe message for the client (no internal details leaked).
    pub fn client_message(&self) -> String {
        match self {
            AppError::Redis(_) => "Internal error".to_string(),
            AppError::Auth(msg) => msg.clone(),
            AppError::SessionNotFound => "Session expired or invalid".to_string(),
            AppError::TwentyCrmApi { message, .. } => message.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Encryption(_) => "Internal error".to_string(),
            AppError::Internal(_) => "Internal error".to_string(),
            AppError::Oidc(_) => "Authentication provider error".to_string(),
            AppError::Serialization(_) => "Internal error".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let client_msg = self.client_message();

        // Log internal details server-side
        match &self {
            AppError::Redis(_)
            | AppError::Encryption(_)
            | AppError::Internal(_)
            | AppError::Serialization(_) => {
                tracing::error!("{self}");
            }
            AppError::Oidc(_) | AppError::TwentyCrmApi { .. } => {
                tracing::warn!("{self}");
            }
            _ => {
                tracing::debug!("{self}");
            }
        }

        let body = json!({
            "error": status.canonical_reason().unwrap_or("Unknown"),
            "message": client_msg,
        });

        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_error_is_500() {
        let err = AppError::Internal("test".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_auth_error_is_401() {
        let err = AppError::Auth("bad token".into());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_session_not_found_is_401() {
        let err = AppError::SessionNotFound;
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_not_found_is_404() {
        let err = AppError::NotFound("Company xyz not found".into());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_validation_is_400() {
        let err = AppError::Validation("missing name".into());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_twentycrm_api_error_status() {
        let err = AppError::TwentyCrmApi {
            status: 429,
            message: "Rate limit".into(),
        };
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_internal_errors_hide_details() {
        let err = AppError::Encryption("AES key invalid".into());
        assert_eq!(err.client_message(), "Internal error");
    }

    #[test]
    fn test_auth_errors_show_message() {
        let err = AppError::Auth("Token expired".into());
        assert_eq!(err.client_message(), "Token expired");
    }
}
