//! Stream declaration event builder for kind-30901 (stable-stream).
//!
//! Full tag schema: docs/adr/ADR-U-003-receipt-event-kind.md

use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use crate::kinds::KIND_STABLE_STREAM;

/// Configuration for a recurring streaming-sats flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDeclaration {
    /// Unique stream identifier (used as the 'd' tag for addressability).
    pub stream_id: String,
    /// Recipient's Nostr public key (hex).
    pub recipient_npub_hex: String,
    /// Amount in millisatoshis to pay per period.
    pub msats_per_period: u64,
    /// Period duration in seconds.
    pub period_seconds: u64,
    /// Target currency (always "USD" for Stable-Channel).
    pub currency: String,
    /// LSP Lightning node public key (hex).
    pub lsp_pubkey_hex: String,
    /// Unix timestamp of the first period.
    pub start_at_unix: u64,
    /// Optional: Unix timestamp when the stream ends.
    pub end_at_unix: Option<u64>,
    /// Optional: maximum total satoshis to pay (hard cap).
    pub max_total_sats: Option<u64>,
    /// Optional: human-readable memo.
    pub memo: Option<String>,
}

/// Build a signed Nostr event (kind 30901) for a stream declaration.
///
/// This is a parameterized replaceable event (NIP-33) addressed by the `d` tag.
pub fn build_stream_event(decl: &StreamDeclaration, keys: &Keys) -> Result<Event, crate::NostrError> {
    let mut builder = EventBuilder::new(KIND_STABLE_STREAM, "")
        .tag(Tag::parse(["d", &decl.stream_id]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["p", &decl.recipient_npub_hex]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["amount", &decl.msats_per_period.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["period", &decl.period_seconds.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["currency", &decl.currency]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["lsp", &decl.lsp_pubkey_hex]).map_err(|e| crate::NostrError::Signing(e.to_string()))?)
        .tag(Tag::parse(["start", &decl.start_at_unix.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?);

    // Optional tags
    if let Some(end) = decl.end_at_unix {
        builder = builder.tag(Tag::parse(["end", &end.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?);
    }
    if let Some(max_sats) = decl.max_total_sats {
        builder = builder.tag(Tag::parse(["max_total_sats", &max_sats.to_string()]).map_err(|e| crate::NostrError::Signing(e.to_string()))?);
    }
    if let Some(memo) = &decl.memo {
        builder = builder.tag(Tag::parse(["memo", memo]).map_err(|e| crate::NostrError::Signing(e.to_string()))?);
    }

    let event = builder
        .finalize(keys)
        .map_err(|e| crate::NostrError::Signing(e.to_string()))?;

    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_declaration_builds_event() {
        // Generate test keys
        let keys = Keys::generate();
        
        let decl = StreamDeclaration {
            stream_id: "test-stream-123".into(),
            recipient_npub_hex: "deadbeef".into(),
            msats_per_period: 100_000,
            period_seconds: 86400,
            currency: "USD".into(),
            lsp_pubkey_hex: "stub_lsp".into(),
            start_at_unix: 1_700_000_000,
            end_at_unix: None,
            max_total_sats: None,
            memo: Some("Test stream".into()),
        };

        let event = build_stream_event(&decl, &keys).unwrap();
        assert_eq!(event.kind(), KIND_STABLE_STREAM);
        
        // Verify required tags are present
        let tags = event.tags();
        assert!(tags.iter().any(|t| t.as_slice().get(0).map(|s| s.as_str()) == Some("d")));
        assert!(tags.iter().any(|t| t.as_slice().get(0).map(|s| s.as_str()) == Some("p")));
        assert!(tags.iter().any(|t| t.as_slice().get(0).map(|s| s.as_str()) == Some("amount")));
    }
}
