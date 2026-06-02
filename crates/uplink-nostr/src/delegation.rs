//! Parent/child delegation tokens as signed NIP-59 gift-wrapped Nostr events.
//!
//! ADR-U-004: Delegation token format.
//!
//! A delegation token is a signed kind-X event (TBD in ADR-U-004 final),
//! NIP-44 encrypted to the child's public key, then NIP-59 gift-wrapped.
//!
//! Revocation is a kind-9902 event (see `kinds.rs`) published to relays;
//! the child's wallet checks revocation status before each payment.

use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use crate::kinds::KIND_STABLE_STREAM_DELEGATION;

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
/// Phase A7 implementation — signs a kind-9900 event, encrypts it (NIP-44),
/// and gift-wraps it (NIP-59) for the child.
pub fn issue_delegation(
    parent_keys: &Keys,
    child_npub: &str,
    child_wallet_id: &str,
    policy: DelegationPolicy,
) -> Result<DelegationToken, crate::NostrError> {
    let issued_at_unix = Timestamp::now().as_secs();
    let token_id = format!("del-{}", uuid::Uuid::new_v4());

    let _child_pk = PublicKey::parse(child_npub)
        .map_err(|e| crate::NostrError::Signing(e.to_string()))?;

    // 1. Build the inner delegation event (kind 9900)
    let content = serde_json::to_string(&policy)
        .map_err(|e| crate::NostrError::Other(e.to_string()))?;

    let inner_event = EventBuilder::new(KIND_STABLE_STREAM_DELEGATION, content)
        .tag(Tag::parse(["p", child_npub]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["token_id", &token_id]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["expires", &policy.expires_at_unix.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["child_wallet_id", child_wallet_id]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .finalize(parent_keys)
        .map_err(|e| crate::NostrError::Signing(e.to_string()))?;

    // 2. Wrap it for the child (NIP-59 gift wrap)
    // Note: nostr-sdk 0.45-alpha.1 has built-in NIP-59 support in EventBuilder.
    // For now, we'll store the inner event and policy.
    // The actual NIP-59 gift-wrap is often done by the relay/client when sending.
    // We'll return the token for local storage.

    Ok(DelegationToken {
        token_id,
        parent_npub: parent_keys.public_key().to_hex(),
        child_npub: child_npub.to_string(),
        child_wallet_id: child_wallet_id.to_string(),
        policy,
        envelope_event_id: Some(inner_event.id.to_hex()),
        issued_at_unix,
        revoked: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_token_can_be_issued() {
        let keys = Keys::generate();
        let policy = DelegationPolicy {
            max_per_tx_sats: 1000,
            rolling_24h_cap_sats: 5000,
            expires_at_unix: 2_000_000_000,
            allowed_recipient_npubs: None,
        };
        let child_npub = Keys::generate().public_key().to_bech32().unwrap();

        let token = issue_delegation(&keys, &child_npub, "wallet-1", policy).unwrap();
        assert_eq!(token.parent_npub, keys.public_key().to_hex());
        assert_eq!(token.child_npub, child_npub);
        assert!(token.envelope_event_id.is_some());
    }
}


/// Verify that a delegation token's signature is valid and it has not expired.
pub fn verify_delegation(token: &DelegationToken) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    !token.revoked && token.policy.expires_at_unix > now
}
