use crate::domain::app_setting::AppSetting;
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct AppSettingRepo {
    pool: SqlitePool,
}

impl AppSettingRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<AppSetting> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("写入应用配置失败")?;

        self.get(key).await?.context("写入应用配置后读取失败")
    }

    pub async fn get(&self, key: &str) -> Result<Option<AppSetting>> {
        sqlx::query_as::<_, AppSetting>(
            r#"
            SELECT key, value, updated_at
            FROM app_settings
            WHERE key = ?
            LIMIT 1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .context("读取应用配置失败")
    }
}
