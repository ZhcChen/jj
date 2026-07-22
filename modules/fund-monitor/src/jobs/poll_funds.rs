use crate::{
    app::state::AppState,
    domain::rule_engine::RuleEngine,
    notifications::telegram::TelegramNotifier,
    providers::fund_source::fetch_and_store_fund_quote_for_job,
    storage::{
        alert_repo::AlertRepo, fund_repo::FundRepo, job_repo::JobRepo, quote_repo::QuoteRepo,
        rule_repo::RuleRepo,
    },
};
use anyhow::{Context, Result};

const ROUND_JOB_TYPE: &str = "poll_funds";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollFundsSummary {
    pub total_funds: usize,
    pub succeeded_funds: usize,
    pub failed_funds: usize,
    pub status: String,
}

pub struct PollFundsJob {
    state: AppState,
}

impl PollFundsJob {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn run_once(&self) -> Result<PollFundsSummary> {
        let fund_repo = FundRepo::new(self.state.pool.clone());
        let quote_repo = QuoteRepo::new(self.state.pool.clone());
        let job_repo = JobRepo::new(self.state.pool.clone());
        let rule_repo = RuleRepo::new(self.state.pool.clone());
        let alert_repo = AlertRepo::new(self.state.pool.clone());

        let round_job = job_repo
            .start(ROUND_JOB_TYPE)
            .await
            .context("创建基金轮询任务记录失败")?;
        tracing::info!(
            category = "poll",
            job_id = round_job.id,
            "poll_funds round started"
        );

        let funds = match fund_repo.list_active().await {
            Ok(funds) => funds,
            Err(err) => {
                finish_round_job(
                    &job_repo,
                    round_job.id,
                    "failed",
                    Some(format!("加载启用基金列表失败：{err:#}")),
                )
                .await?;
                return Err(err).context("查询启用基金列表失败");
            }
        };

        if funds.is_empty() {
            let summary = PollFundsSummary {
                total_funds: 0,
                succeeded_funds: 0,
                failed_funds: 0,
                status: "skipped".to_owned(),
            };

            finish_round_job(
                &job_repo,
                round_job.id,
                &summary.status,
                Some("当前没有启用基金，跳过抓取".to_owned()),
            )
            .await?;

            return Ok(summary);
        }

        let enabled_rules = match rule_repo.list_enabled().await {
            Ok(rules) => rules,
            Err(err) => {
                finish_round_job(
                    &job_repo,
                    round_job.id,
                    "failed",
                    Some(format!("加载启用规则失败：{err:#}")),
                )
                .await?;
                return Err(err).context("查询启用规则失败");
            }
        };

        let mut succeeded_funds = 0usize;
        let mut failed_funds = 0usize;
        let mut failure_messages = Vec::new();
        let mut had_rule_errors = false;

        for fund in funds {
            let job_type = format!("fund_poll_fetch:{}", fund.code);
            match fetch_and_store_fund_quote_for_job(
                self.state.fund_source.as_ref(),
                &fund,
                &quote_repo,
                &job_repo,
                &job_type,
            )
            .await
            {
                Ok(quote) => {
                    succeeded_funds += 1;
                    if let Err(err) = evaluate_rules_for_fund(
                        &self.state,
                        &fund,
                        &quote,
                        &enabled_rules,
                        &rule_repo,
                        &alert_repo,
                    )
                    .await
                    {
                        had_rule_errors = true;
                        tracing::error!(
                            category = "rule",
                            fund_code = %fund.code,
                            error = %format!("{err:#}"),
                            "rule evaluation failed after quote fetch"
                        );
                        failure_messages.push(format!("{}: {err:#}", fund.code));
                    }
                }
                Err(err) => {
                    failed_funds += 1;
                    tracing::error!(
                        category = "fetch",
                        fund_code = %fund.code,
                        error = %err,
                        "fund poll fetch failed"
                    );
                    failure_messages.push(format!("{}: {}", fund.code, err.user_message()));
                }
            }
        }

        let summary = build_summary(succeeded_funds, failed_funds, had_rule_errors);
        finish_round_job(
            &job_repo,
            round_job.id,
            &summary.status,
            build_summary_message(&summary, &failure_messages),
        )
        .await?;
        tracing::info!(
            category = "poll",
            job_id = round_job.id,
            total_funds = summary.total_funds,
            succeeded_funds = summary.succeeded_funds,
            failed_funds = summary.failed_funds,
            status = %summary.status,
            "poll_funds round finished"
        );

        Ok(summary)
    }
}

fn build_summary(
    succeeded_funds: usize,
    failed_funds: usize,
    had_rule_errors: bool,
) -> PollFundsSummary {
    let total_funds = succeeded_funds + failed_funds;
    let status = if failed_funds == 0 && !had_rule_errors {
        "success"
    } else if succeeded_funds == 0 {
        "failed"
    } else {
        "partial_success"
    };

    PollFundsSummary {
        total_funds,
        succeeded_funds,
        failed_funds,
        status: status.to_owned(),
    }
}

fn build_summary_message(
    summary: &PollFundsSummary,
    failure_messages: &[String],
) -> Option<String> {
    if failure_messages.is_empty() {
        return None;
    }

    Some(format!(
        "共 {} 只基金，成功 {}，失败 {}。{}",
        summary.total_funds,
        summary.succeeded_funds,
        summary.failed_funds,
        failure_messages.join("；")
    ))
}

async fn evaluate_rules_for_fund(
    state: &AppState,
    fund: &crate::domain::fund::Fund,
    quote: &crate::domain::fund_quote::FundQuote,
    enabled_rules: &[crate::domain::monitor_rule::MonitorRule],
    rule_repo: &RuleRepo,
    alert_repo: &AlertRepo,
) -> Result<()> {
    for rule in enabled_rules {
        let last_alert_at = alert_repo
            .latest_for_rule_and_fund(rule.id, fund.id)
            .await
            .with_context(|| {
                format!(
                    "查询规则最近告警失败，rule_id={}，fund_id={}",
                    rule.id, fund.id
                )
            })?
            .map(|alert| alert.triggered_at);

        let Some(trigger) = RuleEngine::evaluate(rule, fund, quote, last_alert_at)
            .with_context(|| format!("执行规则失败，rule_id={}，fund_id={}", rule.id, fund.id))?
        else {
            continue;
        };

        let triggered_at = trigger.triggered_at;
        let created_alert = alert_repo
            .create(trigger.into_new_alert())
            .await
            .with_context(|| {
                format!("写入告警事件失败，rule_id={}，fund_id={}", rule.id, fund.id)
            })?;
        let notification_result = deliver_notification(
            state.telegram_notifier.as_deref(),
            &created_alert,
            fund,
            rule,
        )
        .await;
        if notification_result.is_some() {
            alert_repo
                .update_notification_result(created_alert.id, notification_result.as_deref())
                .await
                .with_context(|| {
                    format!(
                        "更新通知结果失败，alert_id={}，fund_id={}",
                        created_alert.id, fund.id
                    )
                })?;
        }
        rule_repo
            .mark_triggered(rule.id, triggered_at)
            .await
            .with_context(|| format!("更新规则触发时间失败，rule_id={}", rule.id))?;
    }

    Ok(())
}

async fn deliver_notification(
    notifier: Option<&TelegramNotifier>,
    alert: &crate::domain::alert_event::AlertEvent,
    fund: &crate::domain::fund::Fund,
    rule: &crate::domain::monitor_rule::MonitorRule,
) -> Option<String> {
    let Some(notifier) = notifier else {
        return None;
    };

    match notifier.send_alert(alert, fund, rule).await {
        Ok(result) => {
            tracing::info!(
                category = "notification",
                fund_code = %fund.code,
                rule_id = rule.id,
                "telegram notification sent"
            );
            Some(result)
        }
        Err(err) => {
            tracing::error!(
                category = "notification",
                fund_code = %fund.code,
                rule_id = rule.id,
                error = %format!("{err:#}"),
                "telegram notification failed"
            );
            Some(format!("telegram 发送失败：{err:#}"))
        }
    }
}

async fn finish_round_job(
    job_repo: &JobRepo,
    job_id: i64,
    status: &str,
    message: Option<String>,
) -> Result<()> {
    job_repo
        .finish(job_id, status, message.as_deref())
        .await
        .with_context(|| format!("更新基金轮询任务记录失败，job_id={job_id}"))?;

    Ok(())
}
