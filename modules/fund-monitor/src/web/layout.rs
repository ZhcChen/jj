use askama::Template;
use axum::{http::StatusCode, response::Html};

pub fn render_html<T: Template>(template: &T) -> Result<Html<String>, StatusCode> {
    template
        .render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
