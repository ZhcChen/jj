use fund_monitor::{
    domain::{
        alert_event::NewAlertEvent,
        fund::{NewFund, UpdateFundMetadata},
        fund_quote::NewFundQuote,
        monitor_rule::NewMonitorRule,
    },
    storage::{
        alert_repo::AlertRepo, app_setting_repo::AppSettingRepo, db, fund_repo::FundRepo,
        job_repo::JobRepo, quote_repo::QuoteRepo, rule_repo::RuleRepo,
    },
};
use tempfile::tempdir;
use time::OffsetDateTime;

#[tokio::test]
async fn repositories_support_core_create_query_and_status_transitions() {
    let temp_dir = tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("repositories.db");
    let database_url = format!("sqlite://{}", db_path.display());
    let pool = db::initialize_database(&database_url)
        .await
        .expect("initialize database");

    let fund_repo = FundRepo::new(pool.clone());
    let quote_repo = QuoteRepo::new(pool.clone());
    let rule_repo = RuleRepo::new(pool.clone());
    let alert_repo = AlertRepo::new(pool.clone());
    let job_repo = JobRepo::new(pool.clone());
    let setting_repo = AppSettingRepo::new(pool.clone());

    let fund = fund_repo
        .create(NewFund {
            code: "000001".to_owned(),
            name: "示例基金".to_owned(),
            note: Some("核心关注".to_owned()),
            group_name: Some("长期".to_owned()),
            tags: Some("指数,低波".to_owned()),
            enabled: true,
        })
        .await
        .expect("create fund");

    let quote = quote_repo
        .insert(NewFundQuote {
            fund_id: fund.id,
            unit_nav: Some(1.2345),
            estimated_nav: Some(1.2401),
            change_rate: Some(0.87),
            fetched_at: OffsetDateTime::now_utc(),
            source: "mock-source".to_owned(),
        })
        .await
        .expect("insert quote");

    let latest_quote = quote_repo
        .latest_for_fund(fund.id)
        .await
        .expect("latest quote")
        .expect("quote exists");
    assert_eq!(latest_quote.id, quote.id);

    let updated_fund = UpdateFundMetadata {
        name: "示例基金A".to_owned(),
        note: Some("更新后的备注".to_owned()),
        group_name: Some("观察".to_owned()),
        tags: Some("指数".to_owned()),
    };
    fund_repo
        .update_metadata(fund.id, updated_fund)
        .await
        .expect("update fund");
    let found_by_code = fund_repo
        .find_by_code("000001")
        .await
        .expect("find fund by code")
        .expect("fund exists");
    assert_eq!(found_by_code.name, "示例基金A");

    let rule = rule_repo
        .create(NewMonitorRule {
            fund_id: Some(fund.id),
            group_name: None,
            rule_type: "change_rate_threshold".to_owned(),
            threshold_config: r#"{"gte":0.5}"#.to_owned(),
            enabled: true,
            cooldown_minutes: 30,
        })
        .await
        .expect("create rule");

    let enabled_rules = rule_repo.list_enabled().await.expect("enabled rules");
    assert_eq!(enabled_rules.len(), 1);
    assert_eq!(enabled_rules[0].id, rule.id);

    let alert = alert_repo
        .create(NewAlertEvent {
            rule_id: rule.id,
            fund_id: fund.id,
            reason: "涨跌幅超过阈值".to_owned(),
            status: "new".to_owned(),
            triggered_at: OffsetDateTime::now_utc(),
            notification_result: None,
        })
        .await
        .expect("create alert");
    alert_repo
        .update_status(alert.id, "processed")
        .await
        .expect("update alert status");
    let updated_alert = alert_repo
        .find_by_id(alert.id)
        .await
        .expect("find alert by id")
        .expect("alert exists");
    assert_eq!(updated_alert.status, "processed");

    let job = job_repo.start("poll_funds").await.expect("start job");
    let finished_job = job_repo
        .finish(job.id, "failed", Some("mock network error"))
        .await
        .expect("finish job");
    assert_eq!(finished_job.status, "failed");
    assert_eq!(
        finished_job.error_message.as_deref(),
        Some("mock network error")
    );

    let setting = setting_repo
        .set("data_source", "mock-source")
        .await
        .expect("set app setting");
    assert_eq!(setting.value, "mock-source");

    fund_repo.disable(fund.id).await.expect("disable fund");
    let active_funds = fund_repo.list_active().await.expect("list active funds");
    assert!(active_funds.is_empty());

    rule_repo
        .set_enabled(rule.id, false)
        .await
        .expect("disable rule");
    let enabled_rules = rule_repo
        .list_enabled()
        .await
        .expect("enabled rules after disable");
    assert!(enabled_rules.is_empty());

    let jobs = job_repo.list_recent(5).await.expect("recent jobs");
    assert_eq!(jobs.len(), 1);
}
