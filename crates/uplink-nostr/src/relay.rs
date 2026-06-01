//! Relay pool management.

use thiserror::Error;
use nostr_sdk::prelude::*;

/// Default relays shipped with Uplink.
pub const DEFAULT_RELAYS: &[&str] = &[
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band",
    // OpenAgents-whitelisted relay (placeholder — replace with production URL):
    "wss://relay.openagents.com",
];

#[derive(Debug, Error)]
pub enum RelayError {
    #[error("relay connection failed: {0}")]
    Connection(String),
    #[error("publish failed: {0}")]
    Publish(String),
}

/// Configuration for the relay pool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayConfig {
    /// Currently active relay URLs.
    pub relays: Vec<String>,
    /// Primary relay for receipt events.
    pub primary_relay: Option<String>,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            relays: DEFAULT_RELAYS.iter().map(|s| s.to_string()).collect(),
            primary_relay: Some("wss://relay.openagents.com".to_string()),
        }
    }
}

/// Relay pool handle.
///
/// Wraps `nostr_sdk::Client` to provide Uplink-specific relay management.
pub struct RelayPool {
    config: RelayConfig,
    client: Client,
}

impl RelayPool {
    /// Initialize a new relay pool with the given config and keys.
    /// Does not connect until `connect()` is called.
    pub fn new(config: RelayConfig, _keys: &nostr::Keys) -> Self {
        // In 0.45.0-alpha.1, keys are passed during client creation if needed,
        // but for profile resolution they might not be strictly necessary if just fetching.
        // However, we'll use Client::default() or similar.
        let client = Client::default();
        Self { config, client }
    }

    /// Connect to all configured relays.
    pub async fn connect(&self) -> Result<(), RelayError> {
        for url in &self.config.relays {
            self.client.add_relay(url).await.map_err(|e| RelayError::Connection(e.to_string()))?;
        }
        self.client.connect().await;
        Ok(())
    }

    /// Add a relay to the config and the active pool.
    pub async fn add_relay(&mut self, url: String) -> Result<(), RelayError> {
        if !self.config.relays.contains(&url) {
            self.config.relays.push(url.clone());
            self.client.add_relay(url.as_str()).await.map_err(|e| RelayError::Connection(e.to_string()))?;
            self.client.connect_relay(url.as_str()).await.map_err(|e| RelayError::Connection(e.to_string()))?;
        }
        Ok(())
    }

    /// Remove a relay.
    pub async fn remove_relay(&mut self, url: &str) -> Result<(), RelayError> {
        self.config.relays.retain(|r| r != url);
        self.client.remove_relay(url).await.map_err(|e| RelayError::Connection(e.to_string()))?;
        Ok(())
    }

    /// Get the current config.
    pub fn config(&self) -> &RelayConfig {
        &self.config
    }

    /// Access the underlying nostr-sdk client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Clone the pool (handles underlying client cloning).
    pub fn clone_pool(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
        }
    }
}

impl Clone for RelayPool {
    fn clone(&self) -> Self {
        self.clone_pool()
    }
}

