use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct FundQuote {
    pub id: i64,
    pub fund_id: i64,
    pub unit_nav: Option<f64>,
    pub estimated_nav: Option<f64>,
    pub change_rate: Option<f64>,
    pub fetched_at: OffsetDateTime,
    pub source: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewFundQuote {
    pub fund_id: i64,
    pub unit_nav: Option<f64>,
    pub estimated_nav: Option<f64>,
    pub change_rate: Option<f64>,
    pub fetched_at: OffsetDateTime,
    pub source: String,
}
