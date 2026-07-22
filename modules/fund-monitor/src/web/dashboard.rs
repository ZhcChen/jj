use crate::{
    app::{errors::render_internal_error, state::AppState},
    storage::{
        alert_repo::{AlertListItem, AlertRepo},
        fund_repo::FundRepo,
        job_repo::JobRepo,
    },
};
use askama::Template;
use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};
use time::{OffsetDateTime, UtcOffset};

use super::layout::render_html;

pub fn routes() -> Router<AppState> {
    Router::new().route("/dashboard", get(show_dashboard))
}

#[derive(Debug, Clone)]
struct DashboardMetric {
    label: &'static str,
    value: String,
    hint: String,
}

#[derive(Debug, Clone)]
struct DashboardJobView {
    job_type: String,
    status: String,
    status_label: String,
    started_at: String,
    finished_at: String,
    error_message: String,
}

#[derive(Debug, Clone)]
struct DashboardAlertView {
    fund_label: String,
    reason: String,
    status: String,
    status_label: String,
    triggered_at: String,
}

#[derive(Template)]
#[template(path = "dashboard/index.html")]
struct DashboardTemplate {
    title: &'static str,
    nav_key: &'static str,
    metrics: Vec<DashboardMetric>,
    recent_jobs: Vec<DashboardJobView>,
    recent_alerts: Vec<DashboardAlertView>,
}

async fn show_dashboard(State(state): State<AppState>) -> Response {
    match build_dashboard_template(&state).await {
        Ok(template) => match render_html(&template) {
            Ok(html) => html.into_response(),
            Err(_) => render_internal_error("仪表盘模板渲染失败，请稍后重试。"),
        },
        Err(message) => render_internal_error(message),
    }
}

async fn build_dashboard_template(state: &AppState) -> Result<DashboardTemplate, String> {
    let fund_repo = FundRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());
    let alert_repo = AlertRepo::new(state.pool.clone());

    let funds = fund_repo
        .list_active()
        .await
        .map_err(|err| format!("读取基金列表失败：{err:#}"))?;
    let recent_jobs = job_repo
        .list_recent(8)
        .await
        .map_err(|err| format!("读取任务执行记录失败：{err:#}"))?;
    let recent_alerts = alert_repo
        .list_recent_with_context(5)
        .await
        .map_err(|err| format!("读取最近告警失败：{err:#}"))?;

    let latest_poll_job = recent_jobs.iter().find(|job| job.job_type == "poll_funds");
    let latest_fetch_job = recent_jobs
        .iter()
        .find(|job| job.job_type.starts_with("fund_poll_fetch:"));

    let metrics = vec![
        DashboardMetric {
            label: "启用基金数",
            value: funds.len().to_string(),
            hint: "当前仍会参与轮询和监控的基金数量".to_owned(),
        },
        DashboardMetric {
            label: "最近轮询状态",
            value: latest_poll_job
                .map(|job| status_label(&job.status).to_owned())
                .unwrap_or_else(|| "暂无记录".to_owned()),
            hint: latest_poll_job
                .map(job_hint)
                .unwrap_or_else(|| "轮询任务尚未执行".to_owned()),
        },
        DashboardMetric {
            label: "最近抓取状态",
            value: latest_fetch_job
                .map(|job| status_label(&job.status).to_owned())
                .unwrap_or_else(|| "暂无记录".to_owned()),
            hint: latest_fetch_job
                .map(job_hint)
                .unwrap_or_else(|| "尚未生成基金抓取记录".to_owned()),
        },
        DashboardMetric {
            label: "最新告警摘要",
            value: recent_alerts
                .first()
                .map(|alert| alert.fund_code.clone())
                .unwrap_or_else(|| "暂无告警".to_owned()),
            hint: recent_alerts
                .first()
                .map(alert_hint)
                .unwrap_or_else(|| "还没有规则命中产生告警".to_owned()),
        },
    ];

    Ok(DashboardTemplate {
        title: "总览看板",
        nav_key: "dashboard",
        metrics,
        recent_jobs: recent_jobs.into_iter().map(map_job_view).collect(),
        recent_alerts: recent_alerts.into_iter().map(map_alert_view).collect(),
    })
}

fn map_job_view(job: crate::domain::job_run::JobRun) -> DashboardJobView {
    DashboardJobView {
        job_type: job.job_type,
        status_label: status_label(&job.status).to_owned(),
        status: job.status,
        started_at: display_datetime(job.started_at),
        finished_at: job
            .finished_at
            .map(display_datetime)
            .unwrap_or_else(|| "-".to_owned()),
        error_message: job.error_message.unwrap_or_else(|| "-".to_owned()),
    }
}

fn map_alert_view(alert: AlertListItem) -> DashboardAlertView {
    let status = alert.status;
    let status_label = status_label(&status).to_owned();

    DashboardAlertView {
        fund_label: format!("{} ({})", alert.fund_name, alert.fund_code),
        reason: alert.reason,
        status,
        status_label,
        triggered_at: display_datetime(alert.triggered_at),
    }
}

fn job_hint(job: &crate::domain::job_run::JobRun) -> String {
    job.error_message
        .clone()
        .unwrap_or_else(|| format!("开始于 {}", display_datetime(job.started_at)))
}

fn alert_hint(alert: &AlertListItem) -> String {
    format!(
        "{} | {}",
        alert.reason,
        display_datetime(alert.triggered_at)
    )
}

fn status_label(status: &str) -> &'static str {
    match status {
        "success" => "成功",
        "partial_success" => "部分成功",
        "failed" => "失败",
        "running" => "执行中",
        "skipped" => "已跳过",
        "new" => "新告警",
        "processed" => "已处理",
        "ignored" => "已忽略",
        _ => "未知状态",
    }
}

fn display_datetime(value: OffsetDateTime) -> String {
    let offset = UtcOffset::from_hms(8, 0, 0).expect("valid Asia/Shanghai UTC offset");
    format!("{}", value.to_offset(offset))
}
