use crate::app::config::AppConfig;
use crate::providers::fund_source::EastmoneyFundSource;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub fund_source: Arc<EastmoneyFundSource>,
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(config: AppConfig, pool: SqlitePool, fund_source: EastmoneyFundSource) -> Self {
        Self {
            config: Arc::new(config),
            fund_source: Arc::new(fund_source),
            pool,
        }
    }
}
