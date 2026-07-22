use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Fund {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub note: Option<String>,
    pub group_name: Option<String>,
    pub tags: Option<String>,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewFund {
    pub code: String,
    pub name: String,
    pub note: Option<String>,
    pub group_name: Option<String>,
    pub tags: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateFundMetadata {
    pub name: String,
    pub note: Option<String>,
    pub group_name: Option<String>,
    pub tags: Option<String>,
}
