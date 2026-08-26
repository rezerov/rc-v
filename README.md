# receipt-verifier

Fetches Ethereum block receipts from an **untrusted** RPC provider and proves
them against the block header's `receiptsRoot`. If the recomputed root does not
match, the service returns an error instead of data. There is no success path
that returns unverified receipts.


**Verified:** the receipts. The service rebuilds the ordered Merkle-Patricia
trie mandated by the yellow paper (key = RLP of the receipt index, value = the
receipt's EIP-2718 encoding) and compares its own root against
`header.receipts_root`. A provider cannot alter a status, a log, gas usage, or
drop or reorder a receipt without changing that root.

**Not verified:** the header itself. In phase 1 the header comes from the same
RPC as the receipts, so a provider that fabricates a *consistent* header and
receipts pair would pass. This is known and accepted; every response says so
explicitly with `"anchor": "unanchored"`. Later phases anchor the header
against an independent source (checkpoint / sync committee).

Pre-Byzantium receipts (`root` field, block < 4,370,000) and all typed
receipts (EIP-2930/1559/4844) are handled.

## Run

```sh
cp .env.example .env      # fill in RPC_URL
cargo run                 # receipt-verifier is the default binary
curl localhost:3000/v1/blocks/19000042/receipts
```

The full HTTP contract, including the error envelope, is documented in
[docs/api.md](docs/api.md). API docs for the code itself: `cargo doc --open`.

## Tests

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

The suite runs offline against the JSON fixtures committed under
`tests/fixtures/` (saved RPC-shaped responses), covering: the happy path, a
single-byte tamper that must yield `root_mismatch`, a differential check
against `alloy_consensus::proofs::calculate_receipt_root`, a pre-Byzantium
block, typed (EIP-1559/4844) receipts, and an end-to-end integration test
where a mock provider serves a genuine header with tampered receipts and the
endpoint answers 502 with both roots.

CI (`.github/workflows/ci.yml`) runs formatting, clippy with warnings denied,
the test suite, and a docs build on every push and pull request — no secrets
required.

## Layout

```
src/
  main.rs           bootstrap: dotenv, tracing, config, banner, serve
  config.rs         Config::from_env — env is read exactly once, at startup
  state.rs          AppState { provider, config } — provider built once
  error.rs          VerifyError + the JSON error envelope
  api/blocks.rs     GET /v1/blocks/{n}/receipts
  verify/mod.rs     verify_receipts_root — the core primitive
```
