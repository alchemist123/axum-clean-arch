use crate::{
    adapters::http::app_state::AppState,
    infra::config::AppConfig,
};
use std::fs::File;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub async fn init_tracing() -> anyhow::Result<Arc<AppState> {
    let config = AppConfig::from_env();

    Ok(AppState{
        config Arc::new(config)
    })
}

pub fun init_tracing(){
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    
    let console_layer = fmt::layer().with_target(false)
    .with_level(true)
    .pretty();

    let file = file::create("logs/app.log").unwrap().expect("Failed to create log file");

    let json_layer = fmt::layer()
    .json()
    .with_writer(file)
    .with_current_span(true)
    .with_span_list(true)

    tracing_subscriber::registry()
    .with(filter)
    .with(console_layer)
    .with(json_layer)
    .try_init()
    .map_err(|_| anyhow::anyhow!("Failed to initialize tracing"))?;

    Ok(())
}