//! Relay pool management — user-configurable whitelist, connect/disconnect, publish.

use nostr::Keys;
use thiserror::Error;

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

/// Relay pool handle (stub — implementation uses nostr-sdk Client).
///
/// Phase A2 will wire this to `nostr_sdk::Client` and the relay connection lifecycle.
pub struct RelayPool {
    config: RelayConfig,
    _keys: Keys,
}

impl RelayPool {
    pub fn new(config: RelayConfig, keys: Keys) -> Self {
        Self { config, _keys: keys }
    }

    pub fn config(&self) -> &RelayConfig {
        &self.config
    }

    pub fn add_relay(&mut self, url: String) {
        if !self.config.relays.contains(&url) {
            self.config.relays.push(url);
        }
    }

    pub fn remove_relay(&mut self, url: &str) {
        self.config.relays.retain(|r| r != url);
    }
}
