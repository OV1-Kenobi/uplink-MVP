//! # uplink-core
//!
//! **The single wasm-bindgen boundary for Uplink.**
//!
//! ## Boundary rule (BOUNDARY.md)
//! TypeScript MUST NOT call any network API (fetch, WebSocket, EventSource) directly.
//! All network operations — Nostr relay connections, LSP WebSocket, LNURL-pay HTTP —
//! are performed inside this crate and exposed only as typed wasm-bindgen functions.
//!
//! ## Surface
//! The wasm-bindgen surface is organized by domain:
//! - `ffi::identity` — create/unlock/export identity
//! - `ffi::relay` — relay pool management
//! - `ffi::wallet` — balance, receive, pay
//! - `ffi::streams` — create/list/pause/resume streaming flows
//! - `ffi::contacts` — add/resolve npub contacts
//! - `ffi::accounts` — multi-user and parent/child delegation
//!
//! Each function in `ffi.rs` is documented in BOUNDARY.md.

#![forbid(unsafe_code)]

pub mod ffi;

// Re-export domain types for native consumers (host-cli, tests)
pub use uplink_identity::UplinkIdentity;
pub use uplink_accounts::{UplinkUser, UplinkWallet, SplitPaymentIntent};
pub use uplink_scheduler::Scheduler;
pub use uplink_nostr::kinds::{KIND_STABLE_STREAM, KIND_STABLE_STREAM_RECEIPT};
