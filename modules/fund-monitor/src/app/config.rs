use anyhow::{Context, Result, bail};
use std::env;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8866";
const DEFAULT_DATABASE_URL: &str = "sqlite://data/fund-monitor.db";
const DEFAULT_POLL_INTERVAL_SECONDS: u64 = 300;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: String,
    pub database_url: String,
    pub poll_interval_seconds: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let bind_addr = read_string("FUND_MONITOR_BIND_ADDR", DEFAULT_BIND_ADDR)?;
        let database_url = read_string("FUND_MONITOR_DATABASE_URL", DEFAULT_DATABASE_URL)?;
        let poll_interval_seconds = read_u64(
            "FUND_MONITOR_POLL_INTERVAL_SECONDS",
            DEFAULT_POLL_INTERVAL_SECONDS,
        )?;
        if poll_interval_seconds == 0 {
            bail!("FUND_MONITOR_POLL_INTERVAL_SECONDS 必须大于 0");
        }

        Ok(Self {
            bind_addr,
            database_url,
            poll_interval_seconds,
        })
    }
}

fn read_string(key: &str, default: &str) -> Result<String> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                bail!("{key} 不能为空");
            }
            Ok(trimmed.to_owned())
        }
        Err(env::VarError::NotPresent) => Ok(default.to_owned()),
        Err(err) => Err(err).with_context(|| format!("读取环境变量 {key} 失败")),
    }
}

fn read_u64(key: &str, default: u64) -> Result<u64> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                bail!("{key} 不能为空");
            }

            trimmed
                .parse::<u64>()
                .with_context(|| format!("{key} 必须是无符号整数"))
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(err) => Err(err).with_context(|| format!("读取环境变量 {key} 失败")),
    }
}
