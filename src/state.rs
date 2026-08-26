use std::sync::Arc;

use alloy::providers::{DynProvider, Provider, ProviderBuilder};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub provider: DynProvider,
    pub config: Arc<Config>,
}

impl AppState {
    /// Build the provider once; every request clones the cheap handle.
    pub fn new(config: Config) -> Self {
        let provider = ProviderBuilder::new()
            .connect_http(config.rpc_url.clone())
            .erased();
        Self {
            provider,
            config: Arc::new(config),
        }
    }
}
