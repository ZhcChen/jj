use crate::domain::alert_event::{AlertEvent, NewAlertEvent};
use anyhow::{Context, Result};
use sqlx::{FromRow, SqlitePool};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct AlertListItem {
    pub id: i64,
    pub rule_id: i64,
    pub fund_id: i64,
    pub fund_code: String,
    pub fund_name: String,
    pub rule_type: String,
    pub reason: String,
    pub status: String,
    pub triggered_at: OffsetDateTime,
    pub notification_result: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct AlertRepo {
    pool: SqlitePool,
}

impl AlertRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: NewAlertEvent) -> Result<AlertEvent> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            INSERT INTO alert_events (
                rule_id, fund_id, reason, status, triggered_at,
                notification_result, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(input.rule_id)
        .bind(input.fund_id)
        .bind(input.reason)
        .bind(input.status)
        .bind(input.triggered_at)
        .bind(input.notification_result)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("创建告警事件失败")?;

        self.find_by_id(result.last_insert_rowid())
            .await?
            .context("创建告警事件后读取记录失败")
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<AlertEvent>> {
        sqlx::query_as::<_, AlertEvent>(
            r#"
            SELECT
                id, rule_id, fund_id, reason, status, triggered_at,
                notification_result, created_at, updated_at
            FROM alert_events
            ORDER BY triggered_at DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("查询最近告警事件失败")
    }

    pub async fn list_recent_with_context(&self, limit: i64) -> Result<Vec<AlertListItem>> {
        sqlx::query_as::<_, AlertListItem>(
            r#"
            SELECT
                alert_events.id,
                alert_events.rule_id,
                alert_events.fund_id,
                funds.code AS fund_code,
                funds.name AS fund_name,
                monitor_rules.rule_type,
                alert_events.reason,
                alert_events.status,
                alert_events.triggered_at,
                alert_events.notification_result,
                alert_events.created_at,
                alert_events.updated_at
            FROM alert_events
            INNER JOIN funds ON funds.id = alert_events.fund_id
            INNER JOIN monitor_rules ON monitor_rules.id = alert_events.rule_id
            ORDER BY alert_events.triggered_at DESC, alert_events.id DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("查询带上下文的告警事件失败")
    }

    pub async fn update_status(&self, id: i64, status: &str) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE alert_events
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新告警状态失败")?;

        Ok(())
    }

    pub async fn update_notification_result(
        &self,
        id: i64,
        notification_result: Option<&str>,
    ) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE alert_events
            SET notification_result = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(notification_result)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新告警通知结果失败")?;

        Ok(())
    }

    pub async fn latest_for_rule_and_fund(
        &self,
        rule_id: i64,
        fund_id: i64,
    ) -> Result<Option<AlertEvent>> {
        sqlx::query_as::<_, AlertEvent>(
            r#"
            SELECT
                id, rule_id, fund_id, reason, status, triggered_at,
                notification_result, created_at, updated_at
            FROM alert_events
            WHERE rule_id = ? AND fund_id = ?
            ORDER BY triggered_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(rule_id)
        .bind(fund_id)
        .fetch_optional(&self.pool)
        .await
        .context("查询规则最近告警事件失败")
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<AlertEvent>> {
        sqlx::query_as::<_, AlertEvent>(
            r#"
            SELECT
                id, rule_id, fund_id, reason, status, triggered_at,
                notification_result, created_at, updated_at
            FROM alert_events
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("按 ID 查询告警事件失败")
    }
}
