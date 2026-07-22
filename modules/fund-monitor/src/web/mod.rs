use crate::{app::state::AppState, storage::db};
use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/"]
struct Asset;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/assets/{*path}", get(asset))
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    db::health_check(&state.pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    Ok("ok")
}

async fn index() -> Result<Html<String>, StatusCode> {
    let file = Asset::get("index.html").ok_or(StatusCode::NOT_FOUND)?;
    let html =
        String::from_utf8(file.data.into_owned()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

async fn asset(Path(path): Path<String>) -> Response {
    let normalized = path.trim_start_matches('/');

    match Asset::get(normalized) {
        Some(file) => (
            [(header::CONTENT_TYPE, content_type_for(normalized))],
            file.data.into_owned(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn content_type_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("html") => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    }
}
