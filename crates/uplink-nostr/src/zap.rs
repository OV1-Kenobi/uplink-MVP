//! NIP-57 Lightning zap request construction and LNURL resolution.
//!
//! Used as the fallback path when the recipient doesn't yet support
//! Stable-Channel streaming (i.e., no LSP feature bits in their profile).
//!
//! Recipient resolution lives in [`crate::recipient`] (ADR-U-007 §3); this module
//! provides the kind-0 → Lightning-address extraction and a thin zap-invoice helper.

use crate::recipient::{resolve_invoice, LnurlClient, RecipientAddress};
use crate::NostrError;

/// Resolve a recipient's Lightning address from their kind-0 metadata JSON.
///
/// Returns `lud16` if present, else `lud06` (a bech32 `lnurl1…`); `None` if neither.
pub fn resolve_lightning_address(profile_content: &str) -> Option<String> {
    let meta: serde_json::Value = serde_json::from_str(profile_content).ok()?;
    if let Some(lud16) = meta.get("lud16").and_then(|v| v.as_str()) {
        if !lud16.is_empty() {
            return Some(lud16.to_string());
        }
    }
    meta.get("lud06")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Resolve a Lightning address / LNURL into a payable BOLT11 invoice via LNURL-pay.
///
/// The HTTP round-trips run through the injected [`LnurlClient`] shim. The returned
/// invoice is paid via a [`uplink_wallet::WalletProvider`].
pub async fn build_zap_invoice(
    client: &dyn LnurlClient,
    recipient_lud16: &str,
    msats: u64,
    comment: Option<&str>,
) -> Result<String, NostrError> {
    let recipient = RecipientAddress::parse(recipient_lud16)?;
    resolve_invoice(client, &recipient, msats, comment).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_lud16_then_lud06() {
        let p = r#"{"name":"bob","lud16":"bob@example.com"}"#;
        assert_eq!(resolve_lightning_address(p).as_deref(), Some("bob@example.com"));
        let p2 = r#"{"name":"bob","lud16":"","lud06":"lnurl1xyz"}"#;
        assert_eq!(resolve_lightning_address(p2).as_deref(), Some("lnurl1xyz"));
        assert_eq!(resolve_lightning_address(r#"{"name":"bob"}"#), None);
    }
}
