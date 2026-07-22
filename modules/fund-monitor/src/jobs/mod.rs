pub mod poll_funds;
pub mod scheduler;

use crate::app::state::AppState;
use tokio::task::JoinHandle;

pub fn start_scheduler(state: AppState) -> JoinHandle<()> {
    tokio::spawn(async move {
        scheduler::Scheduler::new(state).run_forever().await;
    })
}
