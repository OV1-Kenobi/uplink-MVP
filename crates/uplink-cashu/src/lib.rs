//! # uplink-cashu
//!
//! Cashu eCash integration for Uplink — Phase A5 fallback payment path.
//!
//! ## Role
//! When a recipient's Nostr profile does NOT include an LSP feature bit
//! (i.e., they can't receive a Stable-Channel credit), Uplink falls back to:
//!   1. NIP-57 Lightning zap (if recipient has `lud16`/`lud06`)
//!   2. NIP-61 nutzap (Cashu P2PK-locked eCash token) — this crate
//!
//! ## Status
//! Stub in Phase A0. CDK integration begins in Phase A5.

#![forbid(unsafe_code)]

/// Send a NIP-61 nutzap to a recipient's npub.
///
/// Resolves the recipient's preferred Cashu mint from their kind-0 metadata,
/// mints a P2PK-locked token, and publishes a kind-9735-equivalent nutzap event.
///
/// Phase A5 implementation.
pub async fn send_nutzap(
    _recipient_npub_hex: &str,
    _msats: u64,
    _memo: Option<&str>,
) -> Result<String, NutzapError> {
    todo!("Phase A5: CDK nutzap send")
}

#[derive(Debug, thiserror::Error)]
pub enum NutzapError {
    #[error("mint resolution failed: {0}")]
    MintResolution(String),
    #[error("CDK error: {0}")]
    Cdk(String),
    #[error("publish error: {0}")]
    Publish(String),
}
