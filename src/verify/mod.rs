//! The verification primitive. Everything else in this service is a shell
//! around this function.

use alloy::consensus::{Header, ReceiptEnvelope};
use alloy::eips::eip2718::Encodable2718;
use alloy::primitives::B256;

use crate::error::VerifyError;

/// Recompute the receipts trie root from the receipts themselves and compare
/// it against the header's `receipts_root`.
///
/// The trie is the ordered Merkle-Patricia trie mandated by the yellow paper:
/// key = RLP of the receipt's index, value = the receipt's EIP-2718 encoding
/// (type-byte prefix for typed receipts, plain RLP for legacy).
/// `alloy_trie::root::ordered_trie_root_with_encoder` implements exactly that
/// keying scheme; we supply the EIP-2718 value encoding.
///
/// Pre-Byzantium receipts carry `Eip658Value::PostState(B256)` instead of a
/// status bool; `Encodable2718` for `ReceiptEnvelope` encodes both correctly,
/// so no status-byte assumption is made here.
///
/// On match, returns the *computed* root — callers must never echo the
/// header's claimed value.
pub fn verify_receipts_root(
    header: &Header,
    receipts: &[ReceiptEnvelope],
) -> Result<B256, VerifyError> {
    let computed = compute_receipts_root(receipts);

    if computed != header.receipts_root {
        return Err(VerifyError::RootMismatch {
            block: header.number,
            expected: header.receipts_root,
            computed,
        });
    }

    Ok(computed)
}

/// The trie computation on its own, with no comparison. Public so the test
/// suite can check it differentially against `alloy_consensus::proofs`.
pub fn compute_receipts_root(receipts: &[ReceiptEnvelope]) -> B256 {
    alloy_trie::root::ordered_trie_root_with_encoder(receipts, |receipt, out| {
        receipt.encode_2718(out)
    })
}
