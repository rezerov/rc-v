//! Test 6 — the demo: a mock provider serves a genuine header alongside
//! tampered receipts; the endpoint must answer 502 with both roots in the
//! body, and must never answer 200.

mod common;

use axum::{Json, Router, extract::State, routing::post};
use http_body_util::BodyExt;
use receipt_verifier::{api, config::Config, state::AppState};
use serde_json::Value;
use tower::ServiceExt;

#[derive(Clone)]
struct MockChain {
    block: Value,
    receipts: Value,
}

async fn mock_rpc(State(chain): State<MockChain>, Json(req): Json<Value>) -> Json<Value> {
    let id = req["id"].clone();
    let result = match req["method"].as_str() {
        Some("eth_getBlockByNumber") => chain.block.clone(),
        Some("eth_getBlockReceipts") => chain.receipts.clone(),
        other => panic!("mock server got unexpected method {other:?}"),
    };
    Json(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

#[tokio::test]
async fn tampered_receipts_from_live_endpoint_return_502_with_both_roots() {
    let fx = common::load("synthetic_typed_21000000");
    let block_number = fx.block.header.inner.number;
    let claimed_root = fx.block.header.inner.receipts_root;

    // Genuine header, tampered receipts — exactly what a lying provider sends.
    let mut receipts = fx.raw["receipts"].clone();
    common::tamper_one_log_byte(&mut receipts);
    let chain = MockChain {
        block: fx.raw["block"].clone(),
        receipts,
    };

    // Serve the mock on an ephemeral port; the verifier dials it over real HTTP.
    let mock = Router::new().route("/", post(mock_rpc)).with_state(chain);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock");
    let mock_url = format!("http://{}", listener.local_addr().expect("addr"));
    tokio::spawn(async move {
        axum::serve(listener, mock).await.expect("mock server");
    });

    let config = Config {
        rpc_url: mock_url.parse().expect("mock url"),
        bind_addr: "127.0.0.1:0".parse().expect("addr"),
    };
    let app = api::routes().with_state(AppState::new(config));

    let response = app
        .oneshot(
            axum::http::Request::get(format!("/v1/blocks/{block_number}/receipts"))
                .body(axum::body::Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");

    assert_eq!(
        response.status(),
        axum::http::StatusCode::BAD_GATEWAY,
        "tampered data must return 502, never 200"
    );

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: Value = serde_json::from_slice(&body).expect("json body");
    let error = &body["error"];

    assert_eq!(error["code"], "root_mismatch", "got: {error}");
    assert_eq!(
        error["details"]["block_number"], block_number,
        "got: {error}"
    );
    assert_eq!(
        error["details"]["header_receipts_root"],
        claimed_root.to_string(),
        "both roots must be in the body; got: {error}"
    );
    let computed = error["details"]["computed_receipts_root"]
        .as_str()
        .expect("computed root present");
    assert!(
        computed.starts_with("0x") && computed.len() == 66,
        "got: {error}"
    );
    assert_ne!(computed, claimed_root.to_string(), "got: {error}");
}

/// The honest case over the same live plumbing: untampered fixture → 200, and
/// the receipts_root in the body is the computed (== claimed) root.
#[tokio::test]
async fn honest_receipts_from_live_endpoint_return_200() {
    let fx = common::load("synthetic_typed_21000000");
    let block_number = fx.block.header.inner.number;
    let claimed_root = fx.block.header.inner.receipts_root;

    let chain = MockChain {
        block: fx.raw["block"].clone(),
        receipts: fx.raw["receipts"].clone(),
    };
    let mock = Router::new().route("/", post(mock_rpc)).with_state(chain);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock");
    let mock_url = format!("http://{}", listener.local_addr().expect("addr"));
    tokio::spawn(async move {
        axum::serve(listener, mock).await.expect("mock server");
    });

    let config = Config {
        rpc_url: mock_url.parse().expect("mock url"),
        bind_addr: "127.0.0.1:0".parse().expect("addr"),
    };
    let app = api::routes().with_state(AppState::new(config));

    let response = app
        .oneshot(
            axum::http::Request::get(format!("/v1/blocks/{block_number}/receipts"))
                .body(axum::body::Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(body["block_number"], block_number);
    assert_eq!(body["anchor"], "unanchored");
    assert_eq!(body["receipts_root"], claimed_root.to_string());
    assert_eq!(body["receipt_count"], fx.receipts.len());
    assert_eq!(
        body["receipts"].as_array().map(Vec::len),
        Some(fx.receipts.len())
    );
}
