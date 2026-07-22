use crate::domain::fund::{Fund, NewFund, UpdateFundMetadata};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct FundRepo {
    pool: SqlitePool,
}

impl FundRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: NewFund) -> Result<Fund> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            INSERT INTO funds (code, name, note, group_name, tags, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(input.code)
        .bind(input.name)
        .bind(input.note)
        .bind(input.group_name)
        .bind(input.tags)
        .bind(input.enabled)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("创建基金失败")?;

        self.find_by_id(result.last_insert_rowid())
            .await?
            .context("创建基金后读取记录失败")
    }

    pub async fn list_active(&self) -> Result<Vec<Fund>> {
        sqlx::query_as::<_, Fund>(
            r#"
            SELECT id, code, name, note, group_name, tags, enabled, created_at, updated_at
            FROM funds
            WHERE enabled = 1
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("查询启用基金列表失败")
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Fund>> {
        sqlx::query_as::<_, Fund>(
            r#"
            SELECT id, code, name, note, group_name, tags, enabled, created_at, updated_at
            FROM funds
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("按 ID 查询基金失败")
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<Fund>> {
        sqlx::query_as::<_, Fund>(
            r#"
            SELECT id, code, name, note, group_name, tags, enabled, created_at, updated_at
            FROM funds
            WHERE code = ?
            LIMIT 1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .context("按代码查询基金失败")
    }

    pub async fn update_metadata(&self, id: i64, input: UpdateFundMetadata) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE funds
            SET name = ?, note = ?, group_name = ?, tags = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(input.name)
        .bind(input.note)
        .bind(input.group_name)
        .bind(input.tags)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新基金元数据失败")?;

        Ok(())
    }

    pub async fn disable(&self, id: i64) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE funds
            SET enabled = 0, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("停用基金失败")?;

        Ok(())
    }
}
