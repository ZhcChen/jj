pub mod app;
pub mod domain;
pub mod jobs;
pub mod notifications;
pub mod providers;
pub mod storage;
pub mod web;

use anyhow::Result;
use app::{config::AppConfig, state::AppState};
use axum::Router;
use notifications::telegram::TelegramNotifier;
use providers::{default_fund_source, fund_source::EastmoneyFundSource};
use storage::db;

pub async fn build_state(config: AppConfig) -> Result<AppState> {
    let fund_source = default_fund_source()?;
    build_state_with_fund_source(config, fund_source).await
}

pub async fn build_state_with_fund_source(
    config: AppConfig,
    fund_source: EastmoneyFundSource,
) -> Result<AppState> {
    let pool = db::initialize_database(&config.database_url).await?;
    let telegram_notifier = TelegramNotifier::from_app_config(&config)?;
    Ok(AppState::new(config, pool, fund_source, telegram_notifier))
}

pub fn app_router(state: AppState) -> Router {
    web::router(state)
}
