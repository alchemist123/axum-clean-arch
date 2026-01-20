use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use crate::{
    adapters::http::app_state::AppState,
    app_error::AppError,
    application::team_usecase::TeamUseCase,
    domain::team::{
        RegisterTeamRequest, ListTeamsQuery, CheckMemberRequest, TeamResponse, TeamCountResponse,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(register_team))
        .route("/", get(list_teams))
        .route("/count", get(get_team_count))
        .route("/check-member", get(check_member))
        .route("/{id}", get(get_team_by_id))
}

async fn register_team(
    State(app_state): State<AppState>,
    Json(request): Json<RegisterTeamRequest>,
) -> Result<(StatusCode, Json<TeamResponse>), AppError> {
    let usecase = TeamUseCase::new(app_state.team_repository.clone());
    let team = usecase.register_team(request).await?;
    Ok((StatusCode::CREATED, Json(team)))
}

async fn list_teams(
    State(app_state): State<AppState>,
    Query(query): Query<ListTeamsQuery>,
) -> Result<Json<crate::domain::team::PaginatedTeamsResponse>, AppError> {
    let usecase = TeamUseCase::new(app_state.team_repository.clone());
    let result = usecase.list_teams(query).await?;
    Ok(Json(result))
}

async fn get_team_by_id(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamResponse>, AppError> {
    let usecase = TeamUseCase::new(app_state.team_repository.clone());
    let team = usecase.get_team_by_id(id).await?;
    Ok(Json(team))
}

async fn check_member(
    State(app_state): State<AppState>,
    Query(query): Query<CheckMemberRequest>,
) -> Result<Json<crate::domain::team::CheckMemberResponse>, AppError> {
    let usecase = TeamUseCase::new(app_state.team_repository.clone());
    let result = usecase.check_member(query).await?;
    Ok(Json(result))
}

async fn get_team_count(
    State(app_state): State<AppState>,
) -> Result<Json<TeamCountResponse>, AppError> {
    let usecase = TeamUseCase::new(app_state.team_repository.clone());
    let result = usecase.get_team_count().await?;
    Ok(Json(result))
}

