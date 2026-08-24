use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("Job not found")]
    NotFound(u64),

    #[error("job state lock was poisoned")]
    LockPoisoned,

    #[error("failed to write job")]
    JobWriteError,
}

impl IntoResponse for JobError {
    fn into_response(self) -> Response {
        let status = match &self {
            JobError::NotFound(_) => StatusCode::NOT_FOUND,
            JobError::LockPoisoned => StatusCode::INTERNAL_SERVER_ERROR,
            JobError::JobWriteError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
