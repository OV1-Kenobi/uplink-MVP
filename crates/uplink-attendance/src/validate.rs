//! The authoritative 7-step inbound-tap validator (ADR-U-011 §5).
//!
//! Pure: a deterministic function of `(tap, injected prior state)`. The relay/Postgres edge
//! supplies the server-held [`SdmVerifier`], the [`TagDirectory`] snapshot, and the prior
//! [`WorkSession`]; this function performs no I/O and never logs secrets. Idempotent
//! redelivery (same `uid`/`read_ctr`) is handled at the edge (step 7); here an equal-or-lower
//! counter is a fail-closed replay reject (step 5).

use uplink_ntag424::{SdmUrl, SdmVerification, SdmVerifier};
use uplink_scheduler::WorkSession;

use crate::kinds::{kind_allowed_for_role, ATTENDANCE_SCHEMA_VERSION};
use crate::outcome::{MalformedKind, RejectReason, TapAccepted, Transition};
use crate::role::Role;
use crate::tag::TagDirectory;

/// A parsed inbound `attendance_tap`, with the sender's resolved allowlist role.
pub struct TapEvaluationInput<'a> {
    /// The tap's Nostr event id (a content hash) — used as the opened session id (replayable).
    pub tap_event_id: &'a str,
    /// The sender's resolved role from the relay allowlist (`None` = not enrolled).
    pub sender_role: Option<Role>,
    /// The event kind.
    pub kind: u16,
    /// The `v` schema-version tag.
    pub schema_version: u32,
    /// The raw SDM URL carried by the tap.
    pub sdm_url: &'a str,
    /// The in-office stream this tap toggles.
    pub stream_id: &'a str,
}

/// Run the authoritative 7-step validation/transition for one inbound tap.
///
/// On success returns the [`TapAccepted`] state the edge must persist (step 7). Any step
/// failure returns the corresponding fail-closed [`RejectReason`].
pub fn evaluate_tap(
    input: &TapEvaluationInput<'_>,
    verifier: &SdmVerifier,
    tags: &dyn TagDirectory,
    prior_session: Option<&WorkSession>,
    now_unix: u64,
) -> Result<TapAccepted, RejectReason> {
    // Step 1 — auth/allowlist: the sender must be an enrolled worker.
    let Some(role) = input.sender_role else {
        return Err(RejectReason::NotAuthorized);
    };
    if role != Role::Worker {
        return Err(RejectReason::NotAuthorized);
    }
    // Step 2 — well-formedness: kind allowed for the role + supported schema version.
    if !kind_allowed_for_role(input.kind, role) {
        return Err(RejectReason::Malformed(MalformedKind::WrongKind));
    }
    if input.schema_version != ATTENDANCE_SCHEMA_VERSION {
        return Err(RejectReason::Malformed(MalformedKind::UnsupportedVersion));
    }
    let url = SdmUrl::parse(input.sdm_url)
        .map_err(|_| RejectReason::Malformed(MalformedKind::UnparseableSdmUrl))?;
    // Step 3 — SDM verify with server-held office keys.
    let SdmVerification { uid, read_ctr } =
        verifier.verify(&url).map_err(|_| RejectReason::SdmVerifyFailed)?;
    // Step 4 — tag enrollment: the UID must map to a known office tag.
    let tag = tags.office_tag(&uid).ok_or(RejectReason::UnknownTag)?;
    // Step 5 — monotonic counter: strictly greater than the last accepted value.
    if let Some(last) = tag.last_read_ctr {
        if read_ctr <= last {
            return Err(RejectReason::ReplayedCounter { seen: read_ctr, last });
        }
    }
    // Step 6 — authoritative session toggle (single-open-per-stream).
    let (transition, session) = transition_session(prior_session, input, now_unix);
    // Step 7 — outcome for the edge to persist/emit.
    Ok(TapAccepted {
        uid,
        read_ctr,
        office_id: tag.office_id.clone(),
        transition,
        session,
        new_last_read_ctr: read_ctr,
    })
}

/// Toggle the authoritative session: an open session closes; any other state (or none) opens
/// a fresh session keyed by the tap's event id (ADR-U-008 §4–5, ADR-U-011 §5.6).
fn transition_session(
    prior: Option<&WorkSession>,
    input: &TapEvaluationInput<'_>,
    now_unix: u64,
) -> (Transition, WorkSession) {
    match prior {
        Some(s) if s.is_open() => {
            let mut closed = s.clone();
            closed.close(now_unix);
            (Transition::Closed, closed)
        }
        _ => (
            Transition::Opened,
            WorkSession::open(input.tap_event_id, input.stream_id, now_unix),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::OfficeTag;
    use std::collections::HashMap;

    // NXP AN12196 worked example with factory (all-zero) keys (see uplink-ntag424).
    const TAP_URL: &str =
        "https://uplink.example/?picc_data=EF963FF7828658A599F3041510671E88&cmac=94EED9EE65337086";

    fn verifier() -> SdmVerifier {
        SdmVerifier::single_key([0u8; 16])
    }

    // The UID/counter recovered from TAP_URL (learned by verifying once).
    fn verified() -> SdmVerification {
        verifier().verify(&SdmUrl::parse(TAP_URL).unwrap()).unwrap()
    }

    fn dir(last_read_ctr: Option<u32>) -> HashMap<[u8; 7], OfficeTag> {
        let uid = verified().uid;
        let mut d = HashMap::new();
        d.insert(uid, OfficeTag { uid, office_id: "hq".into(), last_read_ctr });
        d
    }

    fn input<'a>(role: Option<Role>, kind: u16, version: u32, url: &'a str) -> TapEvaluationInput<'a> {
        TapEvaluationInput {
            tap_event_id: "evt-1",
            sender_role: role,
            kind,
            schema_version: version,
            sdm_url: url,
            stream_id: "stream-1",
        }
    }

    fn ok_input<'a>() -> TapEvaluationInput<'a> {
        input(Some(Role::Worker), crate::kinds::ATTENDANCE_TAP, ATTENDANCE_SCHEMA_VERSION, TAP_URL)
    }

    #[test]
    fn first_tap_opens_session() {
        let acc = evaluate_tap(&ok_input(), &verifier(), &dir(None), None, 1_000).unwrap();
        assert_eq!(acc.transition, Transition::Opened);
        assert!(acc.session.is_open());
        assert_eq!(acc.session.session_id, "evt-1");
        assert_eq!(acc.new_last_read_ctr, acc.read_ctr);
        assert_eq!(acc.office_id, "hq");
    }

    #[test]
    fn second_tap_closes_open_session() {
        let v = verified();
        let prior = WorkSession::open("evt-0", "stream-1", 500);
        let tags = dir(Some(v.read_ctr - 1));
        let acc = evaluate_tap(&ok_input(), &verifier(), &tags, Some(&prior), 2_000).unwrap();
        assert_eq!(acc.transition, Transition::Closed);
        assert!(!acc.session.is_open());
        assert_eq!(acc.session.closed_at_unix, Some(2_000));
    }

    #[test]
    fn rejects_unauthorized_and_malformed() {
        let tags = dir(None);
        let v = verifier();
        let tap = crate::kinds::ATTENDANCE_TAP;
        assert_eq!(
            evaluate_tap(&input(None, tap, 1, TAP_URL), &v, &tags, None, 0).unwrap_err(),
            RejectReason::NotAuthorized
        );
        assert_eq!(
            evaluate_tap(&input(Some(Role::Backend), tap, 1, TAP_URL), &v, &tags, None, 0).unwrap_err(),
            RejectReason::NotAuthorized
        );
        assert_eq!(
            evaluate_tap(&input(Some(Role::Worker), crate::kinds::ATTENDANCE_SESSION, 1, TAP_URL), &v, &tags, None, 0).unwrap_err(),
            RejectReason::Malformed(MalformedKind::WrongKind)
        );
        assert_eq!(
            evaluate_tap(&input(Some(Role::Worker), tap, 2, TAP_URL), &v, &tags, None, 0).unwrap_err(),
            RejectReason::Malformed(MalformedKind::UnsupportedVersion)
        );
        assert_eq!(
            evaluate_tap(&input(Some(Role::Worker), tap, 1, "https://x/"), &v, &tags, None, 0).unwrap_err(),
            RejectReason::Malformed(MalformedKind::UnparseableSdmUrl)
        );
    }

    #[test]
    fn rejects_bad_mac_unknown_tag_and_replay() {
        // Wrong file-read key → MAC mismatch (step 3).
        let bad = SdmVerifier::new([0u8; 16], [0x11u8; 16]);
        assert_eq!(
            evaluate_tap(&ok_input(), &bad, &dir(None), None, 0).unwrap_err(),
            RejectReason::SdmVerifyFailed
        );
        // Empty directory → unknown tag (step 4).
        let empty: HashMap<[u8; 7], OfficeTag> = HashMap::new();
        assert_eq!(
            evaluate_tap(&ok_input(), &verifier(), &empty, None, 0).unwrap_err(),
            RejectReason::UnknownTag
        );
        // last_read_ctr == this tap's counter → replay (step 5, strictly-greater required).
        let v = verified();
        assert_eq!(
            evaluate_tap(&ok_input(), &verifier(), &dir(Some(v.read_ctr)), None, 0).unwrap_err(),
            RejectReason::ReplayedCounter { seen: v.read_ctr, last: v.read_ctr }
        );
    }
}
