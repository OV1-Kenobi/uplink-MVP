//! Validator outcomes: the accepted transition to persist, or a fail-closed reject reason
//! (ADR-U-011 §5). Reject reasons carry only non-secret detail (counters, static codes);
//! tag keys / writer secrets never appear here (§8).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use uplink_scheduler::WorkSession;

/// The authoritative session toggle produced by an accepted tap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transition {
    /// none/closed/suspended/auto-closed → a fresh `Open` session.
    Opened,
    /// open → `Closed` (normal clock-out).
    Closed,
}

/// Why a tap failed step 2 well-formedness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MalformedKind {
    /// Event kind is not allowed for the sender's role.
    WrongKind,
    /// The `v` schema version is unsupported.
    UnsupportedVersion,
    /// The SDM URL could not be parsed.
    UnparseableSdmUrl,
}

/// A fail-closed rejection of an inbound tap, in 7-step order (ADR-U-011 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum RejectReason {
    /// Step 1: sender is not an enrolled `worker`.
    #[error("not_authorized")]
    NotAuthorized,
    /// Step 2: malformed event (kind/version/SDM URL).
    #[error("malformed")]
    Malformed(MalformedKind),
    /// Step 3: SDM verification failed (MAC mismatch or undecodable PICCData).
    #[error("sdm_verify_failed")]
    SdmVerifyFailed,
    /// Step 4: the verified UID is not an enrolled office tag.
    #[error("unknown_tag")]
    UnknownTag,
    /// Step 5: read counter not strictly greater than last-seen (replay/forward).
    #[error("replayed_counter")]
    ReplayedCounter {
        /// The counter presented by this tap.
        seen: u32,
        /// The last accepted counter for the UID.
        last: u32,
    },
}

impl RejectReason {
    /// Stable, safe audit/client code for this rejection.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::NotAuthorized => "not_authorized",
            Self::Malformed(_) => "malformed",
            Self::SdmVerifyFailed => "sdm_verify_failed",
            Self::UnknownTag => "unknown_tag",
            Self::ReplayedCounter { .. } => "replayed_counter",
        }
    }
}

/// An accepted tap: the verified identity and the authoritative state the edge must persist
/// (step 7). The edge appends the raw event, upserts the session, advances `new_last_read_ctr`
/// in `office_tags`, and authors the backend `attendance_session` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapAccepted {
    /// The verified tag UID.
    pub uid: [u8; 7],
    /// The verified, accepted read counter.
    pub read_ctr: u32,
    /// The office this tag gates.
    pub office_id: String,
    /// Whether the tap opened or closed the session.
    pub transition: Transition,
    /// The authoritative `WorkSession` after the toggle.
    pub session: WorkSession,
    /// The counter to persist as the UID's new last-seen value (= `read_ctr`).
    pub new_last_read_ctr: u32,
}
