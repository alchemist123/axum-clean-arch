use axum::{Router, http};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use tower_http::{cors::{CorsLayer, AllowOrigin, AllowMethods, AllowHeaders}, trace::TraceLayer};
use uuid::Uuid;

use crate::{
    adapters::{self, http::app_state::AppState},
    infra::setup::init_tracing
};

pub fn create_app(app_state: AppState) -> Router {
    init_tracing();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .allow_credentials(true);

    Router::new().nest("/api/v1", adapters::http::routes::router())
    .with_state(app_state)
    .layer(cors)
    .layer(
        TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
            let request_id = Uuid::new_v4();
            tracing::info_span!(
                "http-request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
                request_id = %request_id
            )
        }),
    )
}