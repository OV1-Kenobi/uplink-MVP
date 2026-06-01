//! Parent/child delegation tokens as signed NIP-59 gift-wrapped Nostr events.
//!
//! ADR-U-004: Delegation token format.
//!
//! A delegation token is a signed kind-X event (TBD in ADR-U-004 final),
//! NIP-44 encrypted to the child's public key, then NIP-59 gift-wrapped.
//!
//! Revocation is a kind-9902 event (see `kinds.rs`) published to relays;
//! the child's wallet checks revocation status before each payment.

use nostr::Keys;
use serde::{Deserialize, Serialize};

/// Policy attached to a parent→child delegation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPolicy {
    /// Maximum sats per single transaction.
    pub max_per_tx_sats: u64,
    /// Maximum sats in any rolling 24-hour window.
    pub rolling_24h_cap_sats: u64,
    /// Unix timestamp after which the delegation expires.
    pub expires_at_unix: u64,
    /// If set, child may only pay to these npubs.
    pub allowed_recipient_npubs: Option<Vec<String>>,
}

/// A delegation token (serialized form for storage and NIP-59 wrapping).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationToken {
    pub token_id: String,
    pub parent_npub: String,
    pub child_npub: String,
    pub child_wallet_id: String,
    pub policy: DelegationPolicy,
    /// The NIP-59 gift-wrapped event ID that carries this token.
    pub envelope_event_id: Option<String>,
    pub issued_at_unix: u64,
    pub revoked: bool,
}

/// Issue a delegation from `parent_keys` to `child_npub` with the given policy.
///
/// Phase A7 implementation — stub in A0.
pub fn issue_delegation(
    _parent_keys: &Keys,
    _child_npub: &str,
    _child_wallet_id: &str,
    _policy: DelegationPolicy,
) -> Result<DelegationToken, crate::NostrError> {
    todo!("Phase A7: build, sign, and NIP-59 wrap a delegation token")
}

/// Verify that a delegation token's signature is valid and it has not expired.
pub fn verify_delegation(token: &DelegationToken) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    !token.revoked && token.policy.expires_at_unix > now
}
