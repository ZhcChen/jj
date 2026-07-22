use anyhow::Result;
use fund_monitor::{app::config::AppConfig, app_router, build_state};

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env()?;
    let bind_addr = config.bind_addr.clone();
    let state = build_state(config).await?;
    let app = app_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
