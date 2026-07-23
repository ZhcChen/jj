use axum::{
    Router,
    http::header,
    response::IntoResponse,
    routing::{get, post},
};
use fund_monitor::{
    app::config::AppConfig,
    build_state_with_fund_source,
    jobs::poll_funds::PollFundsJob,
    providers::{fund_source::EastmoneyFundSource, http_client::HttpClient},
    storage::fund_repo::FundRepo,
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
use tempfile::tempdir;
use tracing_subscriber::fmt::MakeWriter;

const TELEGRAM_BOT_TOKEN: &str = "TEST_TOKEN";

#[tokio::test]
async fn logs_include_fetch_rule_and_notification_categories() {
    let server = spawn_fixture_server().await;
    let state = test_state_with_base_url(&server.base_url).await;

    create_fund(&state.pool, "000001", "通知失败基金").await;
    create_fund(&state.pool, "000002", "规则失败基金").await;
    create_fund(&state.pool, "000003", "抓取失败基金").await;

    let rule_repo = fund_monitor::storage::rule_repo::RuleRepo::new(state.pool.clone());
    rule_repo
        .create(fund_monitor::domain::monitor_rule::NewMonitorRule {
            fund_id: Some(1),
            group_name: None,
            rule_type: "change_rate_threshold".to_owned(),
            threshold_config: r#"{"gte":0.0}"#.to_owned(),
            enabled: true,
            cooldown_minutes: 30,
        })
        .await
        .expect("create notification rule");
    rule_repo
        .create(fund_monitor::domain::monitor_rule::NewMonitorRule {
            fund_id: Some(2),
            group_name: None,
            rule_type: "estimated_nav_deviation".to_owned(),
            threshold_config: r#"{"abs_gte":0.0}"#.to_owned(),
            enabled: true,
            cooldown_minutes: 30,
        })
        .await
        .expect("create rule failure rule");

    let writer = SharedWriter::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(true)
        .with_writer(writer.clone())
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);
    PollFundsJob::new(state)
        .run_once()
        .await
        .expect("run poll job");

    let logs = writer.contents();
    assert!(logs.contains("category=\"fetch\""), "{logs}");
    assert!(logs.contains("category=\"rule\""), "{logs}");
    assert!(logs.contains("category=\"notification\""), "{logs}");
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
    let db_path = root.join("logging.db");
    let database_url = format!("sqlite://{}", db_path.display());

    let config = AppConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url,
        poll_interval_seconds: 60,
        telegram_api_base_url: base_url.to_owned(),
        telegram_bot_token: Some(TELEGRAM_BOT_TOKEN.to_owned()),
        telegram_chat_id: Some("123456".to_owned()),
    };

    let source = EastmoneyFundSource::new(HttpClient::new(base_url).expect("http client"));
    build_state_with_fund_source(config, source)
        .await
        .expect("build state")
}

#[derive(Clone, Default)]
struct SharedWriter {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl SharedWriter {
    fn contents(&self) -> String {
        String::from_utf8(self.inner.lock().expect("writer lock").clone()).expect("utf8 logs")
    }
}

impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        SharedWriterGuard {
            inner: self.inner.clone(),
        }
    }
}

struct SharedWriterGuard {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedWriterGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner
            .lock()
            .expect("writer lock")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
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

async fn spawn_fixture_server() -> FixtureServer {
    let app = Router::new()
        .route(
            "/pingzhongdata/000001.js",
            get(|| async move {
                (
                    [(
                        header::CONTENT_TYPE,
                        "application/javascript; charset=utf-8",
                    )],
                    fixture_with_estimated("000001", "通知失败基金", 1.2000, 1.2300, 1.25),
                )
            }),
        )
        .route(
            "/pingzhongdata/000002.js",
            get(|| async move {
                (
                    [(
                        header::CONTENT_TYPE,
                        "application/javascript; charset=utf-8",
                    )],
                    fixture_without_estimated("000002", "规则失败基金", 1.2000, 1.25),
                )
            }),
        )
        .route(
            &format!("/bot{TELEGRAM_BOT_TOKEN}/sendMessage"),
            post(|| async move {
                (
                    axum::http::StatusCode::BAD_GATEWAY,
                    [(header::CONTENT_TYPE, "application/json")],
                    r#"{"ok":false,"description":"mock telegram failure"}"#,
                )
                    .into_response()
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

fn fixture_without_estimated(code: &str, name: &str, unit_nav: f64, change_rate: f64) -> String {
    format!(
        r#"var fS_name = "{name}";var fS_code = "{code}";var gszzl = "{change_rate:.2}";var Data_netWorthTrend = [{{"x":1721606400000,"y":1.1111,"equityReturn":0.11}},{{"x":1721692800000,"y":{unit_nav:.4},"equityReturn":{change_rate:.2}}}];"#
    )
}
