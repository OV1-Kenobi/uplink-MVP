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
}

impl StreamPolicy {
    /// Compute the period index for a given `now` timestamp.
    pub fn period_index_at(&self, now_unix: u64) -> Option<u64> {
        if now_unix < self.start_at_unix {
            return None;
        }
        Some((now_unix - self.start_at_unix) / self.period_seconds)
    }

    /// Whether a payment is due at `now` (i.e., the current period hasn't been paid yet).
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
}
