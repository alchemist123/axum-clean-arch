use dotenvy::dotenv;

use tracing::info;

use profile_blog_axum::infra::{app::create_app, setup::init_app_state};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let app_state = init_app_state().await?;

    let app = create_app((*app_state).clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Starting server on port 3000");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}