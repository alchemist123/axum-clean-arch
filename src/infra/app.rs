use axum::Router;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use create::{
    adapters::{self, http::app_state:AppState},
    infra::setup::init_tracing
};

pub fn create_app(app_state: AppState) -> Router {
    init_tracing();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);

    Router::new().nest("/api/v1", adapters::http::routes::router())
    .with_state(app_state)
    .layer(cors)
    .layer(
        TraceLayer::new_for_http()
        .make_span_with(|request: &http::Request<_>|){
            let request_id = Uuid::new_v4();
            tracing::info_span("http_request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
                request_id = %request_id
            );
        }
    )
}