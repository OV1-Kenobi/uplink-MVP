//! # uplink-accounts
//!
//! Multi-tenant account model for Uplink — inspired by LNbits but Rust-native.
//!
//! ## Hierarchy
//! ```text
//! User (identity anchor, BIP-39 mnemonic owner)
//!   └── Wallet (funds-bearing, one Admin + N application wallets)
//!         └── Extension (bounded capability module)
//!               ├── SplitPayment      — N-leg recurring payment dispatch
//!               ├── NwcGrant          — NIP-47 NWC delegation to external clients
//!               └── ParentChildLink   — delegates spend authority to child Wallets
//! ```
//!
//! ADRs: ADR-U-004 (delegation tokens), ADR-U-003 (receipt kinds)

#![forbid(unsafe_code)]

pub mod user;
pub mod wallet;
pub mod extension;
pub mod split_payment;
pub mod error;

pub use user::UplinkUser;
pub use wallet::{UplinkWallet, WalletRole};
pub use split_payment::{SplitPaymentIntent, SplitLeg};
pub use error::AccountError;
