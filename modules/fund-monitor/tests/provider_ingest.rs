use axum::{
    Router,
    http::header,
    routing::{get, post},
};
use fund_monitor::{
    app::config::AppConfig,
    build_state_with_fund_source,
    providers::{
        fund_source::{EastmoneyFundSource, fetch_and_store_fund_quote},
        http_client::HttpClient,
    },
    storage::{fund_repo::FundRepo, job_repo::JobRepo, quote_repo::QuoteRepo},
};
use tempfile::tempdir;
use time::OffsetDateTime;

#[tokio::test]
async fn valid_provider_response_writes_quote_and_success_job() {
    let server = spawn_fixture_server(valid_fixture("000001", "示例基金", 1.2401, 0.45)).await;
    let state = test_state_with_base_url(&server.base_url).await;

    let fund = create_fund(&state.pool, "000001", "示例基金").await;
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());
    let fetch_started_at = OffsetDateTime::now_utc();

    let quote =
        fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
            .await
            .expect("ingest quote");
    let fetch_finished_at = OffsetDateTime::now_utc();

    assert_eq!(quote.unit_nav, Some(1.2401));
    assert!(quote.nav_date.is_some());
    assert_eq!(quote.confirmed_change_rate, Some(0.45));
    assert_eq!(quote.estimated_nav, None);
    assert_eq!(quote.estimated_change_rate, None);
    assert_eq!(quote.estimated_at, None);
    assert_eq!(quote.change_rate, Some(0.45));
    assert_eq!(quote.source, "eastmoney/pingzhongdata");
    assert!(quote.fetched_at >= fetch_started_at);
    assert!(quote.fetched_at <= fetch_finished_at);

    let latest = quote_repo
        .latest_for_fund(fund.id)
        .await
        .expect("latest quote")
        .expect("quote exists");
    assert_eq!(latest.id, quote.id);
    assert_eq!(latest.nav_date, quote.nav_date);
    assert_eq!(latest.confirmed_change_rate, quote.confirmed_change_rate);
    assert_eq!(latest.estimated_change_rate, quote.estimated_change_rate);
    assert_eq!(latest.fetched_at, quote.fetched_at);

    let jobs = job_repo.list_recent(5).await.expect("job list");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, "success");
    assert!(jobs[0].error_message.is_none());
}

#[tokio::test]
async fn missing_fields_are_rejected_and_failed_job_is_recorded() {
    let server = spawn_fixture_server(
        r#"var fS_code = "000001";var Data_netWorthTrend = [{"x":1721692800000,"y":1.2401,"equityReturn":0.45}];"#
            .to_owned(),
    )
    .await;
    let state = test_state_with_base_url(&server.base_url).await;

    let fund = create_fund(&state.pool, "000001", "示例基金").await;
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    let err = fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
        .await
        .expect_err("missing fields should fail");

    assert_eq!(err.user_message(), "基金数据源缺少必要字段：基金名称");
    assert!(
        quote_repo
            .latest_for_fund(fund.id)
            .await
            .expect("latest quote")
            .is_none()
    );

    let jobs = job_repo.list_recent(5).await.expect("job list");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, "failed");
    assert_eq!(
        jobs[0].error_message.as_deref(),
        Some("基金数据源缺少必要字段：基金名称")
    );
}

#[tokio::test]
async fn network_failure_records_failed_job_without_dirty_data() {
    let state = test_state_with_base_url(&closed_base_url()).await;

    let fund = create_fund(&state.pool, "000001", "示例基金").await;
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    let err = fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
        .await
        .expect_err("network failure should fail");

    assert!(err.user_message().contains("请求基金数据源失败"));
    assert!(
        quote_repo
            .latest_for_fund(fund.id)
            .await
            .expect("latest quote")
            .is_none()
    );

    let jobs = job_repo.list_recent(5).await.expect("job list");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, "failed");
    assert!(
        jobs[0]
            .error_message
            .as_deref()
            .expect("job error message")
            .contains("请求基金数据源失败")
    );
}

#[tokio::test]
async fn latest_quote_returns_newest_after_two_fetches() {
    let first_server =
        spawn_fixture_server(valid_fixture("000001", "示例基金", 1.2100, 0.10)).await;
    let second_server =
        spawn_fixture_server(valid_fixture("000001", "示例基金", 1.2600, 1.30)).await;
    let state = test_state_with_base_url(&first_server.base_url).await;

    let fund = create_fund(&state.pool, "000001", "示例基金").await;
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
        .await
        .expect("first ingest");

    let second_source =
        EastmoneyFundSource::new(HttpClient::new(&second_server.base_url).expect("http client"));
    fetch_and_store_fund_quote(&second_source, &fund, &quote_repo, &job_repo)
        .await
        .expect("second ingest");

    let latest = quote_repo
        .latest_for_fund(fund.id)
        .await
        .expect("latest quote")
        .expect("quote exists");
    assert_eq!(latest.unit_nav, Some(1.2600));
    assert_eq!(latest.confirmed_change_rate, Some(1.30));
    assert_eq!(latest.change_rate, Some(1.30));

    let history = quote_repo
        .list_recent_for_fund(fund.id, 10)
        .await
        .expect("quote history");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].unit_nav, Some(1.2600));
    assert_eq!(history[1].unit_nav, Some(1.2100));
}

#[tokio::test]
async fn estimated_snapshot_fields_are_kept_separate_from_confirmed_nav() {
    let server = spawn_fixture_server(estimated_fixture()).await;
    let state = test_state_with_base_url(&server.base_url).await;

    let fund = create_fund(&state.pool, "000001", "示例基金").await;
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    let quote =
        fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
            .await
            .expect("ingest estimated snapshot");

    assert_eq!(quote.unit_nav, Some(1.2401));
    assert_eq!(quote.confirmed_change_rate, Some(0.45));
    assert_eq!(quote.estimated_nav, Some(1.2523));
    assert_eq!(quote.estimated_change_rate, Some(1.13));
    assert_eq!(quote.change_rate, Some(1.13));
    assert!(quote.estimated_at.is_some());
}

#[tokio::test]
async fn valuation_endpoint_backfills_estimated_snapshot_when_pingzhongdata_has_none() {
    let server = spawn_fixture_server_with_valuation(
        valid_fixture("000001", "示例基金", 1.2401, 0.45),
        Some(valuation_fixture(1.2688, 2.31, "2026-07-23 14:38")),
    )
    .await;
    let state = test_state_with_base_url(&server.base_url).await;

    let fund = create_fund(&state.pool, "000001", "示例基金").await;
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    let quote =
        fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
            .await
            .expect("ingest estimated snapshot from valuation api");

    assert_eq!(quote.unit_nav, Some(1.2401));
    assert_eq!(quote.confirmed_change_rate, Some(0.45));
    assert_eq!(quote.estimated_nav, Some(1.2688));
    assert_eq!(quote.estimated_change_rate, Some(2.31));
    assert_eq!(quote.change_rate, Some(2.31));
    assert_eq!(quote.source, "eastmoney/pingzhongdata+eastmoney/fundcomapi");
    assert!(quote.estimated_at.is_some());
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

async fn test_state_with_base_url(base_url: &str) -> fund_monitor::app::state::AppState {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("provider-ingest.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 300,
        telegram_api_base_url: "https://api.telegram.org".to_owned(),
        telegram_bot_token: None,
        telegram_chat_id: None,
    };

    let source = EastmoneyFundSource::new(HttpClient::new(base_url).expect("http client"));
    build_state_with_fund_source(config, source)
        .await
        .expect("build state")
}

fn closed_base_url() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind local port");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);
    format!("http://{addr}")
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
    spawn_fixture_server_with_valuation(body, None).await
}

async fn spawn_fixture_server_with_valuation(
    body: String,
    valuation_body: Option<String>,
) -> FixtureServer {
    let mut app = Router::new().route(
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

    if let Some(valuation_body) = valuation_body {
        app = app.route(
            "/mm/newCore/FundValuationLast",
            post({
                let valuation_body = valuation_body.clone();
                move || {
                    let valuation_body = valuation_body.clone();
                    async move {
                        (
                            [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
                            valuation_body,
                        )
                    }
                }
            }),
        );
    }

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

fn estimated_fixture() -> String {
    r#"var fS_name = "示例基金";var fS_code = "000001";var gsz = "1.2523";var gszzl = "1.13";var gztime = "2026-07-22 14:36";var Data_netWorthTrend = [{"x":1721606400000,"y":1.1111,"equityReturn":0.11},{"x":1721692800000,"y":1.2401,"equityReturn":0.45}];"#.to_owned()
}

fn valuation_fixture(estimated_nav: f64, change_rate: f64, estimated_at: &str) -> String {
    format!(
        r#"{{"data":[{{"GSZ":{estimated_nav},"GSZZL":{change_rate},"GZTIME":"{estimated_at}"}}],"errorCode":0,"success":true}}"#
    )
}
