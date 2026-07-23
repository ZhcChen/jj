use fund_monitor::{
    app::config::AppConfig,
    build_state, ensure_default_funds,
    storage::{app_setting_repo::AppSettingRepo, db, fund_repo::FundRepo},
};
use sqlx::{
    Row,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::str::FromStr;
use tempfile::tempdir;

#[tokio::test]
async fn bootstrap_initializes_sqlite_and_runs_migrations() {
    let temp_dir = tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("fund-monitor.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        database_url,
        poll_interval_seconds: 60,
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
        .set("poll_interval_seconds", "60")
        .await
        .expect("insert setting");

    assert_eq!(saved.key, "poll_interval_seconds");
    assert_eq!(saved.value, "60");

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

#[tokio::test]
async fn quote_timestamp_backfill_migration_repairs_legacy_rows() {
    let temp_dir = tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("fund-monitor.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let options = SqliteConnectOptions::from_str(&database_url)
        .expect("parse sqlite url")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect bootstrap db");

    sqlx::raw_sql(include_str!(
        "../migrations/202607220001_initial_schema.sql"
    ))
    .execute(&pool)
    .await
    .expect("apply initial schema");

    sqlx::query(
        "
        INSERT INTO funds (id, code, name, note, group_name, tags, enabled, created_at, updated_at)
        VALUES (1, '012734', '测试基金', NULL, NULL, NULL, 1, '2026-07-22T08:40:00Z', '2026-07-22T08:40:00Z')
        ",
    )
    .execute(&pool)
    .await
    .expect("insert test fund");

    sqlx::query(
        "
        INSERT INTO fund_quotes (
            id, fund_id, unit_nav, estimated_nav, change_rate, fetched_at, source, created_at
        )
        VALUES
            (
                1,
                1,
                2.2785,
                NULL,
                7.17,
                '2026-07-20T16:00:00Z',
                'eastmoney/pingzhongdata',
                '2026-07-22T09:03:11.123462Z'
            ),
            (
                2,
                1,
                2.2786,
                NULL,
                7.18,
                '2026-07-22T16:00:00Z',
                'eastmoney/pingzhongdata',
                '2026-07-22T16:00:01Z'
            )
        ",
    )
    .execute(&pool)
    .await
    .expect("insert quote history");

    sqlx::raw_sql(include_str!(
        "../migrations/202607220002_backfill_legacy_quote_timestamps.sql"
    ))
    .execute(&pool)
    .await
    .expect("apply backfill migration");

    let repaired = sqlx::query("SELECT fetched_at, created_at FROM fund_quotes WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("load repaired quote");
    assert_eq!(
        repaired.get::<String, _>("fetched_at"),
        repaired.get::<String, _>("created_at")
    );

    let untouched = sqlx::query("SELECT fetched_at, created_at FROM fund_quotes WHERE id = 2")
        .fetch_one(&pool)
        .await
        .expect("load untouched quote");
    assert_eq!(
        untouched.get::<String, _>("fetched_at"),
        "2026-07-22T16:00:00Z"
    );
    assert_eq!(
        untouched.get::<String, _>("created_at"),
        "2026-07-22T16:00:01Z"
    );
}
