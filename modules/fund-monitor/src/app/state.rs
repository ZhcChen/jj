use crate::app::config::AppConfig;
use crate::notifications::telegram::TelegramNotifier;
use crate::providers::fund_source::EastmoneyFundSource;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub fund_source: Arc<EastmoneyFundSource>,
    pub telegram_notifier: Option<Arc<TelegramNotifier>>,
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        pool: SqlitePool,
        fund_source: EastmoneyFundSource,
        telegram_notifier: Option<TelegramNotifier>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            fund_source: Arc::new(fund_source),
            telegram_notifier: telegram_notifier.map(Arc::new),
            pool,
        }
    }
}
