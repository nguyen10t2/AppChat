use std::sync::Arc;

use crate::{configs::AppConfig, observability::AppMetrics};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub metrics: Arc<AppMetrics>,
}

impl AppState {
    pub fn new(config: AppConfig, metrics: AppMetrics) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(metrics),
        }
    }
}
