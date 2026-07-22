use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub enum FundIngestError {
    SourceUnavailable(String),
    InvalidSourceData(String),
    StorageFailure(String),
}

impl FundIngestError {
    pub fn source_unavailable(message: impl Into<String>) -> Self {
        Self::SourceUnavailable(message.into())
    }

    pub fn invalid_source_data(message: impl Into<String>) -> Self {
        Self::InvalidSourceData(message.into())
    }

    pub fn storage_failure(message: impl Into<String>) -> Self {
        Self::StorageFailure(message.into())
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::SourceUnavailable(_) => StatusCode::BAD_GATEWAY,
            Self::InvalidSourceData(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::StorageFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn user_message(&self) -> &str {
        match self {
            Self::SourceUnavailable(message)
            | Self::InvalidSourceData(message)
            | Self::StorageFailure(message) => message,
        }
    }
}

impl fmt::Display for FundIngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.user_message())
    }
}

impl Error for FundIngestError {}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPageTemplate {
    title: String,
    status_code: u16,
    headline: String,
    message: String,
}

pub fn render_error_page(
    status: StatusCode,
    headline: impl Into<String>,
    message: impl Into<String>,
) -> Response {
    let template = ErrorPageTemplate {
        title: format!("{} {}", status.as_u16(), default_headline(status)),
        status_code: status.as_u16(),
        headline: headline.into(),
        message: message.into(),
    };

    match template.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(_) => (
            status,
            format!("{} - {}", default_headline(status), template.message),
        )
            .into_response(),
    }
}

pub fn render_internal_error(message: impl Into<String>) -> Response {
    render_error_page(StatusCode::INTERNAL_SERVER_ERROR, "页面暂时不可用", message)
}

pub fn render_not_found(message: impl Into<String>) -> Response {
    render_error_page(StatusCode::NOT_FOUND, "页面不存在", message)
}

pub fn render_bad_request(message: impl Into<String>) -> Response {
    render_error_page(StatusCode::BAD_REQUEST, "请求参数有误", message)
}

fn default_headline(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "Bad Request",
        StatusCode::NOT_FOUND => "Not Found",
        StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error",
        _ => "Request Failed",
    }
}
