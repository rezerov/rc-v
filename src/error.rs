use alloy::primitives::B256;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("block {0} not found")]
    BlockNotFound(u64),

    /// The one error that returns full detail: neither root is secret, and
    /// both are exactly what a caller needs to investigate.
    #[error("receipts root mismatch for block {block}")]
    RootMismatch {
        block: u64,
        expected: B256,
        computed: B256,
    },

    /// Transport / decode failures. The detail (which can contain the provider
    /// URL, and provider URLs carry API keys) is logged, never returned.
    #[error("upstream rpc failure")]
    Upstream { detail: String },
}

impl VerifyError {
    pub fn upstream(err: impl std::fmt::Display) -> Self {
        Self::Upstream {
            detail: err.to_string(),
        }
    }

    /// Stable, machine-readable identifier for each failure mode.
    pub fn code(&self) -> &'static str {
        match self {
            VerifyError::BlockNotFound(_) => "block_not_found",
            VerifyError::RootMismatch { .. } => "root_mismatch",
            VerifyError::Upstream { .. } => "upstream_error",
        }
    }
}

impl IntoResponse for VerifyError {
    /// Every error renders as the same envelope:
    ///
    /// ```json
    /// { "error": { "code": "...", "message": "...", "details": { ... } } }
    /// ```
    ///
    /// `code` is stable and meant for machines; `message` is for humans;
    /// `details` carries the structured facts (both roots on a mismatch).
    fn into_response(self) -> Response {
        let (status, details) = match &self {
            VerifyError::BlockNotFound(n) => (StatusCode::NOT_FOUND, json!({ "block_number": n })),
            // 502: the client did nothing wrong — an upstream server returned
            // data that fails verification.
            VerifyError::RootMismatch {
                block,
                expected,
                computed,
            } => {
                tracing::warn!(block, %expected, %computed, "receipts root mismatch");
                (
                    StatusCode::BAD_GATEWAY,
                    json!({
                        "block_number": block,
                        "header_receipts_root": expected,
                        "computed_receipts_root": computed,
                    }),
                )
            }
            VerifyError::Upstream { detail } => {
                tracing::error!(%detail, "upstream rpc failure");
                (StatusCode::BAD_GATEWAY, json!(null))
            }
        };

        let body = json!({
            "error": {
                "code": self.code(),
                "message": self.to_string(),
                "details": details,
            }
        });
        (status, Json(body)).into_response()
    }
}
