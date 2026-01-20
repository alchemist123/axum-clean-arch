use dotenvy::dotenv;

use tracing::info;

use profile_blog_axum::infra::{app::create_app, setup::init_app_state};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let app_state = init_app_state().await?;

    let app = create_app((*app_state).clone());
    // Bind to 0.0.0.0 to allow connections from outside the container (Docker)
    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    info!("Starting server on {}", bind_address);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}