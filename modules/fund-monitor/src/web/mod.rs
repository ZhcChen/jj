mod alerts;
mod funds;
mod layout;
mod routes;

use crate::app::state::AppState;
use axum::Router;

pub fn router(state: AppState) -> Router {
    routes::router().with_state(state)
}
