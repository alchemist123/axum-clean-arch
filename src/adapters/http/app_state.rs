use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    domain::productivity_repository::ProductivityRepository,
    domain::repository::TeamRepository,
    infra::config::AppConfig,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub team_repository: Arc<dyn TeamRepository + Send + Sync>,
    pub productivity_repository: Arc<dyn ProductivityRepository + Send + Sync>,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        team_repository: Arc<dyn TeamRepository + Send + Sync>,
        productivity_repository: Arc<dyn ProductivityRepository + Send + Sync>,
    ) -> Self {
        Self {
            config,
            team_repository,
            productivity_repository,
        }
    }
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.config.as_ref().clone()
    }
}