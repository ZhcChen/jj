use askama::Template;
use axum::{http::StatusCode, response::Html};
use time::{OffsetDateTime, UtcOffset};

pub fn render_html<T: Template>(template: &T) -> Result<Html<String>, StatusCode> {
    template
        .render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn display_datetime(value: OffsetDateTime) -> String {
    let offset = UtcOffset::from_hms(8, 0, 0).expect("valid Asia/Shanghai UTC offset");
    let local = value.to_offset(offset);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        local.year(),
        local.month() as u8,
        local.day(),
        local.hour(),
        local.minute(),
        local.second()
    )
}
