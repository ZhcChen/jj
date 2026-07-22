use crate::{app::state::AppState, jobs::poll_funds::PollFundsJob};
use std::{future::Future, time::Duration};
use tokio::time::{self, Instant, MissedTickBehavior};

pub struct Scheduler {
    interval: Duration,
    state: AppState,
}

impl Scheduler {
    pub fn new(state: AppState) -> Self {
        Self {
            interval: Duration::from_secs(state.config.poll_interval_seconds),
            state,
        }
    }

    pub async fn run_forever(self) {
        tracing::info!(
            poll_interval_seconds = self.interval.as_secs(),
            "fund poll scheduler started"
        );
        self.run_until(std::future::pending::<()>()).await;
    }

    pub async fn run_until<F>(self, shutdown: F)
    where
        F: Future<Output = ()>,
    {
        let mut ticker = time::interval_at(Instant::now() + self.interval, self.interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    break;
                }
                _ = ticker.tick() => {
                    tracing::info!(category = "poll", "fund poll scheduler tick triggered");
                    if let Err(err) = PollFundsJob::new(self.state.clone()).run_once().await {
                        tracing::error!(category = "poll", error = %format!("{err:#}"), "poll_funds scheduler tick failed");
                    }
                }
            }
        }
    }
}
