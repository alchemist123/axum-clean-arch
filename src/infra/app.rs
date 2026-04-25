use axum::{Router, http};
use http::{header::{AUTHORIZATION, CONTENT_TYPE, ACCEPT}, Method};
use tower_http::{cors::{CorsLayer, AllowOrigin, AllowMethods, AllowHeaders}, trace::TraceLayer};
use uuid::Uuid;

use crate::{
    adapters::{self, http::app_state::AppState},
    infra::setup::init_tracing
};

pub fn create_app(app_state: AppState) -> Router {
    init_tracing().expect("Failed to initialize tracing");

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            "http://localhost:3000".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://127.0.0.1:5173".parse().unwrap(),
            "https://hackthon..in".parse().unwrap(),
        ]))
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            http::header::ORIGIN,
            http::header::REFERER,
            http::header::USER_AGENT,
        ]))
        .allow_credentials(true);

    Router::new().nest("/api/v1", adapters::routes::router())
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