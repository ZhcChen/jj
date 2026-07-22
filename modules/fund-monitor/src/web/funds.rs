use crate::{
    app::state::AppState,
    domain::fund::{Fund, NewFund, UpdateFundMetadata},
    storage::fund_repo::FundRepo,
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;

use super::layout::render_html;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/funds", get(list_funds).post(create_fund))
        .route("/funds/{id}", get(show_fund).post(update_fund))
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
) -> Result<impl IntoResponse, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());
    let fund = repo
        .find_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    render_html(&FundDetailTemplate {
        title: "基金详情",
        fund: map_fund_detail(fund),
        has_error: false,
        error_message: String::new(),
    })
}

pub async fn update_fund(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<FundMetadataFormInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = FundRepo::new(state.pool.clone());

    let current = repo
        .find_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let name = form.name.trim().to_owned();
    if name.is_empty() {
        let template = FundDetailTemplate {
            title: "基金详情",
            fund: FundDetailView {
                id: current.id,
                code: current.code,
                name: form.name,
                note: form.note,
                group_name: form.group_name,
                tags: form.tags,
            },
            has_error: true,
            error_message: "基金名称不能为空".to_owned(),
        };

        return Ok((StatusCode::BAD_REQUEST, render_html(&template)?).into_response());
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

fn empty_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}
