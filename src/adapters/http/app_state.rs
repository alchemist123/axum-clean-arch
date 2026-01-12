use std::sync::Arc;

use axum::exttract::FromRef;

use crate::{infra::config::AppConfig};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(app_state: &AppState) -> self {
        app_state.config.clone()
    }
}