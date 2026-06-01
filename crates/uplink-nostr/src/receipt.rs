//! Nostr receipt event builder for stable-stream payments (kind 9901).
//!
//! Full tag schema: docs/adr/ADR-U-003-receipt-event-kind.md

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

    pub fn to_nostr_event(&self, keys: &nostr::Keys) -> Result<nostr::Event, crate::NostrError> {
        let receipt_hash = self.hash();
        let stream_eid = EventId::parse(&self.stream_event_id)
            .map_err(|e| crate::NostrError::Signing(e.to_string()))?;
        let recipient_pk = PublicKey::parse(&self.recipient_npub)
            .map_err(|e| crate::NostrError::Signing(e.to_string()))?;
        let content = format!(
            "Stable-stream receipt: {} msats paid for stream {} period {}",
            self.msats_paid, self.stream_id, self.period_index
        );
        let mk = |k: &str, v: &str| Tag::parse([k, v]).map_err(|e| crate::NostrError::Signing(e.to_string()));
        let event = EventBuilder::new(KIND_STABLE_STREAM_RECEIPT, content)
            .tag(mk("e", &stream_eid.to_hex())?)
            .tag(Tag::public_key(recipient_pk))
            .tag(mk("amount", &self.msats_paid.to_string())?)
            .tag(mk("period_index", &self.period_index.to_string())?)
            .tag(mk("receipt_hash", &receipt_hash)?)
            .tag(mk("lsp_preimage", &self.lsp_preimage_hex)?)
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
        assert_eq!(r.hash(), r.hash());
    }
}
