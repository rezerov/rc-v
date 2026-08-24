use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("job {0} not found")]
    NotFound(u64),

    #[error("queue is full (max {max} jobs)")]
    QueueFull { max: usize },

    #[error("failed to persist job")]
    Io(#[from] std::io::Error),

    #[error("job state lock was poisoned")]
    LockPoisoned,

    #[error("failed to write job")]
    JobWriteError,
}

impl IntoResponse for JobError {
    fn into_response(self) -> Response {
        let status = match &self {
            JobError::NotFound(_) => StatusCode::NOT_FOUND,
            JobError::QueueFull { .. } => StatusCode::SERVICE_UNAVAILABLE,
            JobError::Io(_) | JobError::LockPoisoned => StatusCode::INTERNAL_SERVER_ERROR,
            JobError::JobWriteError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
