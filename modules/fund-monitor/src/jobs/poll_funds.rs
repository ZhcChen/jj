use crate::{
    app::state::AppState,
    providers::fund_source::fetch_and_store_fund_quote_for_job,
    storage::{fund_repo::FundRepo, job_repo::JobRepo, quote_repo::QuoteRepo},
};
use anyhow::{Context, Result};

const ROUND_JOB_TYPE: &str = "poll_funds";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollFundsSummary {
    pub total_funds: usize,
    pub succeeded_funds: usize,
    pub failed_funds: usize,
    pub status: String,
}

pub struct PollFundsJob {
    state: AppState,
}

impl PollFundsJob {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn run_once(&self) -> Result<PollFundsSummary> {
        let fund_repo = FundRepo::new(self.state.pool.clone());
        let quote_repo = QuoteRepo::new(self.state.pool.clone());
        let job_repo = JobRepo::new(self.state.pool.clone());

        let round_job = job_repo
            .start(ROUND_JOB_TYPE)
            .await
            .context("创建基金轮询任务记录失败")?;

        let funds = match fund_repo.list_active().await {
            Ok(funds) => funds,
            Err(err) => {
                finish_round_job(
                    &job_repo,
                    round_job.id,
                    "failed",
                    Some(format!("加载启用基金列表失败：{err:#}")),
                )
                .await?;
                return Err(err).context("查询启用基金列表失败");
            }
        };

        if funds.is_empty() {
            let summary = PollFundsSummary {
                total_funds: 0,
                succeeded_funds: 0,
                failed_funds: 0,
                status: "skipped".to_owned(),
            };

            finish_round_job(
                &job_repo,
                round_job.id,
                &summary.status,
                Some("当前没有启用基金，跳过抓取".to_owned()),
            )
            .await?;

            return Ok(summary);
        }

        let mut succeeded_funds = 0usize;
        let mut failed_funds = 0usize;
        let mut failure_messages = Vec::new();

        for fund in funds {
            let job_type = format!("fund_poll_fetch:{}", fund.code);
            match fetch_and_store_fund_quote_for_job(
                self.state.fund_source.as_ref(),
                &fund,
                &quote_repo,
                &job_repo,
                &job_type,
            )
            .await
            {
                Ok(_) => {
                    succeeded_funds += 1;
                }
                Err(err) => {
                    failed_funds += 1;
                    failure_messages.push(format!("{}: {}", fund.code, err.user_message()));
                }
            }
        }

        let summary = build_summary(succeeded_funds, failed_funds);
        finish_round_job(
            &job_repo,
            round_job.id,
            &summary.status,
            build_summary_message(&summary, &failure_messages),
        )
        .await?;

        Ok(summary)
    }
}

fn build_summary(succeeded_funds: usize, failed_funds: usize) -> PollFundsSummary {
    let total_funds = succeeded_funds + failed_funds;
    let status = if failed_funds == 0 {
        "success"
    } else if succeeded_funds == 0 {
        "failed"
    } else {
        "partial_success"
    };

    PollFundsSummary {
        total_funds,
        succeeded_funds,
        failed_funds,
        status: status.to_owned(),
    }
}

fn build_summary_message(
    summary: &PollFundsSummary,
    failure_messages: &[String],
) -> Option<String> {
    if failure_messages.is_empty() {
        return None;
    }

    Some(format!(
        "共 {} 只基金，成功 {}，失败 {}。{}",
        summary.total_funds,
        summary.succeeded_funds,
        summary.failed_funds,
        failure_messages.join("；")
    ))
}

async fn finish_round_job(
    job_repo: &JobRepo,
    job_id: i64,
    status: &str,
    message: Option<String>,
) -> Result<()> {
    job_repo
        .finish(job_id, status, message.as_deref())
        .await
        .with_context(|| format!("更新基金轮询任务记录失败，job_id={job_id}"))?;

    Ok(())
}
