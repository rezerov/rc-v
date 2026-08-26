use std::process::exit;

use receipt_verifier::api;
use receipt_verifier::config::Config;
use receipt_verifier::state;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "receipt_verifier=info,info".into()),
        )
        .init();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("configuration error: {e}");
        eprintln!("hint: copy .env.example to .env and fill in RPC_URL");
        exit(1);
    });

    print_banner(&config);

    let bind_addr = config.bind_addr;
    let state = state::AppState::new(config);
    let app = api::routes().with_state(state);

    let listener = match tokio::net::TcpListener::bind(bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("could not bind {bind_addr}: {e}");
            exit(1);
        }
    };

    tracing::info!("listening on {bind_addr}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("server error: {e}");
        exit(1);
    }
}

/// Startup banner: name, version, and where things point. The upstream is
/// shown as host only — provider URLs carry API keys and must never be
/// printed in full.
fn print_banner(config: &Config) {
    let title = format!("rc-v v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("  ┌{}┐", "─".repeat(48));
    println!("  │  {title:<46}│");
    println!("  └{}┘", "─".repeat(48));
    println!("  listening : {}", config.bind_addr);
    println!(
        "  upstream  : {}",
        config.rpc_url.host_str().unwrap_or("<unknown>")
    );
    println!();
}
