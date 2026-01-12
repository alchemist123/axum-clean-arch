pub mod health;

use axum::Router;
use crate::adapters::http::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().nest("/health", health::router())
}