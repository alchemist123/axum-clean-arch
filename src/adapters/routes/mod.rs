pub mod health;
pub mod teams;

use axum::Router;
use crate::adapters::http::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .nest("/teams", teams::router())
}