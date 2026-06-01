//! NIP-57 Lightning zap request construction and LNURL resolution.
//!
//! Used as the fallback path when the recipient doesn't yet support
//! Stable-Channel streaming (i.e., no LSP feature bits in their profile).
//!
//! Phase A5 implementation — stub only in A0.

/// Resolve a recipient's Lightning address from their kind-0 metadata.
///
/// Returns `None` if the profile has no `lud16` or `lud06` field.
pub fn resolve_lightning_address(_profile_content: &str) -> Option<String> {
    // Phase A5: parse kind-0 JSON, extract lud16 / lud06
    todo!("Phase A5: parse kind-0 metadata for lud16/lud06")
}

/// Construct a NIP-57 zap request and LNURL-pay it, returning a BOLT11 invoice.
///
/// Returns the invoice string that should be paid via `uplink-wallet`.
pub async fn build_zap_invoice(
    _recipient_lud16: &str,
    _msats: u64,
    _comment: Option<&str>,
) -> Result<String, crate::NostrError> {
    // Phase A5: HTTP call to lud16 LNURL-pay endpoint, then invoice back
    todo!("Phase A5: LNURL-pay zap invoice")
}
