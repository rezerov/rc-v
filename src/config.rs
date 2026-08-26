use std::net::SocketAddr;

use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid value for {0}: {1}")]
    Invalid(&'static str, String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: Url,
    pub bind_addr: SocketAddr,
}

impl Config {
    /// Read configuration once at startup. Nothing else in the codebase is
    /// allowed to touch `std::env`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let raw = std::env::var("RPC_URL").map_err(|_| ConfigError::Missing("RPC_URL"))?;
        let rpc_url = raw.parse::<Url>().map_err(|e| {
            let mut detail = e.to_string();
            if !raw.contains("://") {
                detail.push_str(" — did you forget the http:// or https:// prefix?");
            }
            ConfigError::Invalid("RPC_URL", detail)
        })?;

        let bind_addr = match std::env::var("BIND_ADDR") {
            Ok(raw) => raw
                .parse()
                .map_err(|_| ConfigError::Invalid("BIND_ADDR", raw))?,
            Err(_) => SocketAddr::from(([127, 0, 0, 1], 3000)),
        };

        Ok(Self { rpc_url, bind_addr })
    }
}
