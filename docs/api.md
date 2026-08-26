# HTTP API

## `GET /v1/blocks/{block_number}/receipts`

Fetches the block header and receipts for `block_number` from the configured
RPC provider, recomputes the receipts trie root locally, and returns the
receipts **only if** the computed root matches the header's `receiptsRoot`.

### 200 — verified

```json
{
  "block_number": 19000042,
  "block_hash": "0x…",
  "receipts_root": "0x…",
  "anchor": "unanchored",
  "receipt_count": 187,
  "receipts": [ … ]
}
```

| Field | Meaning |
|---|---|
| `block_number` | Height of the verified block. |
| `block_hash` | Recomputed locally from the received header via `hash_slow()` — not echoed from the provider. |
| `receipts_root` | **Always the value this service computed.** Never the header's claim (on a 200 they are equal, by construction). |
| `anchor` | Always `"unanchored"` in phase 1: the header itself came from the same RPC and was not independently verified. |
| `receipt_count` | Number of receipts in the block. |
| `receipts` | Full RPC-shaped receipts (`transactionHash`, `logs`, `status`, …). |

Note that RPC-only metadata (`transactionHash`, `transactionIndex`,
`blockHash` inside each receipt) is **not** committed to the receipts trie and
is therefore not covered by the proof; the committed fields are status /
post-state root, cumulative gas, the logs bloom, and every log's address,
topics, and data.

### Errors

Every error uses one envelope:

```json
{
  "error": {
    "code": "<machine-readable>",
    "message": "<human-readable>",
    "details": { … }
  }
}
```

| HTTP | `code` | When | `details` |
|---|---|---|---|
| 404 | `block_not_found` | The provider has no such block. | `block_number` |
| 502 | `root_mismatch` | The recomputed root differs from the header's claim — the provider served corrupted or inconsistent data. | `block_number`, `header_receipts_root`, `computed_receipts_root` |
| 502 | `upstream_error` | Provider unreachable, non-JSON answer, undecodable payload, or it answered with the wrong block. | `null` — detail is logged server-side, never returned, because provider URLs carry API keys |

Example `root_mismatch` body:

```json
{
  "error": {
    "code": "root_mismatch",
    "message": "receipts root mismatch for block 261653",
    "details": {
      "block_number": 261653,
      "header_receipts_root": "0xebc8…e6ad",
      "computed_receipts_root": "0xd95b…84e4"
    }
  }
}
```

### Why 502 for a root mismatch?

Because the blame is upstream. `4xx` says "the client's request was wrong" —
it wasn't; the request was a well-formed ask for a real block. `500` says
"this service failed" — it didn't; it did its job, which is exactly to detect
that **an upstream server returned invalid data**. That is the textbook
definition of `502 Bad Gateway` ("the server, while acting as a gateway,
received an invalid response from an inbound server"). It also gives clients
one clean rule: `502` from this service means *don't trust this provider
response, retry or switch providers* — which is precisely the action a root
mismatch should trigger.

## Configuration

Read once at startup; the request path never touches the environment.

| Variable | Required | Default | Meaning |
|---|---|---|---|
| `RPC_URL` | yes | — | Ethereum JSON-RPC endpoint (scheme required, e.g. `https://…`). |
| `BIND_ADDR` | no | `127.0.0.1:3000` | Listen address. |

On missing or invalid configuration the process prints a readable message to
stderr and exits with status 1.
