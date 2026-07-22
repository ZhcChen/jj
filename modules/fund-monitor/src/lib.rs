pub mod app;
pub mod domain;
pub mod providers;
pub mod storage;
pub mod web;

use anyhow::Result;
use app::{config::AppConfig, state::AppState};
use axum::Router;
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
    Ok(AppState::new(config, pool, fund_source))
}

pub fn app_router(state: AppState) -> Router {
    web::router(state)
}
