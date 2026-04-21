use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use wardnet_common::api::ApiError;

use crate::request_context;

/// Application-level error type that maps to HTTP responses.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("conflict: {0}")]
    Conflict(String),

    /// An external service (release manifest host, provider API, etc.) failed
    /// in a way we want the caller to see verbatim. Mapped to 502 Bad Gateway
    /// and the string is surfaced in the response `detail` field.
    #[error("upstream unavailable: {0}")]
    UpstreamUnavailable(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, detail) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "not found", Some(msg.clone())),
            Self::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "unauthorized", Some(msg.clone()))
            }
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", Some(msg.clone())),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad request", Some(msg.clone())),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "conflict", Some(msg.clone())),
            Self::UpstreamUnavailable(msg) => {
                // Log at warn-level with the cause in the message so the
                // recent-errors feed captures it; not a programmer bug so
                // not an error-level event.
                tracing::warn!("upstream unavailable: {msg}");
                (
                    StatusCode::BAD_GATEWAY,
                    "upstream unavailable",
                    Some(msg.clone()),
                )
            }
            Self::Internal(err) => {
                tracing::error!(error = %err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error",
                    None,
                )
            }
            Self::Database(err) => {
                tracing::error!(error = %err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error",
                    None,
                )
            }
        };

        let body = ApiError {
            error: error_message.to_owned(),
            detail,
            request_id: request_context::current_request_id(),
        };

        (status, Json(body)).into_response()
    }
}
