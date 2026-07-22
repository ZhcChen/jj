use crate::app::config::AppConfig;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(config: AppConfig, pool: SqlitePool) -> Self {
        Self {
            config: Arc::new(config),
            pool,
        }
    }
}
