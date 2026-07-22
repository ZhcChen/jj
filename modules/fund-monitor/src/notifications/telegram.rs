use crate::{
    app::config::AppConfig,
    domain::{alert_event::AlertEvent, fund::Fund, monitor_rule::MonitorRule},
};
use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

#[derive(Clone)]
pub struct TelegramNotifier {
    api_base_url: String,
    bot_token: String,
    chat_id: String,
    client: Client,
}

impl TelegramNotifier {
    pub fn from_app_config(config: &AppConfig) -> Result<Option<Self>> {
        match (&config.telegram_bot_token, &config.telegram_chat_id) {
            (None, None) => Ok(None),
            (Some(_), None) | (None, Some(_)) => bail!(
                "FUND_MONITOR_TELEGRAM_BOT_TOKEN 和 FUND_MONITOR_TELEGRAM_CHAT_ID 必须同时配置"
            ),
            (Some(bot_token), Some(chat_id)) => {
                Self::new(&config.telegram_api_base_url, bot_token, chat_id).map(Some)
            }
        }
    }

    pub fn new(api_base_url: &str, bot_token: &str, chat_id: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("fund-monitor/0.1")
            .build()
            .context("创建 Telegram HTTP 客户端失败")?;

        Ok(Self {
            api_base_url: api_base_url.trim_end_matches('/').to_owned(),
            bot_token: bot_token.to_owned(),
            chat_id: chat_id.to_owned(),
            client,
        })
    }

    pub async fn send_alert(
        &self,
        alert: &AlertEvent,
        fund: &Fund,
        rule: &MonitorRule,
    ) -> Result<String> {
        let url = format!("{}/bot{}/sendMessage", self.api_base_url, self.bot_token);
        let request = SendMessageRequest {
            chat_id: self.chat_id.clone(),
            text: render_alert_message(alert, fund, rule),
        };

        let response = self
            .client
            .post(&url)
            .form(&request)
            .send()
            .await
            .with_context(|| format!("调用 Telegram sendMessage 失败：{url}"))?;

        let response = response
            .error_for_status()
            .with_context(|| format!("Telegram sendMessage 返回错误状态：{url}"))?;

        let payload = response
            .json::<TelegramResponse>()
            .await
            .context("解析 Telegram sendMessage 响应失败")?;

        if !payload.ok {
            bail!(
                "Telegram sendMessage 返回失败：{}",
                payload.description.unwrap_or_else(|| "未知错误".to_owned())
            );
        }

        let message_id = payload
            .result
            .map(|result| result.message_id)
            .unwrap_or_default();
        Ok(format!("telegram 发送成功，message_id={message_id}"))
    }
}

fn render_alert_message(alert: &AlertEvent, fund: &Fund, rule: &MonitorRule) -> String {
    let offset = UtcOffset::from_hms(8, 0, 0).expect("valid Asia/Shanghai UTC offset");
    let triggered_at = alert.triggered_at.to_offset(offset);

    format!(
        concat!(
            "[基金告警]\n",
            "基金：{} ({})\n",
            "规则：{}\n",
            "原因：{}\n",
            "触发时间：{}\n",
            "告警状态：{}"
        ),
        fund.name,
        fund.code,
        rule.rule_type,
        alert.reason,
        display_datetime(triggered_at),
        alert.status,
    )
}

fn display_datetime(value: OffsetDateTime) -> String {
    format!("{}", value)
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageRequest {
    chat_id: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Option<TelegramMessage>,
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TelegramMessage {
    message_id: i64,
}
