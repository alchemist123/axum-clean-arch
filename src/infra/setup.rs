use crate::{
    adapters::http::app_state::AppState,
    infra::config::AppConfig,
};
use std::fs::{File, create_dir_all};
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub async fn init_app_state() -> anyhow::Result<Arc<AppState>>{
    let config = AppConfig::from_env();

    Ok(Arc::new(AppState{
        config: Arc::new(config)
    }))
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