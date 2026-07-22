use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/assets/{*path}", get(asset));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn healthz() -> impl IntoResponse {
    "ok"
}

async fn index() -> Result<Html<String>, StatusCode> {
    let file = Asset::get("index.html").ok_or(StatusCode::NOT_FOUND)?;
    let html = String::from_utf8(file.data.into_owned())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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

