use crate::{
    app::state::AppState,
    domain::monitor_rule::{MonitorRule, NewMonitorRule},
    storage::{fund_repo::FundRepo, rule_repo::RuleRepo},
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
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};
use time::{OffsetDateTime, UtcOffset};

use super::layout::render_html;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rules", get(list_rules).post(create_rule))
        .route("/rules/{id}/toggle", post(toggle_rule))
        .route("/rules/{id}/delete", post(delete_rule))
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RulesQuery {
    updated: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RuleFormInput {
    target_scope: String,
    fund_id: String,
    group_name: String,
    rule_type: String,
    cooldown_minutes: String,
    change_rate_gte: String,
    change_rate_lte: String,
    nav_min: String,
    nav_max: String,
    deviation_abs_gte: String,
    deviation_gte: String,
    deviation_lte: String,
}

impl Default for RuleFormInput {
    fn default() -> Self {
        Self {
            target_scope: "all".to_owned(),
            fund_id: String::new(),
            group_name: String::new(),
            rule_type: String::new(),
            cooldown_minutes: "30".to_owned(),
            change_rate_gte: String::new(),
            change_rate_lte: String::new(),
            nav_min: String::new(),
            nav_max: String::new(),
            deviation_abs_gte: String::new(),
            deviation_gte: String::new(),
            deviation_lte: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct ToggleRuleForm {
    enabled: String,
}

#[derive(Debug, Clone)]
struct RuleView {
    id: i64,
    target_label: String,
    rule_type: String,
    rule_type_label: String,
    threshold_summary: String,
    enabled: bool,
    status_class: String,
    status_label: String,
    cooldown_minutes: i64,
    last_triggered_at: String,
}

#[derive(Debug, Clone)]
struct FundOption {
    label: String,
    value: String,
}

#[derive(Template)]
#[template(path = "rules/index.html")]
struct RulesTemplate {
    title: &'static str,
    nav_key: &'static str,
    rules: Vec<RuleView>,
    funds: Vec<FundOption>,
    groups: Vec<String>,
    has_error: bool,
    error_message: String,
    has_notice: bool,
    notice_message: String,
    form: RuleFormInput,
}

async fn list_rules(
    State(state): State<AppState>,
    Query(query): Query<RulesQuery>,
) -> Result<Response, StatusCode> {
    let notice = query.updated.as_deref().and_then(updated_notice_message);
    render_rules_page(
        &state,
        StatusCode::OK,
        RuleFormInput::default(),
        None,
        notice,
    )
    .await
}

async fn create_rule(
    State(state): State<AppState>,
    Form(form): Form<RuleFormInput>,
) -> Result<Response, StatusCode> {
    let input = match build_new_rule(&form) {
        Ok(input) => input,
        Err(message) => {
            return render_rules_page(&state, StatusCode::BAD_REQUEST, form, Some(message), None)
                .await;
        }
    };

    RuleRepo::new(state.pool.clone())
        .create(input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/rules?updated=created").into_response())
}

async fn toggle_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<ToggleRuleForm>,
) -> Result<Response, StatusCode> {
    let enabled = match form.enabled.trim() {
        "true" => true,
        "false" => false,
        _ => {
            return render_rules_page(
                &state,
                StatusCode::BAD_REQUEST,
                RuleFormInput::default(),
                Some("规则启停参数不合法".to_owned()),
                None,
            )
            .await;
        }
    };

    RuleRepo::new(state.pool.clone())
        .set_enabled(id, enabled)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated = if enabled { "enabled" } else { "disabled" };
    Ok(Redirect::to(&format!("/rules?updated={updated}")).into_response())
}

async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, StatusCode> {
    RuleRepo::new(state.pool.clone())
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/rules?updated=deleted").into_response())
}

async fn render_rules_page(
    state: &AppState,
    status: StatusCode,
    form: RuleFormInput,
    error_message: Option<String>,
    notice_message: Option<String>,
) -> Result<Response, StatusCode> {
    let fund_repo = FundRepo::new(state.pool.clone());
    let rule_repo = RuleRepo::new(state.pool.clone());

    let funds = fund_repo
        .list_active()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rules = rule_repo
        .list_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let fund_options = funds
        .iter()
        .map(|fund| FundOption {
            label: format!("{} ({})", fund.name, fund.code),
            value: fund.id.to_string(),
        })
        .collect::<Vec<_>>();
    let fund_labels = funds
        .iter()
        .map(|fund| (fund.id, format!("{} ({})", fund.name, fund.code)))
        .collect::<HashMap<_, _>>();
    let groups = funds
        .iter()
        .filter_map(|fund| fund.group_name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let template = RulesTemplate {
        title: "规则管理",
        nav_key: "rules",
        rules: rules
            .iter()
            .map(|rule| map_rule_view(rule, &fund_labels))
            .collect(),
        funds: fund_options,
        groups,
        has_error: error_message.is_some(),
        error_message: error_message.unwrap_or_default(),
        has_notice: notice_message.is_some(),
        notice_message: notice_message.unwrap_or_default(),
        form,
    };

    Ok((status, render_html(&template)?).into_response())
}

fn build_new_rule(form: &RuleFormInput) -> Result<NewMonitorRule, String> {
    let target_scope = if form.target_scope.trim().is_empty() {
        "all"
    } else {
        form.target_scope.trim()
    };
    let rule_type = form.rule_type.trim();
    if rule_type.is_empty() {
        return Err("规则类型不能为空".to_owned());
    }

    let cooldown_minutes = parse_i64_field(&form.cooldown_minutes, "冷却时间")?.unwrap_or(30);
    if cooldown_minutes < 0 {
        return Err("冷却时间不能小于 0".to_owned());
    }

    let (fund_id, group_name) = match target_scope {
        "all" => (None, None),
        "fund" => {
            let Some(fund_id) = parse_i64_field(&form.fund_id, "基金 ID")? else {
                return Err("按基金生效时必须选择一只基金".to_owned());
            };
            (Some(fund_id), None)
        }
        "group" => {
            let group_name = form.group_name.trim();
            if group_name.is_empty() {
                return Err("按分组生效时必须填写分组名称".to_owned());
            }
            (None, Some(group_name.to_owned()))
        }
        _ => return Err("规则目标范围不合法".to_owned()),
    };

    let threshold_config = build_threshold_config(rule_type, form)?;

    Ok(NewMonitorRule {
        fund_id,
        group_name,
        rule_type: rule_type.to_owned(),
        threshold_config,
        enabled: true,
        cooldown_minutes,
    })
}

fn build_threshold_config(rule_type: &str, form: &RuleFormInput) -> Result<String, String> {
    let mut map = Map::new();

    match rule_type {
        "change_rate_threshold" => {
            insert_optional_number(&mut map, "gte", &form.change_rate_gte, "涨跌幅 gte")?;
            insert_optional_number(&mut map, "lte", &form.change_rate_lte, "涨跌幅 lte")?;
        }
        "nav_range" => {
            insert_optional_number(&mut map, "min", &form.nav_min, "净值最小值")?;
            insert_optional_number(&mut map, "max", &form.nav_max, "净值最大值")?;
        }
        "estimated_nav_deviation" => {
            insert_optional_number(
                &mut map,
                "abs_gte",
                &form.deviation_abs_gte,
                "估值偏离 abs_gte",
            )?;
            insert_optional_number(&mut map, "gte", &form.deviation_gte, "估值偏离 gte")?;
            insert_optional_number(&mut map, "lte", &form.deviation_lte, "估值偏离 lte")?;
        }
        _ => return Err("不支持的规则类型".to_owned()),
    }

    if map.is_empty() {
        return Err("当前规则至少需要填写一个阈值条件".to_owned());
    }

    serde_json::to_string(&map).map_err(|_| "序列化规则阈值失败".to_owned())
}

fn insert_optional_number(
    map: &mut Map<String, Value>,
    key: &str,
    raw: &str,
    label: &str,
) -> Result<(), String> {
    let Some(value) = parse_f64_field(raw, label)? else {
        return Ok(());
    };
    map.insert(key.to_owned(), Value::from(value));
    Ok(())
}

fn parse_f64_field(raw: &str, label: &str) -> Result<Option<f64>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    trimmed
        .parse::<f64>()
        .map(Some)
        .map_err(|_| format!("{label} 必须是数字"))
}

fn parse_i64_field(raw: &str, label: &str) -> Result<Option<i64>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    trimmed
        .parse::<i64>()
        .map(Some)
        .map_err(|_| format!("{label} 必须是整数"))
}

fn map_rule_view(rule: &MonitorRule, fund_labels: &HashMap<i64, String>) -> RuleView {
    RuleView {
        id: rule.id,
        target_label: target_label(rule, fund_labels),
        rule_type: rule.rule_type.clone(),
        rule_type_label: rule_type_label(&rule.rule_type).to_owned(),
        threshold_summary: threshold_summary(rule),
        enabled: rule.enabled,
        status_class: if rule.enabled {
            "success".to_owned()
        } else {
            "skipped".to_owned()
        },
        status_label: if rule.enabled {
            "已启用".to_owned()
        } else {
            "已停用".to_owned()
        },
        cooldown_minutes: rule.cooldown_minutes,
        last_triggered_at: rule
            .last_triggered_at
            .map(display_datetime)
            .unwrap_or_else(|| "-".to_owned()),
    }
}

fn target_label(rule: &MonitorRule, fund_labels: &HashMap<i64, String>) -> String {
    if let Some(fund_id) = rule.fund_id {
        return fund_labels
            .get(&fund_id)
            .cloned()
            .unwrap_or_else(|| format!("基金 #{fund_id}"));
    }
    if let Some(group_name) = rule.group_name.as_deref() {
        return format!("分组：{group_name}");
    }
    "全局".to_owned()
}

fn threshold_summary(rule: &MonitorRule) -> String {
    let Ok(value) = serde_json::from_str::<Value>(&rule.threshold_config) else {
        return rule.threshold_config.clone();
    };

    match value {
        Value::Object(map) => map
            .into_iter()
            .map(|(key, value)| format!("{key}={}", stringify_json_value(&value)))
            .collect::<Vec<_>>()
            .join(", "),
        _ => rule.threshold_config.clone(),
    }
}

fn stringify_json_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn rule_type_label(rule_type: &str) -> &'static str {
    match rule_type {
        "change_rate_threshold" => "涨跌幅阈值",
        "nav_range" => "净值区间",
        "estimated_nav_deviation" => "估值偏离",
        _ => "未知规则",
    }
}

fn updated_notice_message(updated: &str) -> Option<String> {
    match updated {
        "created" => Some("已新增监控规则".to_owned()),
        "enabled" => Some("已启用该规则".to_owned()),
        "disabled" => Some("已停用该规则".to_owned()),
        "deleted" => Some("已删除该规则".to_owned()),
        _ => None,
    }
}

fn display_datetime(value: OffsetDateTime) -> String {
    let offset = UtcOffset::from_hms(8, 0, 0).expect("valid Asia/Shanghai UTC offset");
    format!("{}", value.to_offset(offset))
}
