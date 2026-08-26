use alloy::consensus::ReceiptEnvelope;
use alloy::primitives::B256;
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;

use crate::{error::VerifyError, state::AppState, verify::verify_receipts_root};

#[derive(Serialize)]
pub struct VerifiedReceipts {
    pub block_number: u64,
    pub block_hash: B256,
    /// Always the value this service computed — never the header's claim.
    pub receipts_root: B256,
    pub anchor: Anchor,
    pub receipt_count: usize,
    pub receipts: Vec<TransactionReceipt>,
}

/// Phase 1 never anchors the header, and says so in every response.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    Unanchored,
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/v1/blocks/{block_number}/receipts",
        get(get_verified_receipts),
    )
}

async fn get_verified_receipts(
    State(state): State<AppState>,
    Path(block_number): Path<u64>,
) -> Result<Json<VerifiedReceipts>, VerifyError> {
    let block = state
        .provider
        .get_block_by_number(block_number.into())
        .await
        .map_err(VerifyError::upstream)?
        .ok_or(VerifyError::BlockNotFound(block_number))?;

    let receipts = state
        .provider
        .get_block_receipts(block_number.into())
        .await
        .map_err(VerifyError::upstream)?
        .ok_or(VerifyError::BlockNotFound(block_number))?;

    // Verification runs on the consensus form of each receipt (`inner`), with
    // RPC-only log metadata stripped down to the primitive logs that are
    // actually committed to the trie.
    let envelopes: Vec<ReceiptEnvelope> = receipts
        .iter()
        .map(|r| r.inner.clone().map_logs(|log| log.inner))
        .collect();

    let header = &block.header.inner;
    if header.number != block_number {
        return Err(VerifyError::upstream(format!(
            "asked for block {block_number}, provider answered with block {}",
            header.number
        )));
    }
    let computed = verify_receipts_root(header, &envelopes)?;

    Ok(Json(VerifiedReceipts {
        block_number: header.number,
        // hash_slow: recompute from the header fields we actually received,
        // rather than trusting the hash the provider reported.
        block_hash: header.hash_slow(),
        receipts_root: computed,
        anchor: Anchor::Unanchored,
        receipt_count: receipts.len(),
        receipts,
    }))
}
