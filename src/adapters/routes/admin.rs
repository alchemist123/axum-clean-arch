use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    adapters::http::{app_state::AppState, jwt_middleware::require_admin_jwt},
    app_error::AppError,
    application::admin_usecase::{AdminUseCase, AdminLoginRequest, AdminLoginResponse},
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
        .route("/teams/{id}", delete(delete_team))
        .layer(middleware::from_fn(require_admin_jwt));

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

async fn delete_team(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let usecase = AdminUseCase::new(app_state.team_repository.clone());
    usecase.delete_team(id).await?;
    Ok(StatusCode::OK)
}
