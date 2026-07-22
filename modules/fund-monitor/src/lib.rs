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
use domain::fund::NewFund;
use notifications::telegram::TelegramNotifier;
use providers::{default_fund_source, fund_source::EastmoneyFundSource};
use sqlx::SqlitePool;
use storage::{db, fund_repo::FundRepo};

const DEFAULT_FUND_CODE: &str = "012734";
const DEFAULT_FUND_NAME: &str = "易方达中证人工智能主题ETF联接C";

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

pub async fn ensure_default_funds(pool: &SqlitePool) -> Result<()> {
    let fund_repo = FundRepo::new(pool.clone());

    if fund_repo.find_by_code(DEFAULT_FUND_CODE).await?.is_some() {
        return Ok(());
    }

    fund_repo
        .create(NewFund {
            code: DEFAULT_FUND_CODE.to_owned(),
            name: DEFAULT_FUND_NAME.to_owned(),
            note: None,
            group_name: None,
            tags: None,
            enabled: true,
        })
        .await?;

    Ok(())
}
