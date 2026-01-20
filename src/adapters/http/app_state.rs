use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    domain::repository::TeamRepository,
    infra::config::AppConfig,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub team_repository: Arc<dyn TeamRepository + Send + Sync>,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>, team_repository: Arc<dyn TeamRepository + Send + Sync>) -> Self {
        Self {
            config,
            team_repository,
        }
    }
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.config.as_ref().clone()
    }
}