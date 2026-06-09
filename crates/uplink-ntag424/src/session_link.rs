//! Tap → work-session bridge (consumes ADR-U-008; feature `session`).
//!
//! A verified tap toggles the in-office work session for a stream. The client toggle is a
//! UX convenience stamped with the verified `(uid, read_ctr)`; the backend remains the
//! authority in Phase 6 (ADR-U-011).

use uplink_scheduler::Scheduler;

use crate::sdm::SdmVerification;

/// What a verified tap does to the current session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapAction {
    /// No open session for the stream → this tap opens (clock-in).
    Open,
    /// An open session exists → this tap closes it (clock-out).
    Close,
}

/// Decide the action for a tap given whether a session is currently open.
pub fn tap_action(session_open: bool) -> TapAction {
    if session_open {
        TapAction::Close
    } else {
        TapAction::Open
    }
}

/// Apply a verified tap to the scheduler for `stream_id`, toggling the in-office session.
///
/// Returns the action taken. `session_id` is used only when opening. The verification's
/// `(uid, read_ctr)` is the audit anchor the caller persists alongside the transition.
pub fn apply_tap(
    scheduler: &mut Scheduler,
    stream_id: &str,
    session_id: impl Into<String>,
    now_unix: u64,
    _verification: &SdmVerification,
) -> TapAction {
    let open = scheduler.open_session_for(stream_id).is_some();
    match tap_action(open) {
        TapAction::Open => {
            scheduler.open_session(session_id, stream_id, now_unix);
            TapAction::Open
        }
        TapAction::Close => {
            scheduler.close_session(stream_id, now_unix);
            TapAction::Close
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uplink_scheduler::stream::{AutomationType, StreamPolicy, StreamStatus};

    fn in_office_stream(id: &str) -> StreamPolicy {
        StreamPolicy {
            stream_id: id.into(),
            source_wallet_id: "w1".into(),
            legs: vec![],
            period_seconds: 360,
            start_at_unix: 1_000_000,
            end_at_unix: None,
            max_total_msats: None,
            status: StreamStatus::Active,
            nostr_event_id: None,
            last_executed_period: None,
            automation_type: AutomationType::InOfficeStreaming,
        }
    }

    fn verification() -> SdmVerification {
        SdmVerification { uid: [1, 2, 3, 4, 5, 6, 7], read_ctr: 42 }
    }

    #[test]
    fn first_tap_opens_second_tap_closes() {
        let mut sched = Scheduler::new(vec![in_office_stream("s1")]);
        let v = verification();

        let a1 = apply_tap(&mut sched, "s1", "sess-1", 1_000_000, &v);
        assert_eq!(a1, TapAction::Open);
        assert!(sched.open_session_for("s1").is_some());

        let a2 = apply_tap(&mut sched, "s1", "sess-2", 1_000_500, &v);
        assert_eq!(a2, TapAction::Close);
        assert!(sched.open_session_for("s1").is_none());
    }
}
