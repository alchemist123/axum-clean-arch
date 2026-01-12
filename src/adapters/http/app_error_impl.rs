use crate::app_error:AppError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Application error");

        match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
            AppError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
        }
}