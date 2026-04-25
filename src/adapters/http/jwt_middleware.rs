use std::env;

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};

use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::application::admin_usecase::Claims;
use crate::app_error::AppError;

/// Validates `Authorization: Bearer <jwt>` with `JWT_SECRET` (default `secret`).
pub async fn require_admin_jwt(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::InvalidCredentials)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InvalidCredentials);
    }

    let token = &auth_header[7..];
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    let _ = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| AppError::InternalError(format!("Invalid token: {}", e)))?;

    Ok(next.run(request).await)
}
