use crate::app_error::AppError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Application error");

        match self {
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg)).into_response(),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", msg)).into_response(),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, format!("Validation error: {}", msg)).into_response(),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, format!("Not found: {}", msg)).into_response(),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, format!("Conflict: {}", msg)).into_response(),
        }
    }
}