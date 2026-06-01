//! Extension trait — bounded capability modules on top of a Wallet.

use serde::{Deserialize, Serialize};

/// The kind of extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtensionKind {
    /// Recurring multi-leg streaming sats policy.
    SplitPayment,
    /// NIP-47 NWC grant — delegates Lightning to an external client (agent or app).
    NwcGrant {
        /// The NWC connection secret (never crosses the wasm boundary in plaintext).
        #[serde(skip)]
        connection_secret: Option<String>,
        /// Permitted methods (e.g. ["pay_invoice", "get_balance"]).
        permitted_methods: Vec<String>,
        /// Max spend per request in msats.
        max_per_request_msats: u64,
        /// Rolling 24h budget in msats.
        budget_24h_msats: u64,
        /// Unix expiry timestamp.
        expires_at_unix: u64,
    },
    /// Parent→child delegation link.
    ParentChildLink {
        parent_wallet_id: String,
        child_wallet_id: String,
    },
}

/// An installed extension on a wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub extension_id: String,
    pub wallet_id: String,
    pub kind: ExtensionKind,
    pub enabled: bool,
    pub created_at_unix: u64,
}
