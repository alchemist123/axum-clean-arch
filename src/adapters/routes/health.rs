use axum::{
    Json,
    Router,
    extract::State,
    routing::get
};

use crate::{
    adapters::http::app_state::AppState, app_error::AppError
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(health_check))
}

async fn health_check(State(_app_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "status": "ok",
        "database": "connected"
    })))
}