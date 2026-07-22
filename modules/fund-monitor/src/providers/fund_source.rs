use crate::{
    app::errors::FundIngestError,
    domain::{
        fund::Fund,
        fund_quote::{FundQuote, NewFundQuote},
    },
    storage::{job_repo::JobRepo, quote_repo::QuoteRepo},
};
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use super::http_client::HttpClient;

const SOURCE_NAME: &str = "eastmoney/pingzhongdata";

static NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"var\s+fS_name\s*=\s*"([^"]+)";"#).expect("compile fS_name regex")
});
static CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"var\s+fS_code\s*=\s*"([^"]+)";"#).expect("compile fS_code regex")
});
static NET_WORTH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"var\s+Data_netWorthTrend\s*=\s*(\[[^;]*\]);"#)
        .expect("compile Data_netWorthTrend regex")
});
static ESTIMATED_NAV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"var\s+gsz\s*=\s*"([^"]+)";"#).expect("compile gsz regex"));
static ESTIMATED_CHANGE_RATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"var\s+gszzl\s*=\s*"([^"]+)";"#).expect("compile gszzl regex"));
static ESTIMATED_AT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"var\s+gztime\s*=\s*"([^"]+)";"#).expect("compile gztime regex"));

#[derive(Debug, Clone)]
pub struct FetchedFundSnapshot {
    pub code: String,
    pub confirmed_change_rate: Option<f64>,
    pub change_rate: Option<f64>,
    pub estimated_nav: Option<f64>,
    pub estimated_change_rate: Option<f64>,
    pub estimated_at: Option<OffsetDateTime>,
    pub fetched_at: OffsetDateTime,
    pub name: String,
    pub nav_date: Option<OffsetDateTime>,
    pub source: String,
    pub unit_nav: Option<f64>,
}

#[derive(Clone)]
pub struct EastmoneyFundSource {
    http_client: HttpClient,
}

impl EastmoneyFundSource {
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    pub async fn fetch_snapshot(
        &self,
        fund_code: &str,
    ) -> Result<FetchedFundSnapshot, FundIngestError> {
        let path = format!(
            "pingzhongdata/{fund_code}.js?v={}",
            OffsetDateTime::now_utc().unix_timestamp()
        );

        let body = self.http_client.get_text(&path).await.map_err(|err| {
            FundIngestError::source_unavailable(format!("请求基金数据源失败：{err:#}"))
        })?;

        parse_pingzhongdata(fund_code, &body)
    }
}

pub async fn fetch_and_store_fund_quote(
    source: &EastmoneyFundSource,
    fund: &Fund,
    quote_repo: &QuoteRepo,
    job_repo: &JobRepo,
) -> Result<FundQuote, FundIngestError> {
    let job_type = format!("fund_manual_fetch:{}", fund.code);
    fetch_and_store_fund_quote_for_job(source, fund, quote_repo, job_repo, &job_type).await
}

pub async fn fetch_and_store_fund_quote_for_job(
    source: &EastmoneyFundSource,
    fund: &Fund,
    quote_repo: &QuoteRepo,
    job_repo: &JobRepo,
    job_type: &str,
) -> Result<FundQuote, FundIngestError> {
    let job = job_repo.start(job_type).await.map_err(|err| {
        FundIngestError::storage_failure(format!("创建抓取任务记录失败：{err:#}"))
    })?;

    let snapshot = match source.fetch_snapshot(&fund.code).await {
        Ok(snapshot) => snapshot,
        Err(err) => {
            record_job_failure(job_repo, job.id, &err).await?;
            return Err(err);
        }
    };

    let quote = match quote_repo
        .insert(NewFundQuote {
            fund_id: fund.id,
            unit_nav: snapshot.unit_nav,
            nav_date: snapshot.nav_date,
            confirmed_change_rate: snapshot.confirmed_change_rate,
            estimated_nav: snapshot.estimated_nav,
            estimated_change_rate: snapshot.estimated_change_rate,
            estimated_at: snapshot.estimated_at,
            change_rate: snapshot.change_rate,
            fetched_at: snapshot.fetched_at,
            source: snapshot.source,
        })
        .await
    {
        Ok(quote) => quote,
        Err(err) => {
            let ingest_error =
                FundIngestError::storage_failure(format!("写入基金抓取结果失败：{err:#}"));
            record_job_failure(job_repo, job.id, &ingest_error).await?;
            return Err(ingest_error);
        }
    };

    job_repo
        .finish(job.id, "success", None)
        .await
        .map_err(|err| {
            FundIngestError::storage_failure(format!("更新抓取任务状态失败：{err:#}"))
        })?;

    Ok(quote)
}

fn parse_pingzhongdata(
    expected_code: &str,
    body: &str,
) -> Result<FetchedFundSnapshot, FundIngestError> {
    let fetched_at = OffsetDateTime::now_utc();
    let normalized = body.trim_start_matches('\u{feff}');
    let code = capture_required(&CODE_RE, normalized, "基金代码")?;
    if code != expected_code {
        return Err(FundIngestError::invalid_source_data(format!(
            "基金数据源返回的基金代码不匹配：期望 {expected_code}，实际 {code}"
        )));
    }

    let name = capture_required(&NAME_RE, normalized, "基金名称")?;
    let history_json = capture_required(&NET_WORTH_RE, normalized, "净值历史")?;
    let history: Vec<NetWorthPoint> = serde_json::from_str(&history_json).map_err(|err| {
        FundIngestError::invalid_source_data(format!("解析基金净值历史失败：{err}"))
    })?;

    let latest = history.last().ok_or_else(|| {
        FundIngestError::invalid_source_data("基金数据源未返回任何净值历史".to_owned())
    })?;

    let nav_date = OffsetDateTime::from_unix_timestamp(latest.timestamp_ms / 1_000).ok();
    let confirmed_change_rate = latest
        .equity_return
        .or_else(|| compute_nav_change_rate(&history));
    let estimated_nav = capture_optional_decimal(&ESTIMATED_NAV_RE, normalized);
    let estimated_change_rate = capture_optional_decimal(&ESTIMATED_CHANGE_RATE_RE, normalized);
    let estimated_at = capture_optional_local_datetime(&ESTIMATED_AT_RE, normalized);
    let change_rate = estimated_change_rate.or(confirmed_change_rate);

    Ok(FetchedFundSnapshot {
        code,
        confirmed_change_rate,
        change_rate,
        estimated_nav,
        estimated_change_rate,
        estimated_at,
        fetched_at,
        name,
        nav_date,
        source: SOURCE_NAME.to_owned(),
        unit_nav: Some(latest.y),
    })
}

async fn record_job_failure(
    job_repo: &JobRepo,
    job_id: i64,
    err: &FundIngestError,
) -> Result<(), FundIngestError> {
    job_repo
        .finish(job_id, "failed", Some(err.user_message()))
        .await
        .map_err(|finish_err| {
            FundIngestError::storage_failure(format!("记录抓取失败任务失败：{finish_err:#}"))
        })?;

    Ok(())
}

fn capture_required(
    regex: &Regex,
    body: &str,
    field_name: &str,
) -> Result<String, FundIngestError> {
    let value = regex
        .captures(body)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            FundIngestError::invalid_source_data(format!("基金数据源缺少必要字段：{field_name}"))
        })?;

    Ok(value)
}

fn capture_optional_decimal(regex: &Regex, body: &str) -> Option<f64> {
    regex
        .captures(body)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().trim().parse::<f64>().ok())
}

fn capture_optional_local_datetime(regex: &Regex, body: &str) -> Option<OffsetDateTime> {
    regex
        .captures(body)
        .and_then(|captures| captures.get(1))
        .and_then(|value| parse_eastmoney_local_datetime(value.as_str().trim()))
}

fn parse_eastmoney_local_datetime(value: &str) -> Option<OffsetDateTime> {
    let (date_part, time_part) = value.split_once(' ')?;
    let mut date_parts = date_part.split('-');
    let year = date_parts.next()?.parse::<i32>().ok()?;
    let month = date_parts.next()?.parse::<u8>().ok()?;
    let day = date_parts.next()?.parse::<u8>().ok()?;
    if date_parts.next().is_some() {
        return None;
    }

    let mut time_parts = time_part.split(':');
    let hour = time_parts.next()?.parse::<u8>().ok()?;
    let minute = time_parts.next()?.parse::<u8>().ok()?;
    let second = time_parts
        .next()
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(0);
    if time_parts.next().is_some() {
        return None;
    }

    let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;
    let time = Time::from_hms(hour, minute, second).ok()?;
    let offset = UtcOffset::from_hms(8, 0, 0).ok()?;

    Some(
        PrimitiveDateTime::new(date, time)
            .assume_offset(offset)
            .to_offset(UtcOffset::UTC),
    )
}

fn compute_nav_change_rate(history: &[NetWorthPoint]) -> Option<f64> {
    if history.len() < 2 {
        return None;
    }

    let previous = history.get(history.len().checked_sub(2)?)?;
    if previous.y == 0.0 {
        return None;
    }

    let latest = history.last()?;
    Some(((latest.y - previous.y) / previous.y) * 100.0)
}

#[derive(Debug, Clone, Deserialize)]
struct NetWorthPoint {
    #[serde(rename = "equityReturn")]
    equity_return: Option<f64>,
    #[serde(rename = "x")]
    timestamp_ms: i64,
    y: f64,
}
