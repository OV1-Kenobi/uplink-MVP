//! `UplinkWallet` — funds-bearing wallet handle.

use serde::{Deserialize, Serialize};

/// Role of a wallet within a user's account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletRole {
    /// The Admin wallet — full spend authority. One per user.
    Admin,
    /// An application wallet — scoped spend authority via capability tokens.
    Application { label: String },
}

/// Per-wallet capability tokens (mirrors LNbits' admin/invoice/read key partitioning).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCapabilityTokens {
    /// Grants full spend + configuration authority (admin equivalent).
    pub admin_token: String,
    /// Grants ability to generate receive invoices only.
    pub invoice_token: String,
    /// Grants balance and history read-only access.
    pub read_token: String,
}

/// A wallet within the Uplink account model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UplinkWallet {
    pub wallet_id: String,
    pub owner_user_id: String,
    pub role: WalletRole,
    /// BIP-32 account index used with `UplinkIdentity::derive_ldk_seed(account)`.
    pub ldk_account_index: u32,
    pub capability_tokens: WalletCapabilityTokens,
    /// IDs of active extensions on this wallet.
    pub extension_ids: Vec<String>,
    /// LSP node pubkey this wallet's channel is opened against (None = not yet connected).
    pub lsp_node_pubkey: Option<String>,
    pub created_at_unix: u64,
}

impl UplinkWallet {
    pub fn is_admin(&self) -> bool {
        matches!(self.role, WalletRole::Admin)
    }
}
