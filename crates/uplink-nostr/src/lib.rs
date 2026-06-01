//! # uplink-nostr
//!
//! Nostr connectivity layer for Uplink.
//!
//! ## Responsibilities
//! - Relay pool management (user-configurable whitelist)
//! - NIP-01 event publish / subscribe
//! - NIP-04 / NIP-44 encryption / decryption
//! - NIP-57 Lightning zap requests (pay-to-npub baseline path)
//! - NIP-59 gift-wrap for delegation + recovery messages
//! - Custom event kinds: 30901 (stable_stream) + 9901 (stable_stream_receipt)
//!
//! ADRs: ADR-U-003 (receipt event kind), ADR-U-004 (delegation tokens)

#![forbid(unsafe_code)]

pub mod relay;
pub mod kinds;
pub mod zap;
pub mod receipt;
pub mod delegation;
pub mod error;

pub use error::NostrError;
pub use kinds::{KIND_STABLE_STREAM, KIND_STABLE_STREAM_RECEIPT};
