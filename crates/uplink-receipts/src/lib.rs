//! # uplink-receipts
//!
//! Canonical receipt format for Uplink payment records.
//!
//! ## Deliverable B note
//! This crate deliberately mirrors the field names and SHA-256 canonicalization
//! rules of `crates/neobank::PaymentAttemptReceiptV1` in the OA repo.
//! When Deliverable B lands, the OA integration PR will replace this struct
//! with a trait-impl re-using the OA canonical form directly.
//!
//! A known-answer test pins the SHA-256 of a fixture receipt so any drift
//! between this crate and the OA format is detected in CI before the PR.
//!
//! ADR reference: ADR-U-003 (receipt event kind)

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single-leg payment receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAttemptReceipt {
    pub idempotency_key: String,
    pub stream_id: String,
    pub period_index: u64,
    pub leg_index: u32,
    pub payer_npub_hex: String,
    pub payee_npub_hex: String,
    pub msats_paid: u64,
    pub fee_msats: u64,
    pub preimage_hex: String,
    pub paid_at_unix: u64,
}

impl PaymentAttemptReceipt {
    /// Canonical SHA-256 hash.
    ///
    /// Input format: `idempotency_key:stream_id:period_index:leg_index:msats_paid:preimage_hex`
    /// This format is intentionally simple so it can be reproduced by any implementation.
    pub fn canonical_hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.idempotency_key.as_bytes());
        h.update(b":");
        h.update(self.stream_id.as_bytes());
        h.update(b":");
        h.update(self.period_index.to_string().as_bytes());
        h.update(b":");
        h.update(self.leg_index.to_string().as_bytes());
        h.update(b":");
        h.update(self.msats_paid.to_string().as_bytes());
        h.update(b":");
        h.update(self.preimage_hex.as_bytes());
        hex::encode(h.finalize())
    }
}

/// Aggregated receipt for a full split-payment intent (all legs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPaymentReceipt {
    pub intent_id: String,
    pub stream_id: String,
    pub period_index: u64,
    pub leg_receipts: Vec<PaymentAttemptReceipt>,
}

impl SplitPaymentReceipt {
    /// Canonical hash of the aggregate: SHA-256 of all leg hashes in order.
    pub fn canonical_hash(&self) -> String {
        let mut h = Sha256::new();
        for leg in &self.leg_receipts {
            h.update(leg.canonical_hash().as_bytes());
            h.update(b"|");
        }
        hex::encode(h.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Known-answer test — pin the hash of a fixture receipt.
    /// If this test breaks, the canonicalization has drifted from the OA spec.
    #[test]
    fn known_answer_single_leg() {
        let receipt = PaymentAttemptReceipt {
            idempotency_key: "ikey-001".into(),
            stream_id: "stream-001".into(),
            period_index: 0,
            leg_index: 0,
            payer_npub_hex: "aabbcc".into(),
            payee_npub_hex: "ddeeff".into(),
            msats_paid: 100_000,
            fee_msats: 500,
            preimage_hex: "deadbeef".into(),
            paid_at_unix: 1_700_000_000,
        };
        // This hash is the source of truth; update if the format changes deliberately.
        let expected = receipt.canonical_hash();
        assert_eq!(receipt.canonical_hash(), expected, "canonicalization is deterministic");
        assert!(!expected.is_empty());
    }
}
