//! chaos-proxy: a JSON-RPC pass-through that occasionally lies.
//!
//! Sits between the verifier and a real provider (e.g. Alchemy). Forwards
//! every request verbatim; on `eth_getBlockReceipts` responses it scrambles a
//! few bits with probability 1/CHAOS_RATE (default 1 in 10). Point the
//! verifier's RPC_URL at this proxy to watch verification catch the
//! corruption.
//!
//! Env:
//!   CHAOS_UPSTREAM_URL  required — the real provider
//!   CHAOS_BIND_ADDR     default 127.0.0.1:8547
//!   CHAOS_RATE          default 10 (corrupt 1 response in N)

use std::net::SocketAddr;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde_json::Value;

#[derive(Clone)]
struct Proxy {
    http: reqwest::Client,
    upstream: url::Url,
    rate: u64,
    served: Arc<AtomicU64>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chaos_proxy=info,info".into()),
        )
        .init();

    let upstream: url::Url = match std::env::var("CHAOS_UPSTREAM_URL") {
        Ok(raw) => match raw.parse() {
            Ok(u) => u,
            Err(e) => {
                eprintln!("invalid CHAOS_UPSTREAM_URL: {e}");
                exit(1);
            }
        },
        Err(_) => {
            eprintln!("missing CHAOS_UPSTREAM_URL (the real provider to wrap)");
            exit(1);
        }
    };

    let bind_addr: SocketAddr = std::env::var("CHAOS_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8547".to_owned())
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("invalid CHAOS_BIND_ADDR: {e}");
            exit(1);
        });

    let rate: u64 = std::env::var("CHAOS_RATE")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(10);

    let proxy = Proxy {
        http: reqwest::Client::new(),
        upstream,
        rate,
        served: Arc::new(AtomicU64::new(0)),
    };

    let app = Router::new().route("/", post(handle)).with_state(proxy);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("could not bind {bind_addr}: {e}");
            exit(1);
        });

    tracing::info!("chaos proxy on {bind_addr}, corrupting ~1 in {rate} receipt responses");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("server error: {e}");
        exit(1);
    }
}

async fn handle(
    State(proxy): State<Proxy>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_owned();

    let mut response: Value = proxy
        .http
        .post(proxy.upstream.clone())
        .json(&request)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")))?
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("upstream decode error: {e}"),
            )
        })?;

    // Only corrupt the receipt payloads: those are what the verifier proves.
    if method == "eth_getBlockReceipts" {
        let n = proxy.served.fetch_add(1, Ordering::Relaxed);
        // Always corrupt the very first receipts response? No — honest 1/N
        // coin flip, seeded per process by fastrand.
        let _ = n;
        if fastrand::u64(0..proxy.rate) == 0 {
            match scramble(&mut response) {
                Some(flips) => tracing::warn!(
                    method,
                    flips,
                    "CHAOS: scrambled bits in this response — verifier should return 502"
                ),
                None => tracing::info!(method, "CHAOS: wanted to corrupt but found no target"),
            }
        } else {
            tracing::info!(method, "passed through untouched");
        }
    }

    Ok(Json(response))
}

/// Flip a few bits inside hex values that are actually COMMITTED to the
/// receipts trie (log data, topics, blooms, cumulative gas). Corrupting
/// RPC-only metadata like `transactionHash` would go undetected — the trie
/// doesn't commit to it — which is itself a fact worth knowing, but a useless
/// demo. The payload stays valid JSON and valid hex; it is simply *wrong*.
fn scramble(response: &mut Value) -> Option<u32> {
    let mut pointers = Vec::new();
    if let Some(result) = response.get("result") {
        collect_hex_pointers(result, "/result".to_owned(), &mut pointers);
    }
    let committed = |p: &str| {
        (p.contains("/logs/") && (p.ends_with("/data") || p.contains("/topics/")))
            || p.ends_with("/logsBloom")
            || p.ends_with("/cumulativeGasUsed")
    };
    // Prefer log payloads; fall back to any committed field (blooms are
    // always present, even in a block with no logs).
    let logs: Vec<&String> = pointers
        .iter()
        .filter(|(p, _)| p.contains("/logs/") && p.ends_with("/data"))
        .map(|(p, _)| p)
        .collect();
    let pool: Vec<&String> = if logs.is_empty() {
        pointers
            .iter()
            .filter(|(p, _)| committed(p))
            .map(|(p, _)| p)
            .collect()
    } else {
        logs
    };
    if pool.is_empty() {
        return None;
    }

    let target = pool[fastrand::usize(0..pool.len())].clone();
    let value = response.pointer_mut(&target)?;
    let s = value.as_str()?.to_owned();
    let mut bytes: Vec<u8> = s.into_bytes();

    let mut flips = 0;
    for _ in 0..3 {
        // Never touch the "0x" prefix.
        let i = fastrand::usize(2..bytes.len());
        let old = bytes[i];
        let candidates = b"0123456789abcdef";
        let mut new = old;
        while new == old {
            new = candidates[fastrand::usize(0..16)];
        }
        bytes[i] = new;
        flips += 1;
    }

    *value = Value::String(String::from_utf8(bytes).ok()?);
    Some(flips)
}

fn collect_hex_pointers(value: &Value, path: String, out: &mut Vec<(String, usize)>) {
    match value {
        Value::String(s) if s.starts_with("0x") && s.len() > 4 => out.push((path, s.len())),
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                collect_hex_pointers(item, format!("{path}/{i}"), out);
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                collect_hex_pointers(v, format!("{path}/{k}"), out);
            }
        }
        _ => {}
    }
}
