//! Nostr layer error type.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NostrError {
    #[error("relay error: {0}")]
    Relay(String),
    #[error("event signing error: {0}")]
    Signing(String),
    #[error("encryption error")]
    Encryption,
    #[error("NIP-57 zap resolution failed: {0}")]
    ZapResolution(String),
    #[error("LNURL fetch error: {0}")]
    LnurlFetch(String),
    #[error("{0}")]
    Other(String),
}
