use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use fund_monitor::{
    app::config::AppConfig,
    app_router, build_state_with_fund_source,
    jobs::poll_funds::PollFundsJob,
    notifications::telegram::TelegramNotifier,
    providers::{fund_source::EastmoneyFundSource, http_client::HttpClient},
    storage::{alert_repo::AlertRepo, fund_repo::FundRepo, rule_repo::RuleRepo},
};
use http_body_util::BodyExt;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tower::util::ServiceExt;

const TELEGRAM_BOT_TOKEN: &str = "TEST_TOKEN";
const TELEGRAM_CHAT_ID: &str = "123456";

#[tokio::test]
async fn alerts_page_shows_generated_alert() {
    let server = spawn_fixture_server(TelegramBehavior::Success).await;
    let state = test_state_with_server(&server.base_url, false).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A").await;
    create_rule(
        &state.pool,
        fund.id,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let app = app_router(state);
    let html = get_html(&app, "/alerts").await;
    assert!(html.contains("示例基金A (000001)"));
    assert!(html.contains("涨跌幅 1.25%"));
    assert!(html.contains("新告警"));
}

#[tokio::test]
async fn alert_status_update_is_visible_on_alerts_page() {
    let server = spawn_fixture_server(TelegramBehavior::Success).await;
    let state = test_state_with_server(&server.base_url, false).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A").await;
    create_rule(
        &state.pool,
        fund.id,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
    )
    .await;

    PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    let app = app_router(state);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/alerts/1/status")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("status=processed"))
                .expect("request"),
        )
        .await
        .expect("status response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/alerts?updated=processed")
    );

    let html = get_html(&app, "/alerts?updated=processed").await;
    assert!(html.contains("已将告警标记为已处理"));
    assert!(html.contains("已处理"));
}

#[test]
fn telegram_config_partial_missing_returns_clear_error() {
    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url: "sqlite://data/test.db".to_owned(),
        poll_interval_seconds: 300,
        telegram_api_base_url: "https://api.telegram.org".to_owned(),
        telegram_bot_token: Some(TELEGRAM_BOT_TOKEN.to_owned()),
        telegram_chat_id: None,
    };

    let err = match TelegramNotifier::from_app_config(&config) {
        Ok(_) => panic!("partial telegram config should fail"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("FUND_MONITOR_TELEGRAM_BOT_TOKEN"));
    assert!(err.to_string().contains("FUND_MONITOR_TELEGRAM_CHAT_ID"));
}

#[tokio::test]
async fn telegram_success_is_recorded_on_alert() {
    let server = spawn_fixture_server(TelegramBehavior::Success).await;
    let state = test_state_with_server(&server.base_url, true).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A").await;
    create_rule(
        &state.pool,
        fund.id,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
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
    assert_eq!(
        alerts[0].notification_result.as_deref(),
        Some("telegram 发送成功，message_id=1001")
    );

    let requests = server
        .telegram_requests
        .lock()
        .expect("telegram requests lock")
        .clone();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].contains("chat_id=123456"));
    assert!(requests[0].contains("text="));
}

#[tokio::test]
async fn telegram_request_failure_keeps_alert_and_tracks_error() {
    let server = spawn_fixture_server(TelegramBehavior::HttpFailure).await;
    let state = test_state_with_server(&server.base_url, true).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A").await;
    create_rule(
        &state.pool,
        fund.id,
        "change_rate_threshold",
        r#"{"gte":1.0}"#,
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
    assert!(
        alerts[0]
            .notification_result
            .as_deref()
            .expect("notification result")
            .contains("telegram 发送失败")
    );

    let app = app_router(state);
    let html = get_html(&app, "/alerts").await;
    assert!(html.contains("telegram 发送失败"));
}

async fn create_fund(
    pool: &sqlx::SqlitePool,
    code: &str,
    name: &str,
) -> fund_monitor::domain::fund::Fund {
    FundRepo::new(pool.clone())
        .create(fund_monitor::domain::fund::NewFund {
            code: code.to_owned(),
            name: name.to_owned(),
            note: None,
            group_name: None,
            tags: None,
            enabled: true,
        })
        .await
        .expect("create fund")
}

async fn create_rule(
    pool: &sqlx::SqlitePool,
    fund_id: i64,
    rule_type: &str,
    threshold_config: &str,
) -> fund_monitor::domain::monitor_rule::MonitorRule {
    RuleRepo::new(pool.clone())
        .create(fund_monitor::domain::monitor_rule::NewMonitorRule {
            fund_id: Some(fund_id),
            group_name: None,
            rule_type: rule_type.to_owned(),
            threshold_config: threshold_config.to_owned(),
            enabled: true,
            cooldown_minutes: 30,
        })
        .await
        .expect("create rule")
}

async fn test_state_with_server(
    base_url: &str,
    enable_telegram: bool,
) -> fund_monitor::app::state::AppState {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("telegram-notifications.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 1,
        telegram_api_base_url: base_url.to_owned(),
        telegram_bot_token: enable_telegram.then(|| TELEGRAM_BOT_TOKEN.to_owned()),
        telegram_chat_id: enable_telegram.then(|| TELEGRAM_CHAT_ID.to_owned()),
    };

    let source = EastmoneyFundSource::new(HttpClient::new(base_url).expect("http client"));
    build_state_with_fund_source(config, source)
        .await
        .expect("build state")
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

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.expect("collect body");
    String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body")
}

#[derive(Clone)]
struct FixtureState {
    eastmoney_body: String,
    telegram_behavior: TelegramBehavior,
    telegram_requests: Arc<Mutex<Vec<String>>>,
}

struct FixtureServer {
    base_url: String,
    handle: tokio::task::JoinHandle<()>,
    telegram_requests: Arc<Mutex<Vec<String>>>,
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[derive(Debug, Clone)]
enum TelegramBehavior {
    Success,
    HttpFailure,
}

async fn spawn_fixture_server(telegram_behavior: TelegramBehavior) -> FixtureServer {
    let telegram_requests = Arc::new(Mutex::new(Vec::new()));
    let state = FixtureState {
        eastmoney_body: fixture_with_estimated("000001", "示例基金A", 1.2000, 1.2300, 1.25),
        telegram_behavior,
        telegram_requests: telegram_requests.clone(),
    };

    let app = Router::new()
        .route("/pingzhongdata/000001.js", get(eastmoney_handler))
        .route(
            &format!("/bot{TELEGRAM_BOT_TOKEN}/sendMessage"),
            post(telegram_handler),
        )
        .with_state(state);

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
        telegram_requests,
    }
}

async fn eastmoney_handler(State(state): State<FixtureState>) -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        state.eastmoney_body,
    )
}

async fn telegram_handler(State(state): State<FixtureState>, body: String) -> impl IntoResponse {
    state
        .telegram_requests
        .lock()
        .expect("telegram requests lock")
        .push(body);

    match state.telegram_behavior {
        TelegramBehavior::Success => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"ok":true,"result":{"message_id":1001}}"#,
        )
            .into_response(),
        TelegramBehavior::HttpFailure => (
            StatusCode::BAD_GATEWAY,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"ok":false,"description":"mock telegram failure"}"#,
        )
            .into_response(),
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
