//! Attendance event kinds + role-based writer policy (ADR-U-011 §2–3).
//!
//! Kinds are allocated to avoid collision with existing allocations (9900 delegation,
//! 9901 receipt, 9902 revocation, 9903 OTP, 30901 stream). Every attendance event carries a
//! `v` (schema version) tag; this core accepts [`ATTENDANCE_SCHEMA_VERSION`].

use crate::role::Role;

/// Worker-authored tap (regular, immutable).
pub const ATTENDANCE_TAP: u16 = 9910;
/// Backend-authored session (parameterized-replaceable, `d` = session_id).
pub const ATTENDANCE_SESSION: u16 = 30910;
/// Backend-authored payout interval (regular, immutable).
pub const ATTENDANCE_INTERVAL: u16 = 9911;
/// Admin-authored attendance admin event (regular, immutable).
pub const ATTENDANCE_ADMIN: u16 = 9912;

/// The single schema version this core understands.
pub const ATTENDANCE_SCHEMA_VERSION: u32 = 1;

/// Whether `role` is authorized to write `kind` (writer authorization, §2–3).
///
/// Workers may write only the tap kind; the backend authors session/interval kinds; admin
/// authors admin kinds. Any other (role, kind) pairing is rejected by the relay/validator.
#[must_use]
pub fn kind_allowed_for_role(kind: u16, role: Role) -> bool {
    match role {
        Role::Worker => kind == ATTENDANCE_TAP,
        Role::Backend => matches!(kind, ATTENDANCE_SESSION | ATTENDANCE_INTERVAL),
        Role::Admin => kind == ATTENDANCE_ADMIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_may_only_write_tap() {
        assert!(kind_allowed_for_role(ATTENDANCE_TAP, Role::Worker));
        assert!(!kind_allowed_for_role(ATTENDANCE_SESSION, Role::Worker));
        assert!(!kind_allowed_for_role(ATTENDANCE_INTERVAL, Role::Worker));
    }

    #[test]
    fn backend_and_admin_kinds() {
        assert!(kind_allowed_for_role(ATTENDANCE_SESSION, Role::Backend));
        assert!(kind_allowed_for_role(ATTENDANCE_INTERVAL, Role::Backend));
        assert!(!kind_allowed_for_role(ATTENDANCE_TAP, Role::Backend));
        assert!(kind_allowed_for_role(ATTENDANCE_ADMIN, Role::Admin));
        assert!(!kind_allowed_for_role(ATTENDANCE_ADMIN, Role::Backend));
    }
}
