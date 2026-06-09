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
/// The CDK nutzap path is not yet enabled. Returns [`NutzapError::NotEnabled`] rather
/// than panicking, honoring the no-`todo!()` invariant (AGENTS.md / ADR-U-007 §5). The
/// real mint/P2PK flow lands when the `cdk` feature is implemented in a later phase.
pub async fn send_nutzap(
    _recipient_npub_hex: &str,
    _msats: u64,
    _memo: Option<&str>,
) -> Result<String, NutzapError> {
    Err(NutzapError::NotEnabled)
}

#[derive(Debug, thiserror::Error)]
pub enum NutzapError {
    #[error("nutzap fallback not enabled (CDK integration pending)")]
    NotEnabled,
    #[error("mint resolution failed: {0}")]
    MintResolution(String),
    #[error("CDK error: {0}")]
    Cdk(String),
    #[error("publish error: {0}")]
    Publish(String),
}
