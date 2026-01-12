use std::sync::Arc;

use axum::extract::FromRef;

use crate::{infra::config::AppConfig};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.config.as_ref().clone()
    }
}