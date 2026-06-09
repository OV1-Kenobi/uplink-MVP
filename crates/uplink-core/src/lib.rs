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

// ---------------------------------------------------------------------------
// wasm time provider
// ---------------------------------------------------------------------------
// `nostr` (via `universal-time`) fails at link time on wasm32-unknown-unknown
// unless a time provider is supplied, because `std::time` is unavailable there.
// On native/std targets `universal-time` uses `std::time` automatically and this
// module is not compiled. Supplied here (the single cdylib boundary) via
// `js_sys::Date`, mirroring the forced-link `getrandom` js shim.
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm_time {
    use core::time::Duration;
    use universal_time::{
        define_time_provider, Instant, MonotonicClock, SystemTime, WallClock,
    };

    struct WasmTimeProvider;

    impl WallClock for WasmTimeProvider {
        fn system_time(&self) -> SystemTime {
            SystemTime::from_unix_duration(Duration::from_secs_f64(js_sys::Date::now() / 1000.0))
        }
    }

    impl MonotonicClock for WasmTimeProvider {
        fn instant(&self) -> Instant {
            Instant::from_ticks(Duration::from_secs_f64(js_sys::Date::now() / 1000.0))
        }
    }

    // The macro emits the `#[export_name]` static `nostr` links against. We use it
    // (rather than a hand-written `#[export_name]`) because the attribute originates
    // from an external macro and so is permitted under `#![forbid(unsafe_code)]`.
    // The exported link name is derived from this crate's `CARGO_PKG_HOMEPAGE`, which
    // the manifest pins to universal-time's homepage so it matches the symbol name
    // universal-time bakes into its `extern` lookup. See the note in `Cargo.toml`.
    define_time_provider!(WasmTimeProvider);
}

// Re-export domain types for native consumers (host-cli, tests)
pub use uplink_identity::UplinkIdentity;
pub use uplink_accounts::{UplinkUser, UplinkWallet, SplitPaymentIntent};
pub use uplink_scheduler::Scheduler;
pub use uplink_nostr::kinds::{KIND_STABLE_STREAM, KIND_STABLE_STREAM_RECEIPT};
