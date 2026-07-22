mod alerts;
mod dashboard;
mod funds;
mod layout;
mod routes;
mod rules;
mod settings;

use crate::app::state::AppState;
use axum::Router;

pub fn router(state: AppState) -> Router {
    routes::router().with_state(state)
}
