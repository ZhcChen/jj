use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: OffsetDateTime,
}
