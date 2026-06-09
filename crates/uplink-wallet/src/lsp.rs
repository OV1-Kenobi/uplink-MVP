//! LSP client stub — Stable-Channels + OpenAgents LSP wire contract.
//!
//! ## Status: STUB (Phase A0)
//!
//! The concrete wire contract (BOLT-spec extensions, JIT-channel request format,
//! authentication scheme, Stable-Channels target-asset messages) will be pinned
//! in ADR-U-002 once the OpenAgents LSP design is finalized.
//!
//! The LSP team is designing the LSP to match this interface. Until the wire contract
//! is pinned (ADR-U-002), these entry points return `WalletError::Lsp` rather than
//! panicking — the no-`todo!()` invariant (AGENTS.md / ADR-U-007 §5) holds.
//!
//! ADR: docs/adr/ADR-U-002-lsp-wire-contract.md

/// Shared message for not-yet-available LSP entry points (ADR-U-002 pending).
const LSP_UNAVAILABLE: &str =
    "OpenAgents LSP wire contract not yet available (ADR-U-002 pending)";

use serde::{Deserialize, Serialize};
use crate::WalletError;

/// Stable-Channels balance from the LSP's perspective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableChannelBalance {
    /// USD-denominated target balance in cents.
    pub usd_target_cents: u64,
    /// Current Lightning balance backing the stable position in msats.
    pub backing_msats: u64,
    /// Whether the stable peg is currently active and within tolerance.
    pub peg_active: bool,
}

/// Request the LSP to open or top-up a JIT channel to this node.
///
/// Called when `wallet_balance().lightning_msats` is insufficient to accept
/// a payment and no inbound liquidity exists.
pub async fn request_jit_channel(
    _lsp_endpoint: &str,
    _our_node_pubkey: &str,
    _requested_capacity_sats: u64,
) -> Result<(), WalletError> {
    // Phase A4: implement BOLT-spec LSP channel-request message.
    Err(WalletError::Lsp(LSP_UNAVAILABLE.into()))
}

/// Credit a recipient's Stable-Channel balance by `msats` via the LSP.
///
/// This is the core primitive for streaming sats flows (kind 30901/9901).
/// The LSP applies the payment to the recipient's stable-channel position
/// instead of routing it as a standard BOLT11 payment.
pub async fn credit_stable_channel(
    _lsp_endpoint: &str,
    _recipient_node_pubkey: &str,
    _msats: u64,
    _idempotency_key: &str,
) -> Result<String, WalletError> {
    // Returns the payment preimage on success.
    // Phase A4: implement Stable-Channels credit message.
    Err(WalletError::Lsp(LSP_UNAVAILABLE.into()))
}

/// Query the current Stable-Channels balance for a node.
pub async fn get_stable_balance(
    _lsp_endpoint: &str,
    _node_pubkey: &str,
) -> Result<StableChannelBalance, WalletError> {
    // Phase A4: implement balance query.
    Err(WalletError::Lsp(LSP_UNAVAILABLE.into()))
}
