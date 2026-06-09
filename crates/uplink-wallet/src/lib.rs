//! # uplink-wallet
//!
//! Lightning + on-chain wallet for Uplink.
//!
//! ## Platform targets
//! | Feature | Target | Implementation |
//! |---|---|---|
//! | `native` | host-cli / desktop | `ldk-node` batteries-included wrapper |
//! | `wasm` | browser PWA | Hand-assembled LDK with IndexedDB + WS peer transport |
//!
//! ## LSP stub (ADR-U-002)
//! The `lsp` module is stubbed. The OpenAgents LSP (Stable-Channels) wire contract
//! will be designed alongside the LSP and pinned in ADR-U-002 before Phase A4 begins.
//!
//! ## Key custody (ADR-U-001)
//! `KeysManager` is seeded from `UplinkIdentity::ldk_node_seed` — a 32-byte value
//! derived under BIP-32 path `m/535348'/0'/account'`. The seed never leaves Rust.

#![forbid(unsafe_code)]

pub mod executor;
pub mod provider;
pub mod executor_provider;
pub mod lnc;
pub mod lsp;
#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "wasm")]
pub mod wasm;

pub mod error;

pub use error::WalletError;
pub use executor::{WalletExecutor, PaymentResult, WalletBalance};
pub use executor_provider::ExecutorProvider;
pub use lnc::LncProvider;
pub use provider::{
    Invoice, InvoiceStatus, ListTxParams, ProviderError, Transaction, TxKind,
    WalletCapabilities, WalletInfo, WalletProvider,
};
