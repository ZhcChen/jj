use crate::app::{errors::render_internal_error, state::AppState};
use askama::Template;
use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};

use super::layout::render_html;

pub fn routes() -> Router<AppState> {
    Router::new().route("/settings", get(show_settings))
}

#[derive(Debug, Clone)]
struct SettingItem {
    label: &'static str,
    value: String,
    hint: String,
}

#[derive(Template)]
#[template(path = "settings/index.html")]
struct SettingsTemplate {
    title: &'static str,
    runtime_settings: Vec<SettingItem>,
    data_source_settings: Vec<SettingItem>,
    notification_settings: Vec<SettingItem>,
}

async fn show_settings(State(state): State<AppState>) -> Response {
    let config = state.config.as_ref();
    let template = SettingsTemplate {
        title: "系统配置",
        runtime_settings: vec![
            SettingItem {
                label: "监听地址",
                value: config.bind_addr.clone(),
                hint: "当前 Web 服务监听的地址和端口".to_owned(),
            },
            SettingItem {
                label: "轮询频率",
                value: format!("{} 秒", config.poll_interval_seconds),
                hint: "后台轮询基金数据的执行间隔".to_owned(),
            },
            SettingItem {
                label: "数据库",
                value: config.database_url.clone(),
                hint: "当前 SQLite 数据库连接字符串".to_owned(),
            },
        ],
        data_source_settings: vec![
            SettingItem {
                label: "基金数据源",
                value: "eastmoney/pingzhongdata".to_owned(),
                hint: "当前用于抓取基金净值、估值和涨跌幅的数据源".to_owned(),
            },
            SettingItem {
                label: "手动抓取入口",
                value: "/funds/{id}/fetch".to_owned(),
                hint: "详情页中的单基金抓取入口".to_owned(),
            },
        ],
        notification_settings: vec![
            SettingItem {
                label: "Telegram 状态",
                value: if state.telegram_notifier.is_some() {
                    "已启用".to_owned()
                } else {
                    "未配置".to_owned()
                },
                hint: "当规则命中后是否会尝试发送 Telegram 告警".to_owned(),
            },
            SettingItem {
                label: "Telegram API",
                value: config.telegram_api_base_url.clone(),
                hint: "Telegram Bot API 基础地址".to_owned(),
            },
            SettingItem {
                label: "Chat ID",
                value: config
                    .telegram_chat_id
                    .clone()
                    .unwrap_or_else(|| "-".to_owned()),
                hint: "告警消息投递的目标会话 ID".to_owned(),
            },
            SettingItem {
                label: "Bot Token",
                value: mask_secret(config.telegram_bot_token.as_deref()),
                hint: "安全起见仅展示部分掩码".to_owned(),
            },
        ],
    };

    match render_html(&template) {
        Ok(html) => html.into_response(),
        Err(_) => render_internal_error("设置页模板渲染失败，请稍后重试。"),
    }
}

fn mask_secret(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "-".to_owned();
    };

    let visible = 4usize.min(value.len());
    let suffix = &value[value.len() - visible..];
    format!(
        "{}{}",
        "*".repeat(value.len().saturating_sub(visible)),
        suffix
    )
}
