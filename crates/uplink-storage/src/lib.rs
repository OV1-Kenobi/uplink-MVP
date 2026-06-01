//! # uplink-storage
//!
//! Encrypted key-value persistence layer for Uplink.
//!
//! ## Security model
//! All stored values are AES-256-GCM encrypted at rest. The encryption key
//! is a 32-byte value derived from the user's passphrase (Argon2id) and
//! stored nowhere — it must be re-derived at unlock time.
//!
//! ## Platform adapters
//! | Feature flag | Adapter | Used by |
//! |---|---|---|
//! | `native` | `sled` embedded DB | host-cli |
//! | `wasm` | IndexedDB via `web-sys` | browser PWA |
//!
//! ADR: docs/adr/ADR-U-005-key-recovery-otp-nostr.md (recovery threat model)

#![forbid(unsafe_code)]

pub mod crypto;
pub mod kv;

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "wasm")]
pub mod idb;

pub use kv::{KvStore, KvError};

/// Re-export the appropriate platform adapter under a unified name.
#[cfg(feature = "native")]
pub type PlatformStore = native::SledStore;

// wasm adapter:
#[cfg(feature = "wasm")]
pub type PlatformStore = idb::LocalStorageStore;
