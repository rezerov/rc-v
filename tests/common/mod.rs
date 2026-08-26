#![allow(dead_code)] // shared by several test binaries; each uses a subset
//! Fixture loading shared by the test suite.
//!
//! A fixture is one JSON file: `{ "block": <eth_getBlockByNumber result>,
//! "receipts": <eth_getBlockReceipts result> }`, saved verbatim from an RPC
//! (or generated in the same wire shape by `examples/gen_synthetic_fixtures`).

use alloy::consensus::ReceiptEnvelope;
use alloy::rpc::types::{Block, TransactionReceipt};
use serde_json::Value;
use std::path::PathBuf;

pub struct Fixture {
    pub name: String,
    pub raw: Value,
    pub block: Block,
    pub receipts: Vec<TransactionReceipt>,
}

impl Fixture {
    pub fn envelopes(&self) -> Vec<ReceiptEnvelope> {
        envelopes_of(&self.receipts)
    }
}

pub fn envelopes_of(receipts: &[TransactionReceipt]) -> Vec<ReceiptEnvelope> {
    receipts
        .iter()
        .map(|r| r.inner.clone().map_logs(|log| log.inner))
        .collect()
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn load(name: &str) -> Fixture {
    let path = fixtures_dir().join(format!("{name}.json"));
    let raw: Value = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")),
    )
    .expect("fixture is valid JSON");
    parse(name.to_owned(), raw)
}

pub fn load_all() -> Vec<Fixture> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(fixtures_dir()).expect("tests/fixtures exists") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|e| e == "json") {
            let name = path
                .file_stem()
                .expect("stem")
                .to_string_lossy()
                .into_owned();
            let raw: Value =
                serde_json::from_str(&std::fs::read_to_string(&path).expect("read fixture"))
                    .expect("fixture is valid JSON");
            out.push(parse(name, raw));
        }
    }
    assert!(
        !out.is_empty(),
        "no fixtures found — run scripts/fetch_fixtures.sh or examples/gen_synthetic_fixtures"
    );
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn parse(name: String, raw: Value) -> Fixture {
    let block: Block = serde_json::from_value(raw["block"].clone())
        .unwrap_or_else(|e| panic!("{name}: block does not decode: {e}"));
    let receipts: Vec<TransactionReceipt> = serde_json::from_value(raw["receipts"].clone())
        .unwrap_or_else(|e| panic!("{name}: receipts do not decode: {e}"));
    Fixture {
        name,
        raw,
        block,
        receipts,
    }
}

/// Flip one byte in one log's `data` field, at the JSON level — the same kind
/// of corruption a lying provider would produce.
pub fn tamper_one_log_byte(receipts_json: &mut Value) {
    let receipts = receipts_json.as_array_mut().expect("receipts array");
    for receipt in receipts.iter_mut() {
        if let Some(logs) = receipt.pointer_mut("/logs").and_then(Value::as_array_mut) {
            for log in logs.iter_mut() {
                if let Some(data) = log.get("data").and_then(Value::as_str).map(String::from)
                    && data.len() > 4
                {
                    let mut bytes = data.into_bytes();
                    let i = bytes.len() - 1;
                    bytes[i] = if bytes[i] == b'0' { b'1' } else { b'0' };
                    log["data"] = Value::String(String::from_utf8(bytes).expect("utf8"));
                    return;
                }
            }
        }
    }
    panic!("no log with data found to tamper — fixture too bare");
}
