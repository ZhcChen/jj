use crate::domain::fund_quote::{FundQuote, NewFundQuote};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct QuoteRepo {
    pool: SqlitePool,
}

impl QuoteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, input: NewFundQuote) -> Result<FundQuote> {
        let created_at = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            INSERT INTO fund_quotes (
                fund_id, unit_nav, nav_date, confirmed_change_rate, estimated_nav,
                estimated_change_rate, estimated_at, change_rate, fetched_at, source, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(input.fund_id)
        .bind(input.unit_nav)
        .bind(input.nav_date)
        .bind(input.confirmed_change_rate)
        .bind(input.estimated_nav)
        .bind(input.estimated_change_rate)
        .bind(input.estimated_at)
        .bind(input.change_rate)
        .bind(input.fetched_at)
        .bind(input.source)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .context("写入基金行情失败")?;

        self.find_by_id(result.last_insert_rowid())
            .await?
            .context("写入基金行情后读取记录失败")
    }

    pub async fn latest_for_fund(&self, fund_id: i64) -> Result<Option<FundQuote>> {
        sqlx::query_as::<_, FundQuote>(
            r#"
            SELECT id, fund_id, unit_nav, nav_date, confirmed_change_rate, estimated_nav,
                   estimated_change_rate, estimated_at, change_rate, fetched_at, source, created_at
            FROM fund_quotes
            WHERE fund_id = ?
            ORDER BY fetched_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(fund_id)
        .fetch_optional(&self.pool)
        .await
        .context("查询基金最新行情失败")
    }

    pub async fn list_recent_for_fund(&self, fund_id: i64, limit: i64) -> Result<Vec<FundQuote>> {
        sqlx::query_as::<_, FundQuote>(
            r#"
            SELECT id, fund_id, unit_nav, nav_date, confirmed_change_rate, estimated_nav,
                   estimated_change_rate, estimated_at, change_rate, fetched_at, source, created_at
            FROM fund_quotes
            WHERE fund_id = ?
            ORDER BY fetched_at DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(fund_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("查询基金历史行情失败")
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<FundQuote>> {
        sqlx::query_as::<_, FundQuote>(
            r#"
            SELECT id, fund_id, unit_nav, nav_date, confirmed_change_rate, estimated_nav,
                   estimated_change_rate, estimated_at, change_rate, fetched_at, source, created_at
            FROM fund_quotes
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("按 ID 查询基金行情失败")
    }
}
