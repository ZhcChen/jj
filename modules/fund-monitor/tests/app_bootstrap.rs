use fund_monitor::{
    app::config::AppConfig,
    build_state, ensure_default_funds,
    storage::{app_setting_repo::AppSettingRepo, db, fund_repo::FundRepo},
};
use tempfile::tempdir;

#[tokio::test]
async fn bootstrap_initializes_sqlite_and_runs_migrations() {
    let temp_dir = tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("fund-monitor.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 300,
        telegram_api_base_url: "https://api.telegram.org".to_owned(),
        telegram_bot_token: None,
        telegram_chat_id: None,
    };

    let state = build_state(config).await.expect("build app state");
    db::health_check(&state.pool)
        .await
        .expect("db health check");

    assert!(db_path.exists(), "database file should be created");

    let settings = AppSettingRepo::new(state.pool.clone());
    let saved = settings
        .set("poll_interval_seconds", "300")
        .await
        .expect("insert setting");

    assert_eq!(saved.key, "poll_interval_seconds");
    assert_eq!(saved.value, "300");

    ensure_default_funds(&state.pool)
        .await
        .expect("seed default fund");
    ensure_default_funds(&state.pool)
        .await
        .expect("seed default fund idempotently");

    let funds = FundRepo::new(state.pool.clone())
        .list_active()
        .await
        .expect("list active funds");
    let default_fund = funds
        .iter()
        .find(|fund| fund.code == "012734")
        .expect("default fund exists");

    assert_eq!(default_fund.name, "易方达中证人工智能主题ETF联接C");
    assert_eq!(funds.iter().filter(|fund| fund.code == "012734").count(), 1);
}
