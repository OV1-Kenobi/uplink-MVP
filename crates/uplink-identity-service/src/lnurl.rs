//! LNURL-pay (LUD-06 / LUD-16) metadata + amount validation for hosted addresses (Phase 5b).
//!
//! Pure builders/validators for the pay document served at `/.well-known/lnurlp/<username>`.
//! Minting the callback invoice (`pr`) needs a live receive credential and happens at the
//! edge; this core only shapes the document and validates the requested amount.

use serde_json::{json, Value};

use crate::error::IdentityServiceError;

/// Default minimum payable amount (msats).
pub const DEFAULT_MIN_SENDABLE_MSAT: u64 = 1_000;
/// Default maximum payable amount (msats).
pub const DEFAULT_MAX_SENDABLE_MSAT: u64 = 1_000_000_000;

/// Build the LUD-06 pay document for `address` (`user@domain`) served at `callback`.
///
/// `metadata` is the LUD-06 stringified JSON array carrying the `text/identifier` and a
/// human-readable `text/plain` description.
#[must_use]
pub fn lnurlp_metadata(
    address: &str,
    callback: &str,
    min_msat: u64,
    max_msat: u64,
    comment_allowed: u64,
) -> Value {
    let metadata = json!([
        ["text/identifier", address],
        ["text/plain", format!("Pay to {address}")],
    ])
    .to_string();
    json!({
        "callback": callback,
        "minSendable": min_msat,
        "maxSendable": max_msat,
        "metadata": metadata,
        "commentAllowed": comment_allowed,
        "tag": "payRequest",
    })
}

/// Validate that `msat` is within the inclusive `[min_msat, max_msat]` range.
pub fn validate_amount(msat: u64, min_msat: u64, max_msat: u64) -> Result<(), IdentityServiceError> {
    if msat < min_msat || msat > max_msat {
        return Err(IdentityServiceError::AmountOutOfRange);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pay_document() {
        let doc = lnurlp_metadata(
            "alice@uplink.example",
            "https://uplink.example/lnurlp/alice/callback",
            DEFAULT_MIN_SENDABLE_MSAT,
            DEFAULT_MAX_SENDABLE_MSAT,
            0,
        );
        assert_eq!(doc["tag"], "payRequest");
        assert_eq!(doc["minSendable"], DEFAULT_MIN_SENDABLE_MSAT);
        assert_eq!(doc["maxSendable"], DEFAULT_MAX_SENDABLE_MSAT);
        // metadata is a stringified JSON array containing the identifier.
        let md = doc["metadata"].as_str().unwrap();
        assert!(md.contains("text/identifier"));
        assert!(md.contains("alice@uplink.example"));
    }

    #[test]
    fn validates_amount_range() {
        assert!(validate_amount(50_000, DEFAULT_MIN_SENDABLE_MSAT, DEFAULT_MAX_SENDABLE_MSAT).is_ok());
        assert!(validate_amount(1, DEFAULT_MIN_SENDABLE_MSAT, DEFAULT_MAX_SENDABLE_MSAT).is_err());
        assert!(validate_amount(u64::MAX, DEFAULT_MIN_SENDABLE_MSAT, DEFAULT_MAX_SENDABLE_MSAT).is_err());
    }
}
