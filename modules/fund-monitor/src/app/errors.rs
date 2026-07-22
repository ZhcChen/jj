use axum::http::StatusCode;
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
