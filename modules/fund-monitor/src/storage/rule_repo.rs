use crate::domain::monitor_rule::{MonitorRule, NewMonitorRule};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct RuleRepo {
    pool: SqlitePool,
}

impl RuleRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: NewMonitorRule) -> Result<MonitorRule> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            INSERT INTO monitor_rules (
                fund_id, group_name, rule_type, threshold_config, enabled,
                cooldown_minutes, last_triggered_at, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(input.fund_id)
        .bind(input.group_name)
        .bind(input.rule_type)
        .bind(input.threshold_config)
        .bind(input.enabled)
        .bind(input.cooldown_minutes)
        .bind(Option::<OffsetDateTime>::None)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("创建监控规则失败")?;

        self.find_by_id(result.last_insert_rowid())
            .await?
            .context("创建监控规则后读取记录失败")
    }

    pub async fn list_enabled(&self) -> Result<Vec<MonitorRule>> {
        sqlx::query_as::<_, MonitorRule>(
            r#"
            SELECT
                id, fund_id, group_name, rule_type, threshold_config, enabled,
                cooldown_minutes, last_triggered_at, created_at, updated_at
            FROM monitor_rules
            WHERE enabled = 1
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("查询启用规则失败")
    }

    pub async fn list_all(&self) -> Result<Vec<MonitorRule>> {
        sqlx::query_as::<_, MonitorRule>(
            r#"
            SELECT
                id, fund_id, group_name, rule_type, threshold_config, enabled,
                cooldown_minutes, last_triggered_at, created_at, updated_at
            FROM monitor_rules
            ORDER BY id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("查询规则列表失败")
    }

    pub async fn set_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE monitor_rules
            SET enabled = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(enabled)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新规则启用状态失败")?;

        Ok(())
    }

    pub async fn mark_triggered(&self, id: i64, triggered_at: OffsetDateTime) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE monitor_rules
            SET last_triggered_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(triggered_at)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新规则最近触发时间失败")?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<MonitorRule>> {
        sqlx::query_as::<_, MonitorRule>(
            r#"
            SELECT
                id, fund_id, group_name, rule_type, threshold_config, enabled,
                cooldown_minutes, last_triggered_at, created_at, updated_at
            FROM monitor_rules
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("按 ID 查询监控规则失败")
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM monitor_rules
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .context("删除监控规则失败")?;

        Ok(())
    }
}
