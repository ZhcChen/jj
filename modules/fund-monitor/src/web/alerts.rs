use crate::{
    app::state::AppState,
    storage::alert_repo::{AlertListItem, AlertRepo},
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;

use super::layout::{display_datetime, render_html};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/alerts", get(list_alerts))
        .route("/alerts/{id}/status", post(update_alert_status))
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct AlertsQuery {
    updated: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct UpdateAlertStatusForm {
    status: String,
}

#[derive(Debug, Clone)]
struct AlertView {
    id: i64,
    fund_label: String,
    rule_type: String,
    reason: String,
    status: String,
    status_label: String,
    triggered_at: String,
    notification_result: String,
}

#[derive(Template)]
#[template(path = "alerts/index.html")]
struct AlertsTemplate {
    title: &'static str,
    nav_key: &'static str,
    alerts: Vec<AlertView>,
    has_error: bool,
    error_message: String,
    has_notice: bool,
    notice_message: String,
}

async fn list_alerts(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> Result<Response, StatusCode> {
    let notice_message = query
        .updated
        .as_deref()
        .and_then(status_updated_notice_message);

    render_alerts_page(&state, None, notice_message).await
}

async fn update_alert_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<UpdateAlertStatusForm>,
) -> Result<Response, StatusCode> {
    let next_status = form.status.trim();
    let Some(_) = status_updated_notice_message(next_status) else {
        return render_alerts_page(&state, Some("不支持的告警状态".to_owned()), None).await;
    };

    let repo = AlertRepo::new(state.pool.clone());
    repo.update_status(id, next_status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/alerts?updated={next_status}")).into_response())
}

async fn render_alerts_page(
    state: &AppState,
    error_message: Option<String>,
    notice_message: Option<String>,
) -> Result<Response, StatusCode> {
    let repo = AlertRepo::new(state.pool.clone());
    let alerts = repo
        .list_recent_with_context(100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(render_html(&AlertsTemplate {
        title: "告警列表",
        nav_key: "alerts",
        alerts: alerts.into_iter().map(map_alert_view).collect(),
        has_error: error_message.is_some(),
        error_message: error_message.unwrap_or_default(),
        has_notice: notice_message.is_some(),
        notice_message: notice_message.unwrap_or_default(),
    })?
    .into_response())
}

fn map_alert_view(alert: AlertListItem) -> AlertView {
    let status = alert.status;
    AlertView {
        id: alert.id,
        fund_label: format!("{} ({})", alert.fund_name, alert.fund_code),
        rule_type: alert.rule_type,
        reason: alert.reason,
        status_label: status_label(&status).to_owned(),
        status,
        triggered_at: display_datetime(alert.triggered_at),
        notification_result: alert
            .notification_result
            .unwrap_or_else(|| "未发送外部通知".to_owned()),
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "new" => "新告警",
        "processed" => "已处理",
        "ignored" => "已忽略",
        _ => "未知状态",
    }
}

fn status_updated_notice_message(status: &str) -> Option<String> {
    match status {
        "processed" => Some("已将告警标记为已处理".to_owned()),
        "ignored" => Some("已将告警标记为已忽略".to_owned()),
        _ => None,
    }
}
