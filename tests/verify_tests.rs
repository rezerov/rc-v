//! Tests 1–5 from the PRD: the verification primitive against fixtures.

mod common;

use alloy::consensus::Eip658Value;
use alloy::consensus::proofs::calculate_receipt_root;
use receipt_verifier::error::VerifyError;
use receipt_verifier::verify::{compute_receipts_root, verify_receipts_root};
use serde_json::Value;

/// Test 1 — happy path: every fixture verifies, and the returned root equals
/// the header's `receipts_root`.
#[test]
fn happy_path_all_fixtures_verify() {
    for fx in common::load_all() {
        let envelopes = fx.envelopes();
        let header = &fx.block.header.inner;
        let root =
            verify_receipts_root(header, &envelopes).unwrap_or_else(|e| panic!("{}: {e}", fx.name));
        assert_eq!(root, header.receipts_root, "{}", fx.name);
    }
}

/// Test 2 — tamper: flip ONE byte in ONE log's data; verification MUST fail
/// with RootMismatch. This is the test that proves the code does anything.
#[test]
fn tampered_log_byte_is_caught() {
    for fx in common::load_all() {
        // Only fixtures that actually contain a log payload can be tampered.
        let has_log_data = fx.raw["receipts"].as_array().is_some_and(|rs| {
            rs.iter().any(|r| {
                r["logs"].as_array().is_some_and(|ls| {
                    ls.iter()
                        .any(|l| l["data"].as_str().is_some_and(|d| d.len() > 4))
                })
            })
        });
        if !has_log_data {
            continue;
        }

        let mut receipts_json: Value = fx.raw["receipts"].clone();
        common::tamper_one_log_byte(&mut receipts_json);
        let receipts: Vec<alloy::rpc::types::TransactionReceipt> =
            serde_json::from_value(receipts_json).expect("tampered receipts still decode");
        let envelopes = common::envelopes_of(&receipts);

        match verify_receipts_root(&fx.block.header.inner, &envelopes) {
            Err(VerifyError::RootMismatch {
                block,
                expected,
                computed,
            }) => {
                assert_eq!(block, fx.block.header.inner.number, "{}", fx.name);
                assert_eq!(expected, fx.block.header.inner.receipts_root, "{}", fx.name);
                assert_ne!(computed, expected, "{}", fx.name);
            }
            other => panic!(
                "{}: tampered data must produce RootMismatch, got {other:?}",
                fx.name
            ),
        }
    }
}

/// Test 3 — differential: our trie construction agrees with
/// `alloy_consensus::proofs::calculate_receipt_root` across all fixtures.
#[test]
fn differential_against_alloy_proofs() {
    for fx in common::load_all() {
        let envelopes = fx.envelopes();
        assert_eq!(
            compute_receipts_root(&envelopes),
            calculate_receipt_root(&envelopes),
            "{}",
            fx.name
        );
    }
}

/// Test 4 — pre-Byzantium: receipts carrying a post-state root (no status
/// byte) verify correctly.
#[test]
fn pre_byzantium_post_state_receipts_verify() {
    let fixtures: Vec<_> = common::load_all()
        .into_iter()
        .filter(|fx| fx.block.header.inner.number < 4_370_000)
        .collect();
    assert!(!fixtures.is_empty(), "no pre-Byzantium fixture present");

    for fx in fixtures {
        let envelopes = fx.envelopes();
        assert!(
            envelopes.iter().all(|e| matches!(
                e.as_receipt().map(|r| &r.status),
                Some(Eip658Value::PostState(_))
            )),
            "{}: pre-Byzantium fixture must carry post-state roots, not status bytes",
            fx.name
        );
        verify_receipts_root(&fx.block.header.inner, &envelopes)
            .unwrap_or_else(|e| panic!("{}: {e}", fx.name));
    }
}

/// Test 5 — typed transactions: a block containing EIP-1559 and EIP-4844
/// receipts verifies correctly.
#[test]
fn typed_transaction_block_verifies() {
    use alloy::consensus::Typed2718;
    let fixtures: Vec<_> = common::load_all()
        .into_iter()
        .filter(|fx| {
            let envs = fx.envelopes();
            envs.iter().any(|e| e.is_eip1559()) && envs.iter().any(|e| e.is_eip4844())
        })
        .collect();
    assert!(
        !fixtures.is_empty(),
        "no fixture with both EIP-1559 and EIP-4844 receipts present"
    );

    for fx in fixtures {
        verify_receipts_root(&fx.block.header.inner, &fx.envelopes())
            .unwrap_or_else(|e| panic!("{}: {e}", fx.name));
    }
}
