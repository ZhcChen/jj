use crate::domain::{
    alert_event::NewAlertEvent, fund::Fund, fund_quote::FundQuote, monitor_rule::MonitorRule,
};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct RuleTrigger {
    pub fund_id: i64,
    pub reason: String,
    pub rule_id: i64,
    pub triggered_at: OffsetDateTime,
}

pub struct RuleEngine;

impl RuleEngine {
    pub fn evaluate(
        rule: &MonitorRule,
        fund: &Fund,
        quote: &FundQuote,
        last_alert_at: Option<OffsetDateTime>,
    ) -> Result<Option<RuleTrigger>> {
        if !rule.enabled || !rule_applies_to_fund(rule, fund) {
            return Ok(None);
        }

        let matches = match rule.rule_type.as_str() {
            "change_rate_threshold" => evaluate_change_rate_rule(rule, fund, quote)?,
            "nav_range" => evaluate_nav_range_rule(rule, fund, quote)?,
            "estimated_nav_deviation" => evaluate_estimated_nav_deviation_rule(rule, fund, quote)?,
            other => bail!("不支持的规则类型：{other}"),
        };

        if matches.is_none() {
            return Ok(None);
        }

        if in_cooldown(rule.cooldown_minutes, last_alert_at, quote.fetched_at) {
            return Ok(None);
        }

        Ok(matches.map(|reason| RuleTrigger {
            fund_id: fund.id,
            reason,
            rule_id: rule.id,
            triggered_at: quote.fetched_at,
        }))
    }
}

impl RuleTrigger {
    pub fn into_new_alert(self) -> NewAlertEvent {
        NewAlertEvent {
            rule_id: self.rule_id,
            fund_id: self.fund_id,
            reason: self.reason,
            status: "new".to_owned(),
            triggered_at: self.triggered_at,
            notification_result: None,
        }
    }
}

fn evaluate_change_rate_rule(
    rule: &MonitorRule,
    fund: &Fund,
    quote: &FundQuote,
) -> Result<Option<String>> {
    let config: ChangeRateThresholdConfig = parse_rule_config(rule)?;
    config.validate()?;

    let change_rate = quote
        .change_rate
        .ok_or_else(|| anyhow::anyhow!("基金 {} 缺少涨跌幅数据", fund.code))?;

    if config.matches(change_rate) {
        return Ok(Some(format!(
            "基金 {} 涨跌幅 {:.2}% 命中涨跌幅阈值规则",
            fund.code, change_rate
        )));
    }

    Ok(None)
}

fn evaluate_nav_range_rule(
    rule: &MonitorRule,
    fund: &Fund,
    quote: &FundQuote,
) -> Result<Option<String>> {
    let config: NavRangeConfig = parse_rule_config(rule)?;
    config.validate()?;

    let unit_nav = quote
        .unit_nav
        .ok_or_else(|| anyhow::anyhow!("基金 {} 缺少单位净值数据", fund.code))?;

    if config.matches(unit_nav) {
        return Ok(Some(format!(
            "基金 {} 单位净值 {:.4} 命中净值区间规则",
            fund.code, unit_nav
        )));
    }

    Ok(None)
}

fn evaluate_estimated_nav_deviation_rule(
    rule: &MonitorRule,
    fund: &Fund,
    quote: &FundQuote,
) -> Result<Option<String>> {
    let config: EstimatedNavDeviationConfig = parse_rule_config(rule)?;
    config.validate()?;

    let unit_nav = quote
        .unit_nav
        .ok_or_else(|| anyhow::anyhow!("基金 {} 缺少单位净值数据", fund.code))?;
    if unit_nav == 0.0 {
        bail!("基金 {} 单位净值为 0，无法计算估值偏离", fund.code);
    }

    let estimated_nav = quote
        .estimated_nav
        .ok_or_else(|| anyhow::anyhow!("基金 {} 缺少估值数据", fund.code))?;
    let deviation = ((estimated_nav - unit_nav) / unit_nav) * 100.0;

    if config.matches(deviation) {
        return Ok(Some(format!(
            "基金 {} 估值偏离 {:.2}% 命中估值偏离规则",
            fund.code, deviation
        )));
    }

    Ok(None)
}

fn parse_rule_config<T>(rule: &MonitorRule) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(&rule.threshold_config).with_context(|| {
        format!(
            "解析规则配置失败，rule_id={}，rule_type={}",
            rule.id, rule.rule_type
        )
    })
}

fn rule_applies_to_fund(rule: &MonitorRule, fund: &Fund) -> bool {
    if let Some(rule_fund_id) = rule.fund_id
        && rule_fund_id != fund.id
    {
        return false;
    }

    if let Some(rule_group_name) = rule.group_name.as_deref()
        && fund.group_name.as_deref() != Some(rule_group_name)
    {
        return false;
    }

    true
}

fn in_cooldown(
    cooldown_minutes: i64,
    last_alert_at: Option<OffsetDateTime>,
    current_triggered_at: OffsetDateTime,
) -> bool {
    if cooldown_minutes <= 0 {
        return false;
    }

    let Some(last_alert_at) = last_alert_at else {
        return false;
    };

    let cooldown = Duration::minutes(cooldown_minutes);
    current_triggered_at < last_alert_at + cooldown
}

#[derive(Debug, Clone, Deserialize)]
struct ChangeRateThresholdConfig {
    gte: Option<f64>,
    lte: Option<f64>,
}

impl ChangeRateThresholdConfig {
    fn validate(&self) -> Result<()> {
        if self.gte.is_none() && self.lte.is_none() {
            bail!("涨跌幅阈值规则至少需要一个 gte 或 lte 条件");
        }

        Ok(())
    }

    fn matches(&self, value: f64) -> bool {
        let gte_matches = self.gte.is_none_or(|gte| value >= gte);
        let lte_matches = self.lte.is_none_or(|lte| value <= lte);
        gte_matches && lte_matches
    }
}

#[derive(Debug, Clone, Deserialize)]
struct NavRangeConfig {
    min: Option<f64>,
    max: Option<f64>,
}

impl NavRangeConfig {
    fn validate(&self) -> Result<()> {
        if self.min.is_none() && self.max.is_none() {
            bail!("净值区间规则至少需要一个 min 或 max 条件");
        }

        Ok(())
    }

    fn matches(&self, value: f64) -> bool {
        let min_matches = self.min.is_none_or(|min| value >= min);
        let max_matches = self.max.is_none_or(|max| value <= max);
        min_matches && max_matches
    }
}

#[derive(Debug, Clone, Deserialize)]
struct EstimatedNavDeviationConfig {
    abs_gte: Option<f64>,
    gte: Option<f64>,
    lte: Option<f64>,
}

impl EstimatedNavDeviationConfig {
    fn validate(&self) -> Result<()> {
        if self.abs_gte.is_none() && self.gte.is_none() && self.lte.is_none() {
            bail!("估值偏离规则至少需要一个 abs_gte、gte 或 lte 条件");
        }

        Ok(())
    }

    fn matches(&self, value: f64) -> bool {
        let abs_matches = self.abs_gte.is_none_or(|abs_gte| value.abs() >= abs_gte);
        let gte_matches = self.gte.is_none_or(|gte| value >= gte);
        let lte_matches = self.lte.is_none_or(|lte| value <= lte);
        abs_matches && gte_matches && lte_matches
    }
}
