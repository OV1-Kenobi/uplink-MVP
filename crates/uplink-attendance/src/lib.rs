//! # uplink-attendance
//!
//! The **authoritative** attendance core (Phase 6, ADR-U-011): a pure, deterministic 7-step
//! inbound-tap validator plus the `WorkSession` toggle. It has **no I/O** — the relay /
//! Postgres edge injects the server-held [`SdmVerifier`](uplink_ntag424::SdmVerifier), the
//! [`TagDirectory`] snapshot, and the prior [`WorkSession`](uplink_scheduler::WorkSession),
//! then persists the returned [`TapAccepted`] outcome.
//!
//! Why a backend authority: a forwarded or replayed SDM URL must never open or extend a paid
//! session, and only the backend — not the client — decides in/out. Every check is
//! fail-closed and ordered (§5); transitions are pure functions of `(tap, prior state)`, so
//! the full session history can be replayed from retained raw events (AGENTS.md, §9).
//!
//! Custody is unchanged: this core gates and audits, it never holds spend keys, and reject
//! reasons carry only non-secret detail (§8).

#![forbid(unsafe_code)]

pub mod kinds;
pub mod outcome;
pub mod role;
pub mod tag;
pub mod validate;

pub use kinds::{
    kind_allowed_for_role, ATTENDANCE_ADMIN, ATTENDANCE_INTERVAL, ATTENDANCE_SCHEMA_VERSION,
    ATTENDANCE_SESSION, ATTENDANCE_TAP,
};
pub use outcome::{MalformedKind, RejectReason, TapAccepted, Transition};
pub use role::Role;
pub use tag::{OfficeTag, TagDirectory};
pub use validate::{evaluate_tap, TapEvaluationInput};
