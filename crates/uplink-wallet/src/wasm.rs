//! `WasmLdkWallet` — hand-assembled LDK for the browser (wasm32).
//!
//! This implementation uses:
//! - `web-sys` for WebSocket peer transport.
//! - `uplink-storage` for IndexedDB/localStorage persistence.
//! - `reqwest` (wasm-bindgen) for Esplora sync.

use std::sync::Arc;
use crate::executor::{WalletExecutor, WalletBalance, PaymentResult};
use crate::WalletError;
use uplink_identity::UplinkIdentity;

/// A hand-assembled LDK wallet for wasm.
pub struct WasmLdkWallet {
    // TODO: Add LDK components (ChannelManager, PeerManager, etc.)
}

impl WasmLdkWallet {
    /// Create a new Wasm LDK wallet from an identity.
    pub async fn new(
        _identity: &UplinkIdentity,
        _esplora_url: &str,
    ) -> Result<Self, WalletError> {
        // Phase A4: Hand-assemble LDK node components
        Ok(Self {})
    }
}

impl WalletExecutor for WasmLdkWallet {
    fn balance(&self) -> Result<WalletBalance, WalletError> {
        Ok(WalletBalance {
            lightning_msats: 0,
            onchain_confirmed_sats: 0,
            stable_channel_usd_cents: None,
        })
    }

    fn receive_invoice(&self, _msats: u64, _memo: &str) -> Result<String, WalletError> {
        Err(WalletError::Ldk("wasm: not implemented".into()))
    }

    fn pay_invoice(
        &self,
        _bolt11: &str,
        _max_fee_msats: u64,
        _idempotency_key: &str,
    ) -> Result<PaymentResult, WalletError> {
        Err(WalletError::Ldk("wasm: not implemented".into()))
    }

    fn receive_onchain_address(&self) -> Result<String, WalletError> {
        Err(WalletError::Ldk("wasm: not implemented".into()))
    }

    fn node_pubkey_hex(&self) -> String {
        "wasm_node_pubkey".to_string()
    }
}
