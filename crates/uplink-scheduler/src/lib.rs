//! # uplink-scheduler
//!
//! Recurring streaming-sats scheduler for Uplink.
//!
//! ## Design (wasm32-safe)
//! The scheduler has **no threads, no system clock access, no timers**.
//! It exposes a single `tick(now_unix: u64)` entry point.
//!
//! - On the browser: JS calls `wasm.tick(Date.now() / 1000)` on a `setInterval`.
//! - On host-cli: tokio schedules `tick()` on an interval.
//! - On service worker: `periodicSync` event triggers `tick()`.
//!
//! This means the same deterministic scheduler logic runs on all platforms.
//!
//! ## Idempotency
//! Each emitted `SplitPaymentIntent` carries a `(stream_id, period_index)` key.
//! Re-calling `tick()` with the same timestamp does not re-emit already-emitted intents.

#![forbid(unsafe_code)]

pub mod scheduler;
pub mod stream;
pub mod error;

pub use scheduler::Scheduler;
pub use stream::{StreamPolicy, StreamStatus};
pub use error::SchedulerError;
