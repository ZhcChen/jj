use crate::{
    app::{errors::FundIngestError, state::AppState},
    domain::{
        fund::{Fund, NewFund, UpdateFundMetadata},
        fund_quote::FundQuote,
    },
    providers::fund_source::fetch_and_store_fund_quote,
    storage::{fund_repo::FundRepo, job_repo::JobRepo, quote_repo::QuoteRepo},
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use time::{OffsetDateTime, UtcOffset};

use super::layout::render_html;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/funds", get(list_funds).post(create_fund))
        .route("/funds/{id}", get(show_fund).post(update_fund))
        .route("/funds/{id}/fetch", post(fetch_fund))
        .route("/funds/{id}/disable", post(disable_fund))
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FundFormInput {
    pub code: String,
    pub name: String,
    pub note: String,
    pub group_name: String,
    pub tags: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FundMetadataFormInput {
    pub name: String,
    pub note: String,
    pub group_name: String,
    pub tags: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FundDetailQuery {
    pub fetched: Option<String>,
}

#[derive(Debug, Clone)]
struct FundListItem {
    id: i64,
    code: String,
    name: String,
    note: String,
    group_name: String,
    tags: String,
}

#[derive(Debug, Clone)]
struct FundDetailView {
    id: i64,
    code: String,
    name: String,
    note: String,
    group_name: String,
    tags: String,
}

#[derive(Debug, Clone, Default)]
struct FundQuoteView {
    unit_nav: String,
    estimated_nav: String,
    change_rate: String,
    fetched_at: String,
    source: String,
}

#[derive(Template)]
#[template(path = "funds/index.html")]
struct FundsIndexTemplate {
    title: &'static str,
    funds: Vec<FundListItem>,
    has_error: bool,
    error_message: String,
    form: FundFormInput,
}

#[derive(Template)]
#[template(path = "funds/detail.html")]
struct FundDetailTemplate {
    title: &'static str,
    fund: FundDetailView,
    has_error: bool,
    error_message: String,
    has_notice: bool,
    notice_message: String,
    has_latest_quote: bool,
    latest_quote: FundQuoteView,
    quote_history: Vec<FundQuoteView>,
}

pub async fn list_funds(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());
    let funds = repo
        .list_active()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    render_html(&FundsIndexTemplate {
        title: "基金列表",
        funds: map_fund_list(funds),
        has_error: false,
        error_message: String::new(),
        form: FundFormInput::default(),
    })
}

pub async fn create_fund(
    State(state): State<AppState>,
    Form(form): Form<FundFormInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());

    let code = form.code.trim().to_owned();
    let name = form.name.trim().to_owned();
    let note = empty_to_none(&form.note);
    let group_name = empty_to_none(&form.group_name);
    let tags = empty_to_none(&form.tags);

    if code.is_empty() || name.is_empty() {
        let funds = repo
            .list_active()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let template = FundsIndexTemplate {
            title: "基金列表",
            funds: map_fund_list(funds),
            has_error: true,
            error_message: "基金代码和名称不能为空".to_owned(),
            form,
        };

        return Ok((StatusCode::BAD_REQUEST, render_html(&template)?).into_response());
    }

    if repo
        .find_by_code(&code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        let funds = repo
            .list_active()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let template = FundsIndexTemplate {
            title: "基金列表",
            funds: map_fund_list(funds),
            has_error: true,
            error_message: "基金代码已存在".to_owned(),
            form,
        };

        return Ok((StatusCode::BAD_REQUEST, render_html(&template)?).into_response());
    }

    repo.create(NewFund {
        code,
        name,
        note,
        group_name,
        tags,
        enabled: true,
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/funds").into_response())
}

pub async fn show_fund(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<FundDetailQuery>,
) -> Result<Response, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());
    let fund = repo
        .find_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    render_fund_detail_response(
        &state,
        fund,
        StatusCode::OK,
        None,
        query
            .fetched
            .map(|_| "已完成最近一次基金数据抓取".to_owned()),
    )
    .await
}

pub async fn update_fund(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<FundMetadataFormInput>,
) -> Result<Response, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());

    let current = repo
        .find_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let name = form.name.trim().to_owned();
    if name.is_empty() {
        let fund = Fund {
            id: current.id,
            code: current.code,
            name: form.name,
            note: empty_to_none(&form.note),
            group_name: empty_to_none(&form.group_name),
            tags: empty_to_none(&form.tags),
            enabled: current.enabled,
            created_at: current.created_at,
            updated_at: current.updated_at,
        };

        return render_fund_detail_response(
            &state,
            fund,
            StatusCode::BAD_REQUEST,
            Some("基金名称不能为空".to_owned()),
            None,
        )
        .await;
    }

    repo.update_metadata(
        id,
        UpdateFundMetadata {
            name,
            note: empty_to_none(&form.note),
            group_name: empty_to_none(&form.group_name),
            tags: empty_to_none(&form.tags),
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/funds/{id}")).into_response())
}

pub async fn fetch_fund(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, StatusCode> {
    let fund_repo = FundRepo::new(state.pool.clone());
    let fund = fund_repo
        .find_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let quote_repo = QuoteRepo::new(state.pool.clone());
    let job_repo = JobRepo::new(state.pool.clone());

    match fetch_and_store_fund_quote(state.fund_source.as_ref(), &fund, &quote_repo, &job_repo)
        .await
    {
        Ok(_) => Ok(Redirect::to(&format!("/funds/{id}?fetched=1")).into_response()),
        Err(err) => render_fetch_error(&state, fund, err).await,
    }
}

pub async fn disable_fund(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());

    repo.disable(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/funds").into_response())
}

fn map_fund_list(funds: Vec<Fund>) -> Vec<FundListItem> {
    funds
        .into_iter()
        .map(|fund| FundListItem {
            id: fund.id,
            code: fund.code,
            name: fund.name,
            note: fund.note.unwrap_or_default(),
            group_name: fund.group_name.unwrap_or_default(),
            tags: fund.tags.unwrap_or_default(),
        })
        .collect()
}

fn map_fund_detail(fund: Fund) -> FundDetailView {
    FundDetailView {
        id: fund.id,
        code: fund.code,
        name: fund.name,
        note: fund.note.unwrap_or_default(),
        group_name: fund.group_name.unwrap_or_default(),
        tags: fund.tags.unwrap_or_default(),
    }
}

fn map_quote_view(quote: FundQuote) -> FundQuoteView {
    FundQuoteView {
        unit_nav: format_optional_decimal(quote.unit_nav, 4, ""),
        estimated_nav: format_optional_decimal(quote.estimated_nav, 4, ""),
        change_rate: format_optional_decimal(quote.change_rate, 2, "%"),
        fetched_at: display_datetime(quote.fetched_at),
        source: quote.source,
    }
}

fn empty_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

async fn render_fetch_error(
    state: &AppState,
    fund: Fund,
    err: FundIngestError,
) -> Result<Response, StatusCode> {
    render_fund_detail_response(
        state,
        fund,
        err.status_code(),
        Some(err.user_message().to_owned()),
        None,
    )
    .await
}

async fn render_fund_detail_response(
    state: &AppState,
    fund: Fund,
    status: StatusCode,
    error_message: Option<String>,
    notice_message: Option<String>,
) -> Result<Response, StatusCode> {
    let template = build_fund_detail_template(state, fund, error_message, notice_message).await?;
    Ok((status, render_html(&template)?).into_response())
}

async fn build_fund_detail_template(
    state: &AppState,
    fund: Fund,
    error_message: Option<String>,
    notice_message: Option<String>,
) -> Result<FundDetailTemplate, StatusCode> {
    let quote_repo = QuoteRepo::new(state.pool.clone());
    let quote_history = quote_repo
        .list_recent_for_fund(fund.id, 10)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let latest_quote = quote_history.first().cloned();
    let quote_history = quote_history
        .into_iter()
        .map(map_quote_view)
        .collect::<Vec<_>>();

    Ok(FundDetailTemplate {
        title: "基金详情",
        fund: map_fund_detail(fund),
        has_error: error_message.is_some(),
        error_message: error_message.unwrap_or_default(),
        has_notice: notice_message.is_some(),
        notice_message: notice_message.unwrap_or_default(),
        has_latest_quote: latest_quote.is_some(),
        latest_quote: latest_quote.map(map_quote_view).unwrap_or_default(),
        quote_history,
    })
}

fn format_optional_decimal(value: Option<f64>, precision: usize, suffix: &str) -> String {
    match value {
        Some(value) => format!("{value:.precision$}{suffix}"),
        None => "-".to_owned(),
    }
}

fn display_datetime(value: OffsetDateTime) -> String {
    let offset = UtcOffset::from_hms(8, 0, 0).expect("valid Asia/Shanghai UTC offset");
    format!("{}", value.to_offset(offset))
}
