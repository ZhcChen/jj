use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct MonitorRule {
    pub id: i64,
    pub fund_id: Option<i64>,
    pub group_name: Option<String>,
    pub rule_type: String,
    pub threshold_config: String,
    pub enabled: bool,
    pub cooldown_minutes: i64,
    pub last_triggered_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewMonitorRule {
    pub fund_id: Option<i64>,
    pub group_name: Option<String>,
    pub rule_type: String,
    pub threshold_config: String,
    pub enabled: bool,
    pub cooldown_minutes: i64,
}
