//! Identity records, validated routing, and the injected persistence trait (Phase 5b).
//!
//! The store is **receive-only**: [`ReceiveRouting`] models only the non-secret routing used
//! to mint receive invoices (ADR-U-010 §2–3). Spend secrets never enter this crate; any
//! encrypted receive credential lives at the Postgres edge, outside the deterministic core.

use async_trait::async_trait;

use crate::error::IdentityServiceError;
use crate::username::NormalizedUsername;

/// A 64-character lowercase-hex Nostr public key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PubkeyHex(String);

impl PubkeyHex {
    /// Validate a 64-char lowercase-hex public key.
    pub fn parse(input: &str) -> Result<Self, IdentityServiceError> {
        let s = input.trim();
        let is_lower_hex =
            |b: u8| b.is_ascii_digit() || (b'a'..=b'f').contains(&b);
        if s.len() == 64 && s.bytes().all(is_lower_hex) {
            Ok(Self(s.to_string()))
        } else {
            Err(IdentityServiceError::PubkeyInvalid)
        }
    }

    /// The validated public key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Non-secret receive routing used to mint LNURL-pay invoices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiveRouting {
    /// A Lightning address `user@domain` (LUD-16).
    LightningAddress { user: String, domain: String },
    /// A bech32 `lnurl1…` LNURL-pay string.
    Lnurl(String),
}

/// A registered vanity identity (receive-only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRecord {
    /// Normalized username (NIP-05 name + LUD-16 local part).
    pub username: NormalizedUsername,
    /// The identity's Nostr public key (64-char lowercase hex).
    pub pubkey: PubkeyHex,
    /// Non-secret receive routing for LNURL-pay minting.
    pub routing: ReceiveRouting,
    /// Creation time (unix seconds).
    pub created_at: i64,
    /// Revocation time (unix seconds); `None` while live.
    pub revoked_at: Option<i64>,
}

impl IdentityRecord {
    /// Whether the identity is currently live (not revoked).
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.revoked_at.is_none()
    }
}

/// Injected persistence for identity records (implemented at the Postgres edge).
#[async_trait]
pub trait IdentityStore: Send + Sync {
    /// Fetch a record by normalized username, if any.
    async fn get_by_username(
        &self,
        username: &NormalizedUsername,
    ) -> Result<Option<IdentityRecord>, IdentityServiceError>;

    /// Insert a new identity record.
    async fn insert(&self, record: &IdentityRecord) -> Result<(), IdentityServiceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_pubkey_hex() {
        assert!(PubkeyHex::parse(&"a".repeat(64)).is_ok());
        assert!(PubkeyHex::parse("abcd").is_err()); // too short
        assert!(PubkeyHex::parse(&"A".repeat(64)).is_err()); // uppercase rejected
        assert!(PubkeyHex::parse(&"g".repeat(64)).is_err()); // non-hex
    }
}
