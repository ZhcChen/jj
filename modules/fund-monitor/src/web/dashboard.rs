use crate::{
    app::{errors::render_internal_error, state::AppState},
    domain::fund::Fund,
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

use super::layout::{display_datetime, render_html};

pub fn routes() -> Router<AppState> {
    Router::new().route("/dashboard", get(show_dashboard))
}

#[derive(Debug, Clone)]
struct DashboardMetric {
    label: &'static str,
    value: String,
    hint: String,
    tone: &'static str,
}

#[derive(Debug, Clone)]
struct DashboardSignalView {
    tone: &'static str,
    status_label: String,
    summary: String,
    meta: String,
}

#[derive(Debug, Clone)]
struct DashboardJobView {
    job_label: String,
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
    rule_label: String,
    notification_result: String,
}

#[derive(Debug, Clone)]
struct DashboardFundView {
    detail_url: String,
    code: String,
    name: String,
    group_name: String,
    tags: String,
    note: String,
}

#[derive(Debug, Clone)]
struct DashboardAlertFocusView {
    tone: &'static str,
    heading: String,
    reason: String,
    status_label: String,
    triggered_at: String,
    rule_label: String,
    notification_result: String,
    action_href: String,
    action_label: String,
}

#[derive(Template)]
#[template(path = "dashboard/index.html")]
struct DashboardTemplate {
    title: &'static str,
    nav_key: &'static str,
    signal: DashboardSignalView,
    metrics: Vec<DashboardMetric>,
    active_fund_total: usize,
    active_funds: Vec<DashboardFundView>,
    alert_focus: DashboardAlertFocusView,
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
    let new_alert_count = recent_alerts
        .iter()
        .filter(|alert| alert.status == "new")
        .count();

    let metrics = vec![
        DashboardMetric {
            label: "启用基金",
            value: format!("{} 只", funds.len()),
            hint: active_fund_hint(&funds),
            tone: "info",
        },
        DashboardMetric {
            label: "轮询任务",
            value: latest_poll_job
                .map(|job| status_label(&job.status).to_owned())
                .unwrap_or_else(|| "未启动".to_owned()),
            hint: latest_poll_job
                .map(job_hint)
                .unwrap_or_else(|| "轮询调度尚未开始".to_owned()),
            tone: latest_poll_job
                .map(|job| status_tone(&job.status))
                .unwrap_or("neutral"),
        },
        DashboardMetric {
            label: "抓取任务",
            value: latest_fetch_job
                .map(|job| status_label(&job.status).to_owned())
                .unwrap_or_else(|| "未触发".to_owned()),
            hint: latest_fetch_job
                .map(job_hint)
                .unwrap_or_else(|| "尚未生成单基金抓取记录".to_owned()),
            tone: latest_fetch_job
                .map(|job| status_tone(&job.status))
                .unwrap_or("neutral"),
        },
        DashboardMetric {
            label: "告警状态",
            value: alert_metric_value(&recent_alerts, new_alert_count),
            hint: alert_metric_hint(&recent_alerts, new_alert_count),
            tone: alert_metric_tone(&recent_alerts, new_alert_count),
        },
    ];

    Ok(DashboardTemplate {
        title: "总览看板",
        nav_key: "dashboard",
        signal: build_signal(
            &recent_jobs,
            &recent_alerts,
            latest_poll_job,
            latest_fetch_job,
            new_alert_count,
        ),
        metrics,
        active_fund_total: funds.len(),
        active_funds: funds.iter().take(6).map(map_fund_view).collect(),
        alert_focus: build_alert_focus(
            recent_alerts.first(),
            latest_poll_job,
            latest_fetch_job,
            new_alert_count,
        ),
        recent_jobs: recent_jobs.into_iter().map(map_job_view).collect(),
        recent_alerts: recent_alerts.into_iter().map(map_alert_view).collect(),
    })
}

fn map_job_view(job: crate::domain::job_run::JobRun) -> DashboardJobView {
    DashboardJobView {
        job_label: job_type_label(&job.job_type),
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
        rule_label: rule_type_label(&alert.rule_type).to_owned(),
        notification_result: alert
            .notification_result
            .unwrap_or_else(|| "尚未记录通知回执".to_owned()),
    }
}

fn map_fund_view(fund: &Fund) -> DashboardFundView {
    DashboardFundView {
        detail_url: format!("/funds/{}", fund.id),
        code: fund.code.clone(),
        name: fund.name.clone(),
        group_name: fund
            .group_name
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "未分组".to_owned()),
        tags: fund
            .tags
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "未设置标签".to_owned()),
        note: fund
            .note
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "暂无补充备注".to_owned()),
    }
}

fn build_signal(
    recent_jobs: &[crate::domain::job_run::JobRun],
    recent_alerts: &[AlertListItem],
    latest_poll_job: Option<&crate::domain::job_run::JobRun>,
    latest_fetch_job: Option<&crate::domain::job_run::JobRun>,
    new_alert_count: usize,
) -> DashboardSignalView {
    if let Some(alert) = recent_alerts.iter().find(|alert| alert.status == "new") {
        return DashboardSignalView {
            tone: "new",
            status_label: "告警待处理".to_owned(),
            summary: format!(
                "{} 在 {} 命中监控规则，需要尽快确认处理。",
                alert.fund_name,
                display_datetime(alert.triggered_at)
            ),
            meta: format!("规则类型：{}", rule_type_label(&alert.rule_type)),
        };
    }

    if let Some(job) = recent_jobs.iter().find(|job| job.status == "failed") {
        return DashboardSignalView {
            tone: "failed",
            status_label: "任务异常".to_owned(),
            summary: format!(
                "{} 最近一次执行失败，需要检查任务日志。",
                job_type_label(&job.job_type)
            ),
            meta: job_hint(job),
        };
    }

    if let Some(job) = recent_jobs.iter().find(|job| job.status == "running") {
        return DashboardSignalView {
            tone: "running",
            status_label: "任务执行中".to_owned(),
            summary: format!(
                "{} 正在运行，等待本轮结果回写。",
                job_type_label(&job.job_type)
            ),
            meta: format!("开始时间：{}", display_datetime(job.started_at)),
        };
    }

    if latest_poll_job.is_none() && latest_fetch_job.is_none() {
        return DashboardSignalView {
            tone: "neutral",
            status_label: "等待启动".to_owned(),
            summary: "当前还没有轮询或抓取执行记录，监控工作台处于冷启动状态。".to_owned(),
            meta: "建议先检查基金池与调度器是否已启动。".to_owned(),
        };
    }

    let signal_summary = if new_alert_count == 0 {
        "轮询与抓取最近一次执行正常，当前没有新的待处理告警。".to_owned()
    } else {
        format!("最近窗口内已有 {new_alert_count} 条告警，建议继续跟进处理状态。")
    };

    let poll_meta = latest_poll_job
        .map(|job| format!("轮询：{}", display_datetime(job.started_at)))
        .unwrap_or_else(|| "轮询：暂无".to_owned());
    let fetch_meta = latest_fetch_job
        .map(|job| format!("抓取：{}", display_datetime(job.started_at)))
        .unwrap_or_else(|| "抓取：暂无".to_owned());

    DashboardSignalView {
        tone: "success",
        status_label: "运行稳定".to_owned(),
        summary: signal_summary,
        meta: format!("{poll_meta} / {fetch_meta}"),
    }
}

fn build_alert_focus(
    primary_alert: Option<&AlertListItem>,
    latest_poll_job: Option<&crate::domain::job_run::JobRun>,
    latest_fetch_job: Option<&crate::domain::job_run::JobRun>,
    new_alert_count: usize,
) -> DashboardAlertFocusView {
    if let Some(alert) = primary_alert {
        return DashboardAlertFocusView {
            tone: status_tone(&alert.status),
            heading: format!("{} ({})", alert.fund_name, alert.fund_code),
            reason: alert.reason.clone(),
            status_label: status_label(&alert.status).to_owned(),
            triggered_at: display_datetime(alert.triggered_at),
            rule_label: rule_type_label(&alert.rule_type).to_owned(),
            notification_result: alert
                .notification_result
                .clone()
                .unwrap_or_else(|| "尚未记录通知回执".to_owned()),
            action_href: "/alerts".to_owned(),
            action_label: "查看告警列表".to_owned(),
        };
    }

    let fallback_time = latest_fetch_job
        .or(latest_poll_job)
        .map(|job| display_datetime(job.started_at))
        .unwrap_or_else(|| "等待首轮调度".to_owned());

    DashboardAlertFocusView {
        tone: "success",
        heading: "当前告警静默".to_owned(),
        reason: "最近时间窗内没有新的规则命中，可继续关注轮询、抓取与基金池维护状态。".to_owned(),
        status_label: if new_alert_count == 0 {
            "运行稳定".to_owned()
        } else {
            "持续观察".to_owned()
        },
        triggered_at: fallback_time,
        rule_label: "暂无焦点规则".to_owned(),
        notification_result: "建议定期复核规则阈值、冷却时间与基金池覆盖范围。".to_owned(),
        action_href: "/rules".to_owned(),
        action_label: "查看规则管理".to_owned(),
    }
}

fn job_hint(job: &crate::domain::job_run::JobRun) -> String {
    job.error_message
        .clone()
        .unwrap_or_else(|| format!("开始于 {}", display_datetime(job.started_at)))
}

fn active_fund_hint(funds: &[Fund]) -> String {
    let grouped_count = funds
        .iter()
        .filter_map(|fund| fund.group_name.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    if grouped_count == 0 {
        "当前基金池尚未配置分组维度".to_owned()
    } else {
        format!("基金池已覆盖 {grouped_count} 个分组维度")
    }
}

fn alert_metric_value(recent_alerts: &[AlertListItem], new_alert_count: usize) -> String {
    if new_alert_count > 0 {
        return format!("{new_alert_count} 条待处理");
    }

    if recent_alerts.is_empty() {
        "静默".to_owned()
    } else {
        "已归档".to_owned()
    }
}

fn alert_metric_hint(recent_alerts: &[AlertListItem], new_alert_count: usize) -> String {
    if let Some(alert) = recent_alerts.first() {
        if new_alert_count > 0 {
            return format!(
                "{} 于 {} 触发最新告警",
                alert.fund_name,
                display_datetime(alert.triggered_at)
            );
        }

        return format!("最近一条告警状态为 {}", status_label(&alert.status));
    }

    "最近窗口内还没有规则命中产生告警".to_owned()
}

fn alert_metric_tone(recent_alerts: &[AlertListItem], new_alert_count: usize) -> &'static str {
    if new_alert_count > 0 {
        "warning"
    } else if recent_alerts.is_empty() {
        "success"
    } else {
        "neutral"
    }
}

fn job_type_label(job_type: &str) -> String {
    if job_type == "poll_funds" {
        return "基金轮询".to_owned();
    }

    if let Some(code) = job_type.strip_prefix("fund_poll_fetch:") {
        return format!("基金抓取 {code}");
    }

    "系统任务".to_owned()
}

fn rule_type_label(rule_type: &str) -> &'static str {
    match rule_type {
        "change_rate_threshold" => "涨跌幅阈值",
        _ => "监控规则",
    }
}

fn status_tone(status: &str) -> &'static str {
    match status {
        "success" | "processed" => "success",
        "new" => "new",
        "partial_success" | "running" | "ignored" => "running",
        "failed" => "failed",
        "skipped" => "neutral",
        _ => "neutral",
    }
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
