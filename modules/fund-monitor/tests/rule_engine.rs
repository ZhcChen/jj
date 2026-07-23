use axum::{Router, http::header, routing::get};
use fund_monitor::{
    app::config::AppConfig,
    build_state_with_fund_source,
    jobs::poll_funds::PollFundsJob,
    providers::{fund_source::EastmoneyFundSource, http_client::HttpClient},
    storage::{
        alert_repo::AlertRepo, fund_repo::FundRepo, quote_repo::QuoteRepo, rule_repo::RuleRepo,
    },
};
use tempfile::tempdir;

#[tokio::test]
async fn change_rate_threshold_hit_creates_new_alert() {
    let server = spawn_fixture_server(fixture_with_estimated(
        "000001",
        "示例基金A",
        1.2000,
        1.2300,
        1.25,
    ))
    .await;
    let state = test_state_with_base_url(&server.base_url).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A", Some("成长")).await;
    let rule = create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
        true,
        30,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let alerts = AlertRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list alerts");
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].rule_id, rule.id);
    assert_eq!(alerts[0].fund_id, fund.id);
    assert!(alerts[0].reason.contains("涨跌幅 1.25%"));

    let stored_rule = RuleRepo::new(state.pool.clone())
        .find_by_id(rule.id)
        .await
        .expect("find rule")
        .expect("rule exists");
    assert!(stored_rule.last_triggered_at.is_some());
}

#[tokio::test]
async fn change_rate_threshold_not_hit_does_not_create_alert() {
    let server = spawn_fixture_server(fixture_with_estimated(
        "000001",
        "示例基金A",
        1.2000,
        1.2100,
        0.20,
    ))
    .await;
    let state = test_state_with_base_url(&server.base_url).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A", Some("成长")).await;
    create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
        true,
        30,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let alerts = AlertRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list alerts");
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn nav_range_rule_creates_alert_when_unit_nav_in_range() {
    let server = spawn_fixture_server(fixture_with_estimated(
        "000001",
        "示例基金A",
        1.2360,
        1.2400,
        0.32,
    ))
    .await;
    let state = test_state_with_base_url(&server.base_url).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A", Some("平衡")).await;
    create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "nav_range",
        r#"{"min":1.2300,"max":1.2400}"#,
        true,
        30,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let alerts = AlertRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list alerts");
    assert_eq!(alerts.len(), 1);
    assert!(alerts[0].reason.contains("单位净值 1.2360"));
}

#[tokio::test]
async fn cooldown_suppresses_duplicate_alerts() {
    let server = spawn_fixture_server(fixture_with_estimated(
        "000001",
        "示例基金A",
        1.2000,
        1.2400,
        1.50,
    ))
    .await;
    let state = test_state_with_base_url(&server.base_url).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A", Some("成长")).await;
    create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
        true,
        30,
    )
    .await;

    let job = PollFundsJob::new(state.clone());
    job.run_once().await.expect("first run");
    job.run_once().await.expect("second run");

    let alerts = AlertRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list alerts");
    assert_eq!(alerts.len(), 1);
}

#[tokio::test]
async fn disabled_rule_is_not_executed() {
    let server = spawn_fixture_server(fixture_with_estimated(
        "000001",
        "示例基金A",
        1.2000,
        1.2400,
        1.50,
    ))
    .await;
    let state = test_state_with_base_url(&server.base_url).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A", Some("成长")).await;
    create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
        false,
        30,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let alerts = AlertRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list alerts");
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn multiple_rules_generate_independent_alerts_for_same_fund() {
    let server = spawn_fixture_server(fixture_with_estimated(
        "000001",
        "示例基金A",
        1.2000,
        1.2600,
        1.80,
    ))
    .await;
    let state = test_state_with_base_url(&server.base_url).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A", Some("成长")).await;
    create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
        true,
        30,
    )
    .await;
    create_rule(
        &state.pool,
        Some(fund.id),
        None,
        "estimated_nav_deviation",
        r#"{"abs_gte":3.0}"#,
        true,
        30,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let alerts = AlertRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list alerts");
    assert_eq!(alerts.len(), 2);
    assert!(alerts.iter().any(|alert| alert.reason.contains("涨跌幅")));
    assert!(
        alerts
            .iter()
            .any(|alert| alert.reason.contains("估值偏离 5.00%"))
    );

    let latest_quote = QuoteRepo::new(state.pool.clone())
        .latest_for_fund(fund.id)
        .await
        .expect("latest quote");
    assert!(latest_quote.is_some());
}

async fn create_fund(
    pool: &sqlx::SqlitePool,
    code: &str,
    name: &str,
    group_name: Option<&str>,
) -> fund_monitor::domain::fund::Fund {
    FundRepo::new(pool.clone())
        .create(fund_monitor::domain::fund::NewFund {
            code: code.to_owned(),
            name: name.to_owned(),
            note: None,
            group_name: group_name.map(str::to_owned),
            tags: None,
            enabled: true,
        })
        .await
        .expect("create fund")
}

async fn create_rule(
    pool: &sqlx::SqlitePool,
    fund_id: Option<i64>,
    group_name: Option<&str>,
    rule_type: &str,
    threshold_config: &str,
    enabled: bool,
    cooldown_minutes: i64,
) -> fund_monitor::domain::monitor_rule::MonitorRule {
    RuleRepo::new(pool.clone())
        .create(fund_monitor::domain::monitor_rule::NewMonitorRule {
            fund_id,
            group_name: group_name.map(str::to_owned),
            rule_type: rule_type.to_owned(),
            threshold_config: threshold_config.to_owned(),
            enabled,
            cooldown_minutes,
        })
        .await
        .expect("create rule")
}

async fn test_state_with_base_url(base_url: &str) -> fund_monitor::app::state::AppState {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("rule-engine.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        database_url,
        poll_interval_seconds: 1,
        telegram_api_base_url: "https://api.telegram.org".to_owned(),
        telegram_bot_token: None,
        telegram_chat_id: None,
    };

    let source = EastmoneyFundSource::new(HttpClient::new(base_url).expect("http client"));
    build_state_with_fund_source(config, source)
        .await
        .expect("build state")
}

struct FixtureServer {
    base_url: String,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn spawn_fixture_server(body: String) -> FixtureServer {
    let app = Router::new().route(
        "/pingzhongdata/000001.js",
        get({
            let body = body.clone();
            move || {
                let body = body.clone();
                async move {
                    (
                        [(
                            header::CONTENT_TYPE,
                            "application/javascript; charset=utf-8",
                        )],
                        body,
                    )
                }
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture server");
    let addr = listener.local_addr().expect("fixture server addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve fixture");
    });

    FixtureServer {
        base_url: format!("http://{addr}"),
        handle,
    }
}

fn fixture_with_estimated(
    code: &str,
    name: &str,
    unit_nav: f64,
    estimated_nav: f64,
    change_rate: f64,
) -> String {
    format!(
        r#"var fS_name = "{name}";var fS_code = "{code}";var gsz = "{estimated_nav:.4}";var gszzl = "{change_rate:.2}";var Data_netWorthTrend = [{{"x":1721606400000,"y":1.1111,"equityReturn":0.11}},{{"x":1721692800000,"y":{unit_nav:.4},"equityReturn":{change_rate:.2}}}];"#
    )
}
