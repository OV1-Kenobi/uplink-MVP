//! Work-session model for session-gated in-office streaming (ADR-U-008).
//!
//! A `WorkSession` represents a worker's clocked-in interval for one stream.
//! Only an `Open` session makes that stream's in-office intervals payable; any
//! other status (or no session) blocks payout. Session transitions are driven by
//! callers — NFC taps (Phase 4) and the backend attendance state machine
//! (Phase 6). This module is the deterministic, replayable core they build on.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a work session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Worker is clocked in; in-office intervals are payable.
    Open,
    /// Normal clock-out; no further payouts.
    Closed,
    /// Presence lost (e.g. geofence exit) within a grace window; not payable.
    Suspended,
    /// System-closed (max duration / missing close tap); not payable.
    AutoClosed,
}

/// A worker's clocked-in interval gating one stream's in-office payouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    /// Stable ID (UUID hex) stamped onto every interval this session authorizes.
    pub session_id: String,
    /// The in-office stream this session gates.
    pub stream_id: String,
    /// Unix timestamp the session opened.
    pub opened_at_unix: u64,
    /// Unix timestamp the session closed (set for `Closed` / `AutoClosed`).
    pub closed_at_unix: Option<u64>,
    pub status: SessionStatus,
}

impl WorkSession {
    /// Open a fresh session for `stream_id`.
    pub fn open(session_id: impl Into<String>, stream_id: impl Into<String>, now_unix: u64) -> Self {
        Self {
            session_id: session_id.into(),
            stream_id: stream_id.into(),
            opened_at_unix: now_unix,
            closed_at_unix: None,
            status: SessionStatus::Open,
        }
    }

    /// True only while the session is `Open` (the sole payout-gating state).
    pub fn is_open(&self) -> bool {
        self.status == SessionStatus::Open
    }

    /// Normal clock-out.
    pub fn close(&mut self, now_unix: u64) {
        self.status = SessionStatus::Closed;
        self.closed_at_unix = Some(now_unix);
    }

    /// Suspend on presence loss (grace window); leaves `closed_at_unix` unset so
    /// the session can be resumed by re-opening.
    pub fn suspend(&mut self) {
        self.status = SessionStatus::Suspended;
    }

    /// System-initiated close (max duration / missing close tap).
    pub fn auto_close(&mut self, now_unix: u64) {
        self.status = SessionStatus::AutoClosed;
        self.closed_at_unix = Some(now_unix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_session_is_payable() {
        let s = WorkSession::open("sess-1", "stream-1", 1_000_000);
        assert!(s.is_open());
        assert_eq!(s.closed_at_unix, None);
    }

    #[test]
    fn close_suspend_auto_close_block_payout() {
        let mut s = WorkSession::open("sess-1", "stream-1", 1_000_000);

        s.suspend();
        assert!(!s.is_open());
        assert_eq!(s.status, SessionStatus::Suspended);

        let mut s2 = WorkSession::open("sess-2", "stream-1", 1_000_000);
        s2.close(1_000_500);
        assert!(!s2.is_open());
        assert_eq!(s2.closed_at_unix, Some(1_000_500));

        let mut s3 = WorkSession::open("sess-3", "stream-1", 1_000_000);
        s3.auto_close(1_009_999);
        assert!(!s3.is_open());
        assert_eq!(s3.status, SessionStatus::AutoClosed);
        assert_eq!(s3.closed_at_unix, Some(1_009_999));
    }
}
