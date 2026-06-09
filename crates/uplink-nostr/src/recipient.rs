//! Recipient resolution: NIP-05 / Lightning-address / LNURL-pay / npub / BOLT11.
//!
//! Resolves a human recipient into a payable BOLT11 invoice (ADR-U-007 §3). The HTTP
//! round-trips are behind the [`LnurlClient`] shim so the parsing + URL derivation are
//! unit-testable without a live network; the platform boundary (Tauri native / wasm)
//! supplies the concrete HTTP client.

use async_trait::async_trait;
use nostr::PublicKey;
use crate::NostrError;

/// A parsed payment recipient.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipientAddress {
    /// A Nostr public key (npub/nprofile/hex). Needs a profile fetch for its `lud16`.
    Npub(PublicKey),
    /// A Lightning address `user@domain` (LUD-16 / NIP-05-style).
    LightningAddress { user: String, domain: String },
    /// A bech32 `lnurl1…` LNURL-pay string.
    Lnurl(String),
    /// An already-payable BOLT11 invoice.
    Bolt11(String),
}

impl RecipientAddress {
    /// Parse free-form recipient input into a typed address.
    pub fn parse(input: &str) -> Result<Self, NostrError> {
        let s = input.trim();
        let lower = s.to_lowercase();
        if lower.starts_with("lnbc") || lower.starts_with("lntb")
            || lower.starts_with("lnbcrt") || lower.starts_with("lnsb")
        {
            return Ok(Self::Bolt11(s.to_string()));
        }
        if lower.starts_with("lnurl1") {
            return Ok(Self::Lnurl(s.to_string()));
        }
        if lower.starts_with("npub1") || lower.starts_with("nprofile1") {
            let pk = PublicKey::parse(s).map_err(|e| NostrError::Other(e.to_string()))?;
            return Ok(Self::Npub(pk));
        }
        if let Some((user, domain)) = s.split_once('@') {
            if !user.is_empty() && domain.contains('.') {
                return Ok(Self::LightningAddress {
                    user: user.to_string(),
                    domain: domain.to_string(),
                });
            }
        }
        if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            let pk = PublicKey::from_hex(s).map_err(|e| NostrError::Other(e.to_string()))?;
            return Ok(Self::Npub(pk));
        }
        Err(NostrError::Other(format!("unrecognized recipient: {s}")))
    }

    /// The LNURL-pay endpoint URL for this recipient, if directly derivable.
    ///
    /// `Npub` returns `None` (resolve its profile `lud16` first); `Bolt11` returns
    /// `None` (already an invoice).
    pub fn lnurlp_url(&self) -> Option<String> {
        match self {
            Self::LightningAddress { user, domain } => {
                Some(format!("https://{domain}/.well-known/lnurlp/{user}"))
            }
            Self::Lnurl(s) => {
                let (_hrp, bytes) = bech32::decode(s).ok()?;
                String::from_utf8(bytes).ok()
            }
            Self::Npub(_) | Self::Bolt11(_) => None,
        }
    }
}

/// HTTP shim used to fetch LNURL-pay documents and callbacks (one GET → body).
#[async_trait]
pub trait LnurlClient: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<String, NostrError>;
}

/// Resolve `recipient` into a payable BOLT11 invoice for `msats`.
///
/// `Bolt11` recipients are returned verbatim. All other forms run the LNURL-pay flow:
/// fetch the pay document, validate the amount, then fetch the callback invoice.
pub async fn resolve_invoice(
    client: &dyn LnurlClient,
    recipient: &RecipientAddress,
    msats: u64,
    comment: Option<&str>,
) -> Result<String, NostrError> {
    if let RecipientAddress::Bolt11(b) = recipient {
        return Ok(b.clone());
    }
    let url = recipient
        .lnurlp_url()
        .ok_or_else(|| NostrError::ZapResolution("recipient has no LNURL-pay endpoint".into()))?;

    let doc: serde_json::Value = serde_json::from_str(&client.fetch(&url).await?)
        .map_err(|e| NostrError::LnurlFetch(e.to_string()))?;

    let callback = doc.get("callback").and_then(|v| v.as_str())
        .ok_or_else(|| NostrError::LnurlFetch("missing callback".into()))?;
    let min = doc.get("minSendable").and_then(serde_json::Value::as_u64).unwrap_or(0);
    let max = doc.get("maxSendable").and_then(serde_json::Value::as_u64).unwrap_or(u64::MAX);
    if msats < min || msats > max {
        return Err(NostrError::ZapResolution(format!(
            "amount {msats} msats outside payable range [{min}, {max}]"
        )));
    }
    let comment_allowed = doc.get("commentAllowed").and_then(serde_json::Value::as_u64).unwrap_or(0);

    let sep = if callback.contains('?') { '&' } else { '?' };
    let mut cb = format!("{callback}{sep}amount={msats}");
    if let (Some(c), true) = (comment, comment_allowed > 0) {
        let trimmed: String = c.chars().take(comment_allowed as usize).collect();
        cb.push_str(&format!("&comment={trimmed}"));
    }

    let invoice_doc: serde_json::Value = serde_json::from_str(&client.fetch(&cb).await?)
        .map_err(|e| NostrError::LnurlFetch(e.to_string()))?;
    invoice_doc
        .get("pr")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| NostrError::LnurlFetch("callback returned no invoice (pr)".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lightning_address() {
        let r = RecipientAddress::parse("alice@example.com").unwrap();
        assert_eq!(
            r.lnurlp_url().as_deref(),
            Some("https://example.com/.well-known/lnurlp/alice")
        );
    }

    #[test]
    fn parses_bolt11_and_npub_and_rejects_garbage() {
        assert!(matches!(
            RecipientAddress::parse("lnbc100n1pjexample").unwrap(),
            RecipientAddress::Bolt11(_)
        ));
        use nostr::ToBech32;
        let npub = nostr::Keys::generate().public_key().to_bech32().unwrap();
        assert!(matches!(
            RecipientAddress::parse(&npub).unwrap(),
            RecipientAddress::Npub(_)
        ));
        assert!(RecipientAddress::parse("not a recipient").is_err());
    }

    struct MockClient;
    #[async_trait]
    impl LnurlClient for MockClient {
        async fn fetch(&self, url: &str) -> Result<String, NostrError> {
            if url.contains("/.well-known/lnurlp/") {
                Ok(r#"{"callback":"https://example.com/cb","minSendable":1000,"maxSendable":1000000000,"commentAllowed":0,"tag":"payRequest"}"#.into())
            } else if url.starts_with("https://example.com/cb?amount=") {
                Ok(r#"{"pr":"lnbc10n1pjresolved","routes":[]}"#.into())
            } else {
                Err(NostrError::LnurlFetch(format!("unexpected url: {url}")))
            }
        }
    }

    #[tokio::test]
    async fn resolve_invoice_runs_lnurl_pay_flow() {
        let r = RecipientAddress::parse("alice@example.com").unwrap();
        let inv = resolve_invoice(&MockClient, &r, 50_000, None).await.unwrap();
        assert_eq!(inv, "lnbc10n1pjresolved");
    }

    #[tokio::test]
    async fn resolve_invoice_rejects_out_of_range_amount() {
        let r = RecipientAddress::parse("alice@example.com").unwrap();
        assert!(resolve_invoice(&MockClient, &r, 1, None).await.is_err());
    }

    #[tokio::test]
    async fn bolt11_recipient_passes_through() {
        let r = RecipientAddress::Bolt11("lnbc1passthrough".into());
        let inv = resolve_invoice(&MockClient, &r, 50_000, None).await.unwrap();
        assert_eq!(inv, "lnbc1passthrough");
    }

    #[test]
    fn decodes_lnurl_bech32_to_url() {
        let lnurl = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
        let r = RecipientAddress::parse(lnurl).unwrap();
        assert_eq!(
            r.lnurlp_url().as_deref(),
            Some("https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b0ccd178df")
        );
    }
}
