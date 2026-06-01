//! Nostr receipt event builder for stable-stream payments (kind 9901).
//!
//! Full tag schema: docs/adr/ADR-U-003-receipt-event-kind.md
//!
//! Phase A0: only the canonical hash is implemented. Full Nostr event
//! construction (signing, tag building) is Phase A5.

use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::kinds::KIND_STABLE_STREAM_RECEIPT;

/// Canonical receipt for a single stable-stream period payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableStreamReceipt {
    pub stream_id: String,
    pub stream_event_id: String,
    pub recipient_npub: String,
    pub period_index: u64,
    pub msats_paid: u64,
    pub lsp_preimage_hex: String,
    pub paid_at_unix: u64,
}

impl StableStreamReceipt {
    /// Compute the canonical SHA-256 receipt hash.
    ///
    /// Input: `stream_id:period_index:msats_paid:lsp_preimage_hex`
    /// Mirrors OA `PaymentAttemptReceiptV1` canonicalization (ADR-0006).
    pub fn hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.stream_id.as_bytes());
        h.update(b":");
        h.update(self.period_index.to_string().as_bytes());
        h.update(b":");
        h.update(self.msats_paid.to_string().as_bytes());
        h.update(b":");
        h.update(self.lsp_preimage_hex.as_bytes());
        hex::encode(h.finalize())
    }

    /// Build a signed Nostr event (kind 9901) for this receipt.
    /// Phase A5 implementation.
    pub fn to_nostr_event(&self, keys: &nostr::Keys) -> Result<nostr::Event, crate::NostrError> {
        let event = EventBuilder::new(KIND_STABLE_STREAM_RECEIPT, "")
            .tag(Tag::parse(["e", &self.stream_event_id]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
            .tag(Tag::parse(["p", &self.recipient_npub]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
            .tag(Tag::parse(["amount", &self.msats_paid.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
            .tag(Tag::parse(["period_index", &self.period_index.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
            .tag(Tag::parse(["receipt_hash", &self.hash()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
            .tag(Tag::parse(["lsp_preimage", &self.lsp_preimage_hex]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
            .finalize(keys)
            .map_err(|e| crate::NostrError::Signing(e.to_string()))?;
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_hash_is_deterministic() {
        let r = StableStreamReceipt {
            stream_id: "abc123".into(),
            stream_event_id: "ev123".into(),
            recipient_npub: "npub1test".into(),
            period_index: 5,
            msats_paid: 100_000,
            lsp_preimage_hex: "deadbeef".into(),
            paid_at_unix: 1_700_000_000,
        };
        assert_eq!(r.hash(), r.hash()); // idempotent
    }
}
