use crate::domain::alert_event::{AlertEvent, NewAlertEvent};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use time::OffsetDateTime;

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
