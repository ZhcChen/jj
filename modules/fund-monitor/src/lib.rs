pub mod app;
pub mod domain;
pub mod storage;
pub mod web;

use anyhow::Result;
use app::{config::AppConfig, state::AppState};
use axum::Router;
use storage::db;

pub async fn build_state(config: AppConfig) -> Result<AppState> {
    let pool = db::initialize_database(&config.database_url).await?;
    Ok(AppState::new(config, pool))
}

pub fn app_router(state: AppState) -> Router {
    web::router(state)
}
