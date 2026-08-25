use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;

use crate::{error::JobError, state::AppState};

#[derive(Serialize)]
pub struct VerifiedReceipts {
    pub block_number: u64,
    pub block_hash: u64,
    pub receipts_root: u64,
    pub anchor: Anchor,
    pub receipt_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    Unanchored, // Checkpoint / SyncCommittee join later
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/{block_number}", get(verify_handler))
}

#[allow(unused_variables)]
async fn verify_handler(
    State(state): State<AppState>,
    Path(block_number): Path<u64>,
) -> Result<Json<VerifiedReceipts>, JobError> {
    // Fetch block header from Rpc

    // Fetch block receips from Rpc

    // Compute root

    Ok(Json(VerifiedReceipts {
        block_number,
        block_hash: 0x1233123,
        receipts_root: 0x33123213,
        anchor: Anchor::Unanchored,
        receipt_count: 2,
    }))
}
