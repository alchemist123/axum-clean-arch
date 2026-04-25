use crate::{
    adapters::{
        http::app_state::AppState,
        persistence::{
            productivity_repository::PostgresProductivityRepository,
            team_repository::PostgresTeamRepository,
        },
    },
    domain::productivity_repository::ProductivityRepository,
    infra::{config::AppConfig, db::init_db},
};
use std::fs::{File, create_dir_all};
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub async fn init_app_state() -> anyhow::Result<Arc<AppState>>{
    let config = AppConfig::from_env();
    let pool = init_db().await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;
    
    let team_repository: Arc<PostgresTeamRepository> = Arc::new(PostgresTeamRepository::new(pool.clone()));
    let productivity_repository: Arc<PostgresProductivityRepository> =
        Arc::new(PostgresProductivityRepository::new(pool));

    Ok(Arc::new(AppState::new(
        Arc::new(config),
        team_repository as Arc<dyn crate::domain::repository::TeamRepository + Send + Sync>,
        productivity_repository as Arc<dyn ProductivityRepository + Send + Sync>,
    )))
}

pub fn init_tracing() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    
    let console_layer = fmt::layer().with_target(false)
    .with_level(true)
    .pretty();

    create_dir_all("logs").map_err(|e| anyhow::anyhow!("Failed to create logs directory: {}", e))?;
    let file = File::create("logs/app.log").map_err(|e| anyhow::anyhow!("Failed to create log file: {}", e))?;

    let json_layer = fmt::layer()
    .json()
    .with_writer(file)
    .with_current_span(true)
    .with_span_list(true);

    tracing_subscriber::registry()
    .with(filter)
    .with(console_layer)
    .with(json_layer)
    .try_init()
    .map_err(|_| anyhow::anyhow!("Failed to initialize tracing"))?;

    Ok(())
}