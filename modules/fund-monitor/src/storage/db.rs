use anyhow::{Context, Result};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{fs, path::Path, str::FromStr};

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn initialize_database(database_url: &str) -> Result<SqlitePool> {
    ensure_sqlite_parent_dir(database_url)?;

    let options = SqliteConnectOptions::from_str(database_url)
        .with_context(|| format!("解析 SQLite 数据库地址失败: {database_url}"))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .with_context(|| format!("连接 SQLite 数据库失败: {database_url}"))?;

    MIGRATOR.run(&pool).await.context("执行数据库迁移失败")?;

    Ok(pool)
}

pub async fn health_check(pool: &SqlitePool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .context("数据库健康检查失败")?;
    Ok(())
}

fn ensure_sqlite_parent_dir(database_url: &str) -> Result<()> {
    let Some(path) = database_url.strip_prefix("sqlite://") else {
        return Ok(());
    };

    if path.is_empty() || path == ":memory:" || path.starts_with("file:") {
        return Ok(());
    }

    let db_path = Path::new(path);
    let Some(parent) = db_path.parent() else {
        return Ok(());
    };

    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    fs::create_dir_all(parent)
        .with_context(|| format!("创建 SQLite 数据目录失败: {}", parent.display()))?;

    Ok(())
}
