use axum::{
    extract::{Path, State, Request},
    http::{StatusCode, HeaderMap},
    middleware::{self, Next},
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;

use crate::{
    adapters::http::app_state::AppState,
    app_error::AppError,
    application::admin_usecase::{AdminUseCase, AdminLoginRequest, AdminLoginResponse, Claims},
    domain::team::TeamStatus,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminStatusRequest {
    pub status: TeamStatus,
    pub remarks: Option<String>,
}

pub fn router() -> Router<AppState> {
    let protected_routes = Router::new()
        .route("/teams/{id}/status", post(update_team_status))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .route("/login", post(login))
        .merge(protected_routes)
}

async fn login(
    State(app_state): State<AppState>,
    Json(request): Json<AdminLoginRequest>,
) -> Result<Json<AdminLoginResponse>, AppError> {
    let usecase = AdminUseCase::new(app_state.team_repository.clone());
    let response = usecase.login(request).await?;
    Ok(Json(response))
}

async fn update_team_status(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<AdminStatusRequest>,
) -> Result<StatusCode, AppError> {
    let usecase = AdminUseCase::new(app_state.team_repository.clone());
    usecase.update_team_status(id, request.status, request.remarks).await?;
    Ok(StatusCode::OK)
}

async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::InternalError("Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InternalError("Invalid authorization header".to_string()));
    }

    let token = &auth_header[7..];
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    let _token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| AppError::InternalError(format!("Invalid token: {}", e)))?;

    Ok(next.run(request).await)
}
