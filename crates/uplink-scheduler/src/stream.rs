//! Streaming payment policy (the persisted declaration of a recurring flow).

use serde::{Deserialize, Serialize};
use uplink_accounts::SplitLeg;

/// Status of an active stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamStatus {
    Active,
    Paused,
    Completed,
    Revoked,
}

/// Fixed cadence for in-office attendance streaming (6 minutes). Not user-tunable —
/// it is the unit of attendance accrual (ADR-U-008).
pub const IN_OFFICE_PERIOD_SECONDS: u64 = 360;

/// How a stream is driven (ADR-U-008).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationType {
    /// Pays once at period 0, then completes.
    OneTime,
    /// Pays every cadence period (the historical default behavior).
    StandardRecurring,
    /// Session-gated; fixed 6-minute intervals, payable only while a work
    /// session is open.
    InOfficeStreaming,
}

impl Default for AutomationType {
    fn default() -> Self {
        AutomationType::StandardRecurring
    }
}

/// Human-friendly cadence presets for `StandardRecurring` streams.
///
/// Month/year are fixed-length approximations for MVP scheduling (ADR-U-008);
/// calendar-exact billing is out of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cadence {
    SixMin,
    Daily,
    Weekly,
    Monthly,
    Annual,
}

impl Cadence {
    /// The period length, in seconds, this preset maps to.
    pub fn as_seconds(self) -> u64 {
        match self {
            Cadence::SixMin => IN_OFFICE_PERIOD_SECONDS,
            Cadence::Daily => 86_400,
            Cadence::Weekly => 604_800,
            Cadence::Monthly => 2_592_000,
            Cadence::Annual => 31_536_000,
        }
    }
}

/// The full declaration of a recurring streaming-sats flow.
///
/// Persisted in `uplink-storage`; also published to Nostr as kind-30901.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamPolicy {
    /// Stable ID (UUID hex). Used as the kind-30901 `d` tag.
    pub stream_id: String,
    /// Source wallet ID.
    pub source_wallet_id: String,
    /// Recipient legs (same structure as `SplitPaymentIntent.legs`).
    pub legs: Vec<SplitLeg>,
    /// Payment period in seconds (e.g. 3600 = hourly, 86400 = daily).
    pub period_seconds: u64,
    /// Unix timestamp of first payment.
    pub start_at_unix: u64,
    /// Optional hard stop.
    pub end_at_unix: Option<u64>,
    /// Optional total sats cap across all periods.
    pub max_total_msats: Option<u64>,
    pub status: StreamStatus,
    /// Nostr event ID of the kind-30901 declaration (set after publish).
    pub nostr_event_id: Option<String>,
    /// Period index of the last successfully executed payment (None = none yet).
    pub last_executed_period: Option<u64>,
    /// How this stream is driven (one-time / standard recurring / in-office).
    /// Defaults to `StandardRecurring` for backward compatibility (ADR-U-008).
    #[serde(default)]
    pub automation_type: AutomationType,
}

impl StreamPolicy {
    /// The period length actually used by this stream. In-office streaming is
    /// fixed at 6 minutes regardless of `period_seconds` (ADR-U-008).
    pub fn effective_period_seconds(&self) -> u64 {
        match self.automation_type {
            AutomationType::InOfficeStreaming => IN_OFFICE_PERIOD_SECONDS,
            _ => self.period_seconds,
        }
    }

    /// Compute the period index for a given `now` timestamp.
    ///
    /// One-time streams collapse to a single period 0.
    pub fn period_index_at(&self, now_unix: u64) -> Option<u64> {
        if now_unix < self.start_at_unix {
            return None;
        }
        if self.automation_type == AutomationType::OneTime {
            return Some(0);
        }
        let period = self.effective_period_seconds();
        if period == 0 {
            return Some(0);
        }
        Some((now_unix - self.start_at_unix) / period)
    }

    /// Whether a payment is due at `now` (i.e., the current period hasn't been paid yet).
    ///
    /// This covers period/one-time/budget/status checks only. Session gating for
    /// in-office streaming is applied by the `Scheduler` (ADR-U-008).
    pub fn is_due_at(&self, now_unix: u64) -> bool {
        if self.status != StreamStatus::Active {
            return false;
        }
        if let Some(end) = self.end_at_unix {
            if now_unix >= end {
                return false;
            }
        }
        match self.period_index_at(now_unix) {
            None => false,
            Some(idx) => match self.last_executed_period {
                None => true,
                Some(last) => idx > last,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_policy(period_seconds: u64, start: u64) -> StreamPolicy {
        StreamPolicy {
            stream_id: "s1".into(),
            source_wallet_id: "w1".into(),
            legs: vec![],
            period_seconds,
            start_at_unix: start,
            end_at_unix: None,
            max_total_msats: None,
            status: StreamStatus::Active,
            nostr_event_id: None,
            last_executed_period: None,
            automation_type: AutomationType::StandardRecurring,
        }
    }

    #[test]
    fn due_on_first_period() {
        let p = make_policy(3600, 1_000_000);
        assert!(p.is_due_at(1_000_000));
        assert!(p.is_due_at(1_003_600));
    }

    #[test]
    fn not_due_before_start() {
        let p = make_policy(3600, 1_000_000);
        assert!(!p.is_due_at(999_999));
    }

    #[test]
    fn one_time_is_due_once_then_not() {
        let mut p = make_policy(3600, 1_000_000);
        p.automation_type = AutomationType::OneTime;
        assert!(p.is_due_at(1_000_000));
        // One-time collapses to period 0 regardless of elapsed time.
        assert_eq!(p.period_index_at(1_500_000), Some(0));
        p.last_executed_period = Some(0);
        assert!(!p.is_due_at(1_500_000));
    }

    #[test]
    fn in_office_uses_fixed_six_minute_period() {
        let mut p = make_policy(3600, 1_000_000); // stored period is ignored
        p.automation_type = AutomationType::InOfficeStreaming;
        assert_eq!(p.effective_period_seconds(), IN_OFFICE_PERIOD_SECONDS);
        assert_eq!(p.period_index_at(1_000_000), Some(0));
        assert_eq!(p.period_index_at(1_000_000 + 359), Some(0));
        assert_eq!(p.period_index_at(1_000_000 + 360), Some(1));
    }

    #[test]
    fn cadence_presets_map_to_seconds() {
        assert_eq!(Cadence::SixMin.as_seconds(), IN_OFFICE_PERIOD_SECONDS);
        assert_eq!(Cadence::Daily.as_seconds(), 86_400);
        assert_eq!(Cadence::Weekly.as_seconds(), 604_800);
        assert_eq!(Cadence::Monthly.as_seconds(), 2_592_000);
        assert_eq!(Cadence::Annual.as_seconds(), 31_536_000);
    }
}
