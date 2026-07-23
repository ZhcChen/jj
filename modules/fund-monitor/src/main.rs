use anyhow::Result;
use fund_monitor::{
    app::{config::AppConfig, logging},
    build_state, ensure_default_funds, jobs,
};

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;
    let config = AppConfig::from_env()?;
    let state = build_state(config).await?;
    ensure_default_funds(&state.pool).await?;
    jobs::start_scheduler(state.clone());
    tracing::info!(
        poll_interval_seconds = state.config.poll_interval_seconds,
        default_fund_code = "012734",
        "fund-monitor scheduler daemon started"
    );
    std::future::pending::<()>().await;

    Ok(())
}
