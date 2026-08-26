//! receipt-verifier: fetches Ethereum block receipts from an untrusted RPC
//! provider and proves them against the block header's `receiptsRoot`.

pub mod api;
pub mod config;
pub mod error;
pub mod state;
pub mod verify;
