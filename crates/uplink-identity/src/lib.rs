//! # uplink-identity
//!
//! Derives a unified identity from a single BIP-39 mnemonic:
//!
//! | Key slot | BIP-32 path | Use |
//! |---|---|---|
//! | NIP-06 Nostr keypair | `m/44'/1237'/account'/0/0` | Nostr identity, DMs, zap auth |
//! | BIP-44 on-chain | `m/44'/0'/account'/0/0` | Bitcoin receive; migration from Spark |
//! | LDK node seed | `m/535348'/0'/account'` | `KeysManager` seed (32 bytes) |
//!
//! ADR: docs/adr/ADR-U-001-ldk-seed-derivation.md

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod credential;
pub mod derivation;
pub mod identity;
pub mod error;

pub use credential::{CredentialKind, CredentialMeta, ExternalCredential};
pub use identity::UplinkIdentity;
pub use error::IdentityError;
