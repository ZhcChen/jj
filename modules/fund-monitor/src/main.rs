use anyhow::Result;
use fund_monitor::{
    app::{config::AppConfig, logging},
    app_router, build_state, ensure_default_funds, jobs,
};

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;
    let config = AppConfig::from_env()?;
    let bind_addr = config.bind_addr.clone();
    let state = build_state(config).await?;
    ensure_default_funds(&state.pool).await?;
    jobs::start_scheduler(state.clone());
    tracing::info!(bind_addr = %bind_addr, "fund-monitor application starting");
    let app = app_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
