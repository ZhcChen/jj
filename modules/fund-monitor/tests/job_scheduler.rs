use axum::{Router, http::header, routing::get};
use fund_monitor::{
    app::config::AppConfig,
    build_state_with_fund_source,
    jobs::{poll_funds::PollFundsJob, scheduler::Scheduler},
    providers::fund_source::EastmoneyFundSource,
    providers::http_client::HttpClient,
    storage::{fund_repo::FundRepo, job_repo::JobRepo, quote_repo::QuoteRepo},
};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn scheduler_triggers_polling_by_interval() {
    let server = spawn_fixture_server(
        Some(valid_fixture("000001", "示例基金A", 1.2001, 0.12)),
        None,
    )
    .await;
    let state = test_state_with_base_url(&server.base_url, 1).await;
    let fund = create_fund(&state.pool, "000001", "示例基金A").await;

    let scheduler = Scheduler::new(state.clone());
    let handle = tokio::spawn(async move {
        scheduler
            .run_until(async {
                tokio::time::sleep(Duration::from_millis(1200)).await;
            })
            .await;
    });
    handle.await.expect("scheduler task");

    let jobs = JobRepo::new(state.pool.clone())
        .list_recent(20)
        .await
        .expect("list jobs");
    assert!(jobs.iter().any(|job| job.job_type == "poll_funds"));
    assert!(
        jobs.iter()
            .any(|job| job.job_type == "fund_poll_fetch:000001")
    );

    let latest_quote = QuoteRepo::new(state.pool.clone())
        .latest_for_fund(fund.id)
        .await
        .expect("latest quote");
    assert!(latest_quote.is_some());
}

#[tokio::test]
async fn poll_job_records_skipped_run_when_no_active_funds() {
    let state = test_state_with_base_url(&closed_base_url(), 1).await;

    let summary = PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    assert_eq!(summary.total_funds, 0);
    assert_eq!(summary.succeeded_funds, 0);
    assert_eq!(summary.failed_funds, 0);
    assert_eq!(summary.status, "skipped");

    let jobs = JobRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list jobs");
    let round_job = jobs
        .into_iter()
        .find(|job| job.job_type == "poll_funds")
        .expect("round job");
    assert_eq!(round_job.status, "skipped");
    assert_eq!(
        round_job.error_message.as_deref(),
        Some("当前没有启用基金，跳过抓取")
    );
    assert!(round_job.finished_at.is_some());
}

#[tokio::test]
async fn poll_job_continues_when_single_fund_fails() {
    let server = spawn_fixture_server(
        Some(valid_fixture("000001", "示例基金A", 1.2001, 0.12)),
        None,
    )
    .await;
    let state = test_state_with_base_url(&server.base_url, 1).await;
    let ok_fund = create_fund(&state.pool, "000001", "示例基金A").await;
    let failed_fund = create_fund(&state.pool, "000002", "示例基金B").await;

    let summary = PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    assert_eq!(summary.total_funds, 2);
    assert_eq!(summary.succeeded_funds, 1);
    assert_eq!(summary.failed_funds, 1);
    assert_eq!(summary.status, "partial_success");

    let quote_repo = QuoteRepo::new(state.pool.clone());
    assert!(
        quote_repo
            .latest_for_fund(ok_fund.id)
            .await
            .expect("ok quote")
            .is_some()
    );
    assert!(
        quote_repo
            .latest_for_fund(failed_fund.id)
            .await
            .expect("failed quote")
            .is_none()
    );

    let jobs = JobRepo::new(state.pool.clone())
        .list_recent(20)
        .await
        .expect("list jobs");
    let round_job = jobs
        .iter()
        .find(|job| job.job_type == "poll_funds")
        .expect("round job");
    assert_eq!(round_job.status, "partial_success");
    assert!(
        round_job
            .error_message
            .as_deref()
            .expect("round message")
            .contains("000002")
    );
}

#[tokio::test]
async fn poll_job_round_record_has_started_and_finished_times() {
    let server = spawn_fixture_server(
        Some(valid_fixture("000001", "示例基金A", 1.2001, 0.12)),
        None,
    )
    .await;
    let state = test_state_with_base_url(&server.base_url, 1).await;
    create_fund(&state.pool, "000001", "示例基金A").await;

    let summary = PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");
    assert_eq!(summary.status, "success");

    let jobs = JobRepo::new(state.pool.clone())
        .list_recent(10)
        .await
        .expect("list jobs");
    let round_job = jobs
        .into_iter()
        .find(|job| job.job_type == "poll_funds")
        .expect("round job");
    assert_eq!(round_job.status, "success");
    assert!(round_job.finished_at.is_some());
}

#[tokio::test]
async fn disabled_fund_is_not_selected_for_polling() {
    let server = spawn_fixture_server(
        Some(valid_fixture("000001", "示例基金A", 1.2001, 0.12)),
        None,
    )
    .await;
    let state = test_state_with_base_url(&server.base_url, 1).await;
    let enabled_fund = create_fund(&state.pool, "000001", "示例基金A").await;
    let disabled_fund = create_fund(&state.pool, "000002", "示例基金B").await;
    FundRepo::new(state.pool.clone())
        .disable(disabled_fund.id)
        .await
        .expect("disable fund");

    let summary = PollFundsJob::new(state.clone())
        .run_once()
        .await
        .expect("run poll job");

    assert_eq!(summary.total_funds, 1);
    assert_eq!(summary.succeeded_funds, 1);
    assert_eq!(summary.failed_funds, 0);
    assert_eq!(summary.status, "success");

    let quote_repo = QuoteRepo::new(state.pool.clone());
    assert!(
        quote_repo
            .latest_for_fund(enabled_fund.id)
            .await
            .expect("enabled quote")
            .is_some()
    );
    assert!(
        quote_repo
            .latest_for_fund(disabled_fund.id)
            .await
            .expect("disabled quote")
            .is_none()
    );

    let jobs = JobRepo::new(state.pool.clone())
        .list_recent(20)
        .await
        .expect("list jobs");
    assert!(
        jobs.iter()
            .any(|job| job.job_type == "fund_poll_fetch:000001")
    );
    assert!(
        !jobs
            .iter()
            .any(|job| job.job_type == "fund_poll_fetch:000002")
    );
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

async fn test_state_with_base_url(
    base_url: &str,
    poll_interval_seconds: u64,
) -> fund_monitor::app::state::AppState {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    let db_path = root.join("job-scheduler.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds,
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

async fn spawn_fixture_server(
    body_000001: Option<String>,
    body_000002: Option<String>,
) -> FixtureServer {
    let mut app = Router::new();

    if let Some(body) = body_000001 {
        app = app.route(
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
    }

    if let Some(body) = body_000002 {
        app = app.route(
            "/pingzhongdata/000002.js",
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
