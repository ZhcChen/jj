use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct AlertEvent {
    pub id: i64,
    pub rule_id: i64,
    pub fund_id: i64,
    pub reason: String,
    pub status: String,
    pub triggered_at: OffsetDateTime,
    pub notification_result: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewAlertEvent {
    pub rule_id: i64,
    pub fund_id: i64,
    pub reason: String,
    pub status: String,
    pub triggered_at: OffsetDateTime,
    pub notification_result: Option<String>,
}
