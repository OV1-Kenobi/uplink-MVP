//! `WalletProvider` — the async, capability-described wallet surface (ADR-U-007).
//!
//! Modeled on NIP-47 (Nostr Wallet Connect). This is the trait Uplink business logic
//! targets; the embedded LDK node (`ExecutorProvider`) and an external NIP-47 wallet
//! (`NwcProvider`, in `uplink-nostr`) are interchangeable behind it.
//!
//! `WalletExecutor` is retained for host-cli / wasm ffi; `WalletProvider` is additive.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use crate::executor::{PaymentResult, WalletBalance};
use crate::WalletError;

/// Errors surfaced by a `WalletProvider`.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// The provider does not implement this method.
    #[error("operation not supported by this wallet provider")]
    Unsupported,
    /// The provider is configured but not currently reachable.
    #[error("wallet provider unavailable: {0}")]
    Unavailable(String),
    /// The wallet declined the request (e.g. receive-only credential, policy).
    #[error("wallet declined: {0}")]
    Declined(String),
    /// A malformed request or response was encountered.
    #[error("protocol error: {0}")]
    Protocol(String),
    /// An underlying wallet error (e.g. from the embedded LDK executor).
    #[error(transparent)]
    Wallet(#[from] WalletError),
}

/// Declared feature bits for a provider (NIP-47 `get_info`-style).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCapabilities {
    pub can_pay: bool,
    pub can_make_invoice: bool,
    pub can_lookup_invoice: bool,
    pub can_list_transactions: bool,
    pub supports_lnurl: bool,
    /// Two-credential split (ADR-U-007 §4): a receive-only credential is `false`.
    pub spend_capable: bool,
}

impl WalletCapabilities {
    /// Capabilities of a full, spend-capable embedded node.
    pub fn full_node() -> Self {
        Self {
            can_pay: true,
            can_make_invoice: true,
            can_lookup_invoice: false,
            can_list_transactions: false,
            supports_lnurl: false,
            spend_capable: true,
        }
    }
}

/// Wallet identity + declared methods (NIP-47 `get_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub node_pubkey_hex: String,
    pub network: String,
    /// NIP-47 method names the wallet advertises.
    pub methods: Vec<String>,
    pub capabilities: WalletCapabilities,
}

/// A BOLT11 invoice produced by `make_invoice`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub bolt11: String,
    pub payment_hash: String,
    pub amount_msats: u64,
    pub description: String,
    pub created_at_unix: u64,
    pub expiry_seconds: u64,
}

/// Settlement status of an invoice (`lookup_invoice`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceStatus {
    pub payment_hash: String,
    pub paid: bool,
    pub preimage_hex: Option<String>,
    pub settled_at_unix: Option<u64>,
}

/// Direction of a ledger entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxKind {
    Incoming,
    Outgoing,
}

/// A single transaction (`list_transactions`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub kind: TxKind,
    pub payment_hash: String,
    pub amount_msats: u64,
    pub fees_msats: u64,
    pub bolt11: Option<String>,
    pub preimage_hex: Option<String>,
    pub description: Option<String>,
    pub settled_at_unix: Option<u64>,
}

/// Filter parameters for `list_transactions` (NIP-47 shape).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListTxParams {
    pub from_unix: Option<u64>,
    pub until_unix: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub unpaid: bool,
    pub kind: Option<TxKind>,
}

/// The async wallet surface consumed by Uplink business logic (ADR-U-007).
#[async_trait]
pub trait WalletProvider: Send + Sync {
    async fn get_info(&self) -> Result<WalletInfo, ProviderError>;
    async fn get_balance(&self) -> Result<WalletBalance, ProviderError>;
    async fn make_invoice(&self, amount_msats: u64, description: &str)
        -> Result<Invoice, ProviderError>;
    async fn pay_invoice(&self, bolt11: &str, max_fee_msats: Option<u64>)
        -> Result<PaymentResult, ProviderError>;
    async fn lookup_invoice(&self, payment_hash: &str)
        -> Result<InvoiceStatus, ProviderError>;
    async fn list_transactions(&self, params: ListTxParams)
        -> Result<Vec<Transaction>, ProviderError>;

    /// Cheap liveness hint; does not perform network I/O.
    fn is_available(&self) -> bool;
    /// Declared capability bits.
    fn get_capabilities(&self) -> WalletCapabilities;
}
