use crate::domain::job_run::JobRun;
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct JobRepo {
    pool: SqlitePool,
}

impl JobRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn start(&self, job_type: &str) -> Result<JobRun> {
        let started_at = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            INSERT INTO job_runs (job_type, status, started_at, finished_at, error_message)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(job_type)
        .bind("running")
        .bind(started_at)
        .bind(Option::<OffsetDateTime>::None)
        .bind(Option::<String>::None)
        .execute(&self.pool)
        .await
        .context("创建任务执行记录失败")?;

        self.find_by_id(result.last_insert_rowid())
            .await?
            .context("创建任务执行记录后读取失败")
    }

    pub async fn finish(
        &self,
        id: i64,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<JobRun> {
        let finished_at = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE job_runs
            SET status = ?, finished_at = ?, error_message = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(finished_at)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新任务执行记录失败")?;

        self.find_by_id(id)
            .await?
            .context("更新任务执行记录后读取失败")
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<JobRun>> {
        sqlx::query_as::<_, JobRun>(
            r#"
            SELECT id, job_type, status, started_at, finished_at, error_message
            FROM job_runs
            ORDER BY started_at DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("查询最近任务执行记录失败")
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<JobRun>> {
        sqlx::query_as::<_, JobRun>(
            r#"
            SELECT id, job_type, status, started_at, finished_at, error_message
            FROM job_runs
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("按 ID 查询任务执行记录失败")
    }
}
