//! `WalletExecutor` trait — the single wallet API consumed by all Uplink surfaces.
//!
//! The trait has two planned implementations:
//! 1. `NativeLdkWallet` (Phase A3) — `ldk-node`-backed native wallet for host-cli.
//! 2. `WasmLdkWallet` (Phase A4) — hand-assembled LDK for the browser (wasm32).
//!
//! The LSP is accessed through the executor; callers do not know whether payments
//! go peer-to-peer or are relayed through the Stable-Channels LSP.

use serde::{Deserialize, Serialize};
use crate::WalletError;

/// The result of a successful payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    /// Lightning payment preimage (proof of payment).
    pub preimage_hex: String,
    /// Actual millisatoshis paid (invoice amount + routing fee).
    pub total_msats_paid: u64,
    /// Idempotency key supplied by the caller.
    pub idempotency_key: String,
}

/// Minimal balance snapshot exposed to callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    /// Spendable Lightning balance in msats.
    pub lightning_msats: u64,
    /// Confirmed on-chain balance in satoshis.
    pub onchain_confirmed_sats: u64,
    /// USD equivalent of the Stable-Channel balance (populated by LSP; None if not connected).
    pub stable_channel_usd_cents: Option<u64>,
}

/// Core wallet operations needed by Uplink.
///
/// All methods are `async` and return `Result<_, WalletError>`.
/// Callers must supply an `idempotency_key` for all payment operations;
/// re-submitting the same key MUST return the original result without re-paying.
pub trait WalletExecutor: Send + Sync {
    /// Get the current balance snapshot.
    fn balance(&self) -> Result<WalletBalance, WalletError>;

    /// Generate a new BOLT11 invoice to receive `msats` with an optional memo.
    fn receive_invoice(&self, msats: u64, memo: &str) -> Result<String, WalletError>;

    /// Pay a BOLT11 invoice with an idempotency key and max fee budget.
    ///
    /// Returns the preimage on success. Idempotent: same key always returns same result.
    fn pay_invoice(
        &self,
        bolt11: &str,
        max_fee_msats: u64,
        idempotency_key: &str,
    ) -> Result<PaymentResult, WalletError>;

    /// Generate a Bitcoin on-chain receive address.
    fn receive_onchain_address(&self) -> Result<String, WalletError>;

    /// The LDK node's Lightning public key (hex).
    fn node_pubkey_hex(&self) -> String;
}

/// Placeholder implementation used during A0 scaffolding.
/// Replaced in A3 by `NativeLdkWallet`.
pub struct StubWallet;

impl WalletExecutor for StubWallet {
    fn balance(&self) -> Result<WalletBalance, WalletError> {
        Ok(WalletBalance {
            lightning_msats: 0,
            onchain_confirmed_sats: 0,
            stable_channel_usd_cents: None,
        })
    }

    fn receive_invoice(&self, _msats: u64, _memo: &str) -> Result<String, WalletError> {
        Err(WalletError::Ldk("stub: not implemented".into()))
    }

    fn pay_invoice(&self, _bolt11: &str, _max_fee_msats: u64, _idempotency_key: &str) -> Result<PaymentResult, WalletError> {
        Err(WalletError::Ldk("stub: not implemented".into()))
    }

    fn receive_onchain_address(&self) -> Result<String, WalletError> {
        Err(WalletError::Ldk("stub: not implemented".into()))
    }

    fn node_pubkey_hex(&self) -> String {
        "stub_node_pubkey".to_string()
    }
}
