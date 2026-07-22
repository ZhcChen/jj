use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
};
use fund_monitor::{
    app::config::AppConfig,
    app_router, build_state, build_state_with_fund_source,
    providers::{fund_source::EastmoneyFundSource, http_client::HttpClient},
};
use http_body_util::BodyExt;
use tempfile::tempdir;
use tower::util::ServiceExt;

#[tokio::test]
async fn funds_page_shows_empty_state_when_no_data() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/funds")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.expect("collect body");
    let text = String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body");
    assert!(text.contains("暂无基金"));
}

#[tokio::test]
async fn create_fund_shows_up_in_list() {
    let app = test_app().await;

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/funds")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(
                    "code=000001&name=%E7%A4%BA%E4%BE%8B%E5%9F%BA%E9%87%91",
                ))
                .expect("request"),
        )
        .await
        .expect("create response");

    assert_eq!(create_response.status(), StatusCode::SEE_OTHER);

    let list_response = app
        .oneshot(
            Request::builder()
                .uri("/funds")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list response");
    let body = list_response
        .into_body()
        .collect()
        .await
        .expect("collect body");
    let text = String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body");
    assert!(text.contains("000001"));
    assert!(text.contains("示例基金"));
}

#[tokio::test]
async fn duplicate_code_returns_validation_error() {
    let app = test_app().await;

    let payload = "code=000001&name=%E7%A4%BA%E4%BE%8B%E5%9F%BA%E9%87%91";

    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/funds")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(payload))
                    .expect("request"),
            )
            .await
            .expect("response");

        if response.status() == StatusCode::BAD_REQUEST {
            let body = response.into_body().collect().await.expect("collect body");
            let text = String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body");
            assert!(text.contains("基金代码已存在"));
            return;
        }
    }

    panic!("second create should fail with duplicate code");
}

#[tokio::test]
async fn update_fund_metadata_is_visible_in_list_and_detail() {
    let app = test_app().await;
    create_fund(&app, "000001", "示例基金").await;

    let detail_before = get_html(&app, "/funds").await;
    let detail_link = extract_first_detail_path(&detail_before).expect("detail path");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&detail_link)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(
                    "name=%E7%A4%BA%E4%BE%8B%E5%9F%BA%E9%87%91A&group_name=%E8%A7%82%E5%AF%9F&tags=%E6%8C%87%E6%95%B0&note=%E6%9B%B4%E6%96%B0%E5%A4%87%E6%B3%A8",
                ))
                .expect("request"),
        )
        .await
        .expect("update response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let list_html = get_html(&app, "/funds").await;
    assert!(list_html.contains("示例基金A"));
    assert!(list_html.contains("观察"));
    assert!(list_html.contains("更新备注"));

    let detail_html = get_html(&app, &detail_link).await;
    assert!(detail_html.contains("示例基金A"));
    assert!(detail_html.contains("观察"));
    assert!(detail_html.contains("更新备注"));
}

#[tokio::test]
async fn disable_fund_removes_it_from_active_list() {
    let app = test_app().await;
    create_fund(&app, "000001", "示例基金").await;

    let list_html = get_html(&app, "/funds").await;
    let detail_path = extract_first_detail_path(&list_html).expect("detail path");
    let disable_path = format!("{detail_path}/disable");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&disable_path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("disable response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let html = get_html(&app, "/funds").await;
    assert!(!html.contains("href=\"/funds/1\""));
    assert!(html.contains("暂无基金"));
}

#[tokio::test]
async fn invalid_form_returns_error_without_writing_data() {
    let app = test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/funds")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("code=&name="))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.expect("collect body");
    let text = String::from_utf8(body.to_bytes().to_vec()).expect("utf8 body");
    assert!(text.contains("基金代码和名称不能为空"));

    let html = get_html(&app, "/funds").await;
    assert!(html.contains("暂无基金"));
}

#[tokio::test]
async fn manual_fetch_route_shows_latest_quote_in_detail_page() {
    let server = spawn_fixture_server(valid_fixture("000001", "示例基金", 1.2345, 0.88)).await;
    let app = test_app_with_source(&server.base_url).await;

    create_fund(&app, "000001", "示例基金").await;

    let list_html = get_html(&app, "/funds").await;
    let detail_path = extract_first_detail_path(&list_html).expect("detail path");
    let fetch_path = format!("{detail_path}/fetch");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&fetch_path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("fetch response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/funds/1?fetched=1")
    );

    let detail_html = get_html(&app, "/funds/1?fetched=1").await;
    assert!(detail_html.contains("已完成最近一次基金数据抓取"));
    assert!(detail_html.contains("1.2345"));
    assert!(detail_html.contains("0.88%"));
    assert!(detail_html.contains("eastmoney/pingzhongdata"));
}

async fn test_app() -> Router {
    let config = test_config();
    let state = build_state(config).await.expect("build state");
    app_router(state)
}

async fn test_app_with_source(base_url: &str) -> Router {
    let config = test_config();
    let source = EastmoneyFundSource::new(HttpClient::new(base_url).expect("http client"));
    let state = build_state_with_fund_source(config, source)
        .await
        .expect("build state");
    app_router(state)
}

fn test_config() -> AppConfig {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("fund-web.db");
    let database_url = format!("sqlite://{}", db_path.display());

    AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 300,
    }
}

async fn create_fund(app: &Router, code: &str, name: &str) {
    let payload = format!("code={code}&name={name}");
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/funds")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(payload))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
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

fn extract_first_detail_path(html: &str) -> Option<String> {
    let marker = "href=\"/funds/";
    let start = html.find(marker)?;
    let rest = &html[start + 6..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
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

fn valid_fixture(code: &str, name: &str, unit_nav: f64, change_rate: f64) -> String {
    format!(
        r#"var fS_name = "{name}";var fS_code = "{code}";var Data_netWorthTrend = [{{"x":1721606400000,"y":1.1111,"equityReturn":0.11}},{{"x":1721692800000,"y":{unit_nav},"equityReturn":{change_rate}}}];"#
    )
}
