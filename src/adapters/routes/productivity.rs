use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Json},
    routing::{get, patch, post, put},
    Router,
};
use uuid::Uuid;

use crate::{
    adapters::http::{app_state::AppState, jwt_middleware::require_admin_jwt},
    app_error::AppError,
    application::productivity_usecase::ProductivityUseCase,
    domain::productivity::{
        AddQueueItemRequest, AddScheduleEventRequest, LayoutResponse, PatchQueueItemRequest,
        ProductivityPeriod, ScheduleQuery, StartTaskRequest,
    },
};

pub fn router() -> Router<AppState> {
    let protected = Router::new()
        .route("/tasks/active", get(get_active_task))
        .route("/tasks/start", post(start_task))
        .route("/tasks/{task_id}/stop", post(stop_task))
        .route("/stats", get(get_stats))
        .route("/queue", get(list_queue).post(add_queue_item))
        .route("/queue/{item_id}", patch(patch_queue_item))
        .route("/schedule", get(list_schedule).post(add_schedule_event))
        .route("/layout", get(get_layout).put(put_layout))
        .layer(middleware::from_fn(require_admin_jwt));

    protected
}

async fn get_active_task(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    let task = usecase.get_active_task().await?;
    match task {
        Some(t) => Ok((StatusCode::OK, Json(t)).into_response()),
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}

async fn start_task(
    State(state): State<AppState>,
    Json(body): Json<StartTaskRequest>,
) -> Result<(StatusCode, Json<crate::domain::productivity::ActiveTaskResponse>), AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    let task = usecase.start_task(body).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

async fn stop_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<crate::domain::productivity::StopTaskResponse>, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    Ok(Json(usecase.stop_task(task_id).await?))
}

async fn get_stats(
    State(state): State<AppState>,
    Query(q): Query<StatsQueryParams>,
) -> Result<Json<crate::domain::productivity::ProductivityStatsResponse>, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    let out = usecase.get_stats(q.period).await?;
    Ok(Json(out))
}

#[derive(serde::Deserialize, Default)]
struct StatsQueryParams {
    period: Option<ProductivityPeriod>,
}

async fn list_queue(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::productivity::QueueItemResponse>>, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    Ok(Json(usecase.list_queue().await?))
}

async fn add_queue_item(
    State(state): State<AppState>,
    Json(body): Json<AddQueueItemRequest>,
) -> Result<(StatusCode, Json<crate::domain::productivity::QueueItemResponse>), AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    let out = usecase.add_queue_item(body).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

async fn patch_queue_item(
    State(state): State<AppState>,
    Path(item_id): Path<Uuid>,
    Json(body): Json<PatchQueueItemRequest>,
) -> Result<Json<crate::domain::productivity::QueueItemResponse>, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    Ok(Json(usecase.patch_queue_item(item_id, body).await?))
}

async fn list_schedule(
    State(state): State<AppState>,
    Query(q): Query<ScheduleQuery>,
) -> Result<Json<Vec<crate::domain::productivity::ScheduleEventResponse>>, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    Ok(Json(usecase.list_schedule(q).await?))
}

async fn add_schedule_event(
    State(state): State<AppState>,
    Json(body): Json<AddScheduleEventRequest>,
) -> Result<(StatusCode, Json<crate::domain::productivity::ScheduleEventResponse>), AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    let out = usecase.add_schedule_event(body).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

async fn get_layout(
    State(state): State<AppState>,
) -> Result<Json<LayoutResponse>, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    Ok(Json(usecase.get_layout().await?))
}

async fn put_layout(
    State(state): State<AppState>,
    Json(body): Json<LayoutResponse>,
) -> Result<StatusCode, AppError> {
    let usecase = ProductivityUseCase::new(state.productivity_repository.clone());
    usecase.put_layout(body).await?;
    Ok(StatusCode::OK)
}
