use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct JobRun {
    pub id: i64,
    pub job_type: String,
    pub status: String,
    pub started_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
    pub error_message: Option<String>,
}
