use axum::{Router, body::Body, http::Request};
use fund_monitor::{
    app::config::AppConfig,
    app_router, build_state_with_fund_source,
    domain::{
        alert_event::NewAlertEvent, fund::NewFund, fund_quote::NewFundQuote,
        monitor_rule::NewMonitorRule,
    },
    providers::{fund_source::EastmoneyFundSource, http_client::HttpClient},
    storage::{
        alert_repo::AlertRepo, fund_repo::FundRepo, job_repo::JobRepo, quote_repo::QuoteRepo,
        rule_repo::RuleRepo,
    },
};
use http_body_util::BodyExt;
use tempfile::tempdir;
use time::{Duration, OffsetDateTime};
use tower::util::ServiceExt;

#[tokio::test]
async fn dashboard_page_shows_overview_and_latest_alert_summary() {
    let app = seeded_app(true).await;

    let html = get_html(&app, "/dashboard").await;
    assert!(html.contains("总览看板"));
    assert!(html.contains("启用基金数"));
    assert!(html.contains("最近轮询状态"));
    assert!(html.contains("示例基金A (000001)"));
    assert!(html.contains("涨跌幅超过阈值"));
    assert!(html.contains("poll_funds"));
}

#[tokio::test]
async fn fund_detail_page_shows_history_descending_and_empty_state() {
    let app_with_history = seeded_app(true).await;
    let seeded_html = get_html(&app_with_history, "/funds/1").await;
    let latest_pos = seeded_html.find("1.3000").expect("latest nav");
    let older_pos = seeded_html.find("1.1000").expect("older nav");
    assert!(latest_pos < older_pos);

    let empty_app = seeded_app(false).await;
    let empty_html = get_html(&empty_app, "/funds/1").await;
    assert!(empty_html.contains("还没有历史抓取记录"));
}

#[tokio::test]
async fn settings_page_shows_runtime_and_notification_config() {
    let app = seeded_app(true).await;

    let html = get_html(&app, "/settings").await;
    assert!(html.contains("系统配置"));
    assert!(html.contains("轮询频率"));
    assert!(html.contains("eastmoney/pingzhongdata"));
    assert!(html.contains("已启用"));
    assert!(html.contains("123456"));
}

#[tokio::test]
async fn dashboard_returns_unified_error_response_when_storage_is_unavailable() {
    let state = seeded_state(true).await;
    let app = app_router(state.clone());
    state.pool.close().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
    let body = response.into_body().collect().await.expect("collect body");
    let html = String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body");
    assert!(html.contains("页面暂时不可用"));
}

async fn seeded_app(with_quote_history: bool) -> Router {
    let state = seeded_state(with_quote_history).await;
    app_router(state)
}

async fn seeded_state(with_quote_history: bool) -> fund_monitor::app::state::AppState {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("dashboard-routes.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 300,
        telegram_api_base_url: "https://api.telegram.org".to_owned(),
        telegram_bot_token: Some("TEST_TOKEN".to_owned()),
        telegram_chat_id: Some("123456".to_owned()),
    };

    let source =
        EastmoneyFundSource::new(HttpClient::new("http://127.0.0.1:65535").expect("http client"));
    let state = build_state_with_fund_source(config, source)
        .await
        .expect("build state");

    seed_state(&state, with_quote_history).await;
    state
}

async fn seed_state(state: &fund_monitor::app::state::AppState, with_quote_history: bool) {
    let fund_repo = FundRepo::new(state.pool.clone());
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let rule_repo = RuleRepo::new(state.pool.clone());
    let alert_repo = AlertRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    let fund = fund_repo
        .create(NewFund {
            code: "000001".to_owned(),
            name: "示例基金A".to_owned(),
            note: Some("核心持仓".to_owned()),
            group_name: Some("成长".to_owned()),
            tags: Some("主动".to_owned()),
            enabled: true,
        })
        .await
        .expect("create fund");

    if with_quote_history {
        let now = OffsetDateTime::now_utc();
        quote_repo
            .insert(NewFundQuote {
                fund_id: fund.id,
                unit_nav: Some(1.1000),
                nav_date: None,
                confirmed_change_rate: Some(0.80),
                estimated_nav: Some(1.1200),
                estimated_change_rate: Some(0.80),
                estimated_at: None,
                change_rate: Some(0.80),
                fetched_at: now - Duration::hours(2),
                source: "mock-source".to_owned(),
            })
            .await
            .expect("insert older quote");
        quote_repo
            .insert(NewFundQuote {
                fund_id: fund.id,
                unit_nav: Some(1.3000),
                nav_date: None,
                confirmed_change_rate: Some(1.20),
                estimated_nav: Some(1.3100),
                estimated_change_rate: Some(1.20),
                estimated_at: None,
                change_rate: Some(1.20),
                fetched_at: now - Duration::hours(1),
                source: "mock-source".to_owned(),
            })
            .await
            .expect("insert latest quote");
    }

    let rule = rule_repo
        .create(NewMonitorRule {
            fund_id: Some(fund.id),
            group_name: None,
            rule_type: "change_rate_threshold".to_owned(),
            threshold_config: r#"{"gte":1.0}"#.to_owned(),
            enabled: true,
            cooldown_minutes: 30,
        })
        .await
        .expect("create rule");

    alert_repo
        .create(NewAlertEvent {
            rule_id: rule.id,
            fund_id: fund.id,
            reason: "涨跌幅超过阈值".to_owned(),
            status: "new".to_owned(),
            triggered_at: OffsetDateTime::now_utc(),
            notification_result: Some("telegram 发送成功，message_id=42".to_owned()),
        })
        .await
        .expect("create alert");

    let poll_job = job_repo.start("poll_funds").await.expect("start poll job");
    job_repo
        .finish(poll_job.id, "success", None)
        .await
        .expect("finish poll job");
}

async fn get_html(app: &Router, path: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = response.into_body().collect().await.expect("collect body");
    String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body")
}
