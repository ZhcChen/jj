use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use fund_monitor::{
    app::config::AppConfig,
    app_router, build_state,
    domain::{fund::NewFund, monitor_rule::NewMonitorRule},
    storage::{fund_repo::FundRepo, rule_repo::RuleRepo},
};
use http_body_util::BodyExt;
use tempfile::tempdir;
use tower::util::ServiceExt;

#[tokio::test]
async fn rules_page_shows_empty_state_when_no_rules() {
    let (app, state) = test_app_with_state().await;
    seed_fund(&state, "000001", "示例基金", Some("成长")).await;

    let html = get_html(&app, "/rules").await;
    assert!(html.contains("当前还没有任何监控规则"));
}

#[tokio::test]
async fn create_rule_shows_up_in_rules_list() {
    let (app, state) = test_app_with_state().await;
    seed_fund(&state, "000001", "示例基金", Some("成长")).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rules")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(
                    "target_scope=fund&fund_id=1&rule_type=change_rate_threshold&cooldown_minutes=30&change_rate_gte=1.5",
                ))
                .expect("request"),
        )
        .await
        .expect("create response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/rules?updated=created")
    );

    let html = get_html(&app, "/rules?updated=created").await;
    assert!(html.contains("已新增监控规则"));
    assert!(html.contains("涨跌幅阈值"));
    assert!(html.contains("示例基金 (000001)"));
    assert!(html.contains("gte=1.5"));
    assert!(html.contains("已启用"));
}

#[tokio::test]
async fn invalid_rule_form_returns_error_without_writing_rule() {
    let (app, state) = test_app_with_state().await;
    seed_fund(&state, "000001", "示例基金", Some("成长")).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rules")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(
                    "target_scope=fund&fund_id=1&rule_type=change_rate_threshold&cooldown_minutes=30",
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.expect("collect body");
    let html = String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body");
    assert!(html.contains("当前规则至少需要填写一个阈值条件"));

    let rules = RuleRepo::new(state.pool.clone())
        .list_all()
        .await
        .expect("list rules");
    assert!(rules.is_empty());
}

#[tokio::test]
async fn toggle_rule_updates_status_on_page_and_storage() {
    let (app, state) = test_app_with_state().await;
    seed_fund(&state, "000001", "示例基金", Some("成长")).await;
    seed_rule(&state, 1).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rules/1/toggle")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("enabled=false"))
                .expect("request"),
        )
        .await
        .expect("toggle response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let html = get_html(&app, "/rules?updated=disabled").await;
    assert!(html.contains("已停用该规则"));
    assert!(html.contains("已停用"));

    let rule = RuleRepo::new(state.pool.clone())
        .find_by_id(1)
        .await
        .expect("find rule")
        .expect("rule exists");
    assert!(!rule.enabled);
}

#[tokio::test]
async fn delete_rule_removes_it_from_list() {
    let (app, state) = test_app_with_state().await;
    seed_fund(&state, "000001", "示例基金", Some("成长")).await;
    seed_rule(&state, 1).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rules/1/delete")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("delete response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let html = get_html(&app, "/rules?updated=deleted").await;
    assert!(html.contains("已删除该规则"));
    assert!(html.contains("当前还没有任何监控规则"));
}

async fn test_app_with_state() -> (Router, fund_monitor::app::state::AppState) {
    let config = test_config();
    let state = build_state(config).await.expect("build state");
    let app = app_router(state.clone());
    (app, state)
}

fn test_config() -> AppConfig {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("rules-web.db");
    let database_url = format!("sqlite://{}", db_path.display());

    AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 300,
        telegram_api_base_url: "https://api.telegram.org".to_owned(),
        telegram_bot_token: None,
        telegram_chat_id: None,
    }
}

async fn seed_fund(
    state: &fund_monitor::app::state::AppState,
    code: &str,
    name: &str,
    group_name: Option<&str>,
) {
    FundRepo::new(state.pool.clone())
        .create(NewFund {
            code: code.to_owned(),
            name: name.to_owned(),
            note: None,
            group_name: group_name.map(str::to_owned),
            tags: None,
            enabled: true,
        })
        .await
        .expect("create fund");
}

async fn seed_rule(state: &fund_monitor::app::state::AppState, fund_id: i64) {
    RuleRepo::new(state.pool.clone())
        .create(NewMonitorRule {
            fund_id: Some(fund_id),
            group_name: None,
            rule_type: "change_rate_threshold".to_owned(),
            threshold_config: r#"{"gte":1.5}"#.to_owned(),
            enabled: true,
            cooldown_minutes: 30,
        })
        .await
        .expect("create rule");
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
