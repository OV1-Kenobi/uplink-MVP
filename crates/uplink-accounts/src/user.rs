//! `UplinkUser` — top-level identity anchor for a human or agent account.

use serde::{Deserialize, Serialize};

/// A user in the Uplink multi-tenant model.
///
/// Holds a stable ID (derived from the NIP-06 public key) and a set of
/// wallet IDs. The actual identity keys live in `uplink-identity`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UplinkUser {
    /// Stable user ID: the hex of the NIP-06 public key.
    pub user_id: String,
    /// Human-readable display name (from kind-0 profile).
    pub display_name: Option<String>,
    /// IDs of wallets owned by this user (first is always the Admin wallet).
    pub wallet_ids: Vec<String>,
    /// Unix timestamp of account creation.
    pub created_at_unix: u64,
    /// Whether this user is a human or an automated agent.
    pub is_agent: bool,
}

impl UplinkUser {
    pub fn new(npub_hex: &str, is_agent: bool) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            user_id: npub_hex.to_string(),
            display_name: None,
            wallet_ids: Vec::new(),
            created_at_unix: now,
            is_agent,
        }
    }

    pub fn admin_wallet_id(&self) -> Option<&str> {
        self.wallet_ids.first().map(String::as_str)
    }
}
