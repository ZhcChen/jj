use fund_monitor::{
    app::config::AppConfig,
    build_state,
    storage::{app_setting_repo::AppSettingRepo, db},
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
}
