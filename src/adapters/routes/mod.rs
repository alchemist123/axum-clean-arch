pub mod health;
pub mod teams;
pub mod admin;
pub mod productivity;

use axum::Router;
use crate::adapters::http::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .nest("/teams", teams::router())
        .nest("/admin", admin::router())
        .nest("/productivity", productivity::router())
}