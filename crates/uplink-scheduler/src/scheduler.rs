//! The `Scheduler` struct — tick-driven recurring payment dispatcher.

use uplink_accounts::{SplitPaymentIntent, SplitLeg};
use crate::stream::StreamPolicy;
use crate::SchedulerError;

/// The Uplink recurring-payment scheduler.
///
/// Call `tick(now_unix)` on every JS timer interval (browser)
/// or tokio interval (native). The scheduler emits `SplitPaymentIntent`s
/// for all due streams and updates their `last_executed_period`.
pub struct Scheduler {
    streams: Vec<StreamPolicy>,
}

impl Scheduler {
    pub fn new(streams: Vec<StreamPolicy>) -> Self {
        Self { streams }
    }

    /// Add or replace a stream policy.
    pub fn upsert_stream(&mut self, policy: StreamPolicy) {
        if let Some(existing) = self.streams.iter_mut().find(|s| s.stream_id == policy.stream_id) {
            *existing = policy;
        } else {
            self.streams.push(policy);
        }
    }

    /// Remove a stream by ID.
    pub fn remove_stream(&mut self, stream_id: &str) {
        self.streams.retain(|s| s.stream_id != stream_id);
    }

    /// Process all due streams at `now_unix` and return payment intents to execute.
    ///
    /// Callers must:
    /// 1. Execute each intent via `WalletExecutor::pay_invoice` (idempotent on leg key).
    /// 2. On success, call `mark_executed(stream_id, period_index)`.
    /// 3. Publish a kind-9901 receipt to Nostr.
    pub fn tick(&self, now_unix: u64) -> Vec<SplitPaymentIntent> {
        self.streams
            .iter()
            .filter(|s| s.is_due_at(now_unix))
            .map(|s| self.build_intent(s, now_unix))
            .collect()
    }

    /// Mark a period as successfully executed (call after wallet payment confirms).
    pub fn mark_executed(&mut self, stream_id: &str, period_index: u64) {
        if let Some(s) = self.streams.iter_mut().find(|s| s.stream_id == stream_id) {
            s.last_executed_period = Some(period_index);
        }
    }

    fn build_intent(&self, policy: &StreamPolicy, now_unix: u64) -> SplitPaymentIntent {
        let period_index = policy.period_index_at(now_unix).unwrap_or(0);
        // Intent ID: deterministic from stream_id + period_index
        let intent_id = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(policy.stream_id.as_bytes());
            h.update(b":");
            h.update(period_index.to_string().as_bytes());
            hex::encode(h.finalize())
        };

        SplitPaymentIntent {
            intent_id,
            stream_id: policy.stream_id.clone(),
            period_index,
            source_wallet_id: policy.source_wallet_id.clone(),
            legs: policy.legs.clone(),
            created_at_unix: now_unix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::StreamStatus;

    fn make_stream(id: &str, period: u64, start: u64) -> StreamPolicy {
        StreamPolicy {
            stream_id: id.into(),
            source_wallet_id: "w1".into(),
            legs: vec![SplitLeg {
                leg_index: 0,
                recipient_npub_hex: "npub1test".into(),
                msats: 10_000,
                max_fee_msats: 1_000,
                memo: None,
                prefer_stable_channel: true,
            }],
            period_seconds: period,
            start_at_unix: start,
            end_at_unix: None,
            max_total_msats: None,
            status: StreamStatus::Active,
            nostr_event_id: None,
            last_executed_period: None,
        }
    }

    #[test]
    fn tick_emits_intent_for_due_stream() {
        let mut sched = Scheduler::new(vec![make_stream("s1", 3600, 1_000_000)]);
        let intents = sched.tick(1_000_000);
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].stream_id, "s1");
        assert_eq!(intents[0].period_index, 0);

        // Mark executed; next tick within same period should not re-emit
        sched.mark_executed("s1", 0);
        let intents2 = sched.tick(1_001_000); // still period 0
        assert_eq!(intents2.len(), 0);

        // Next period tick
        let intents3 = sched.tick(1_003_600);
        assert_eq!(intents3.len(), 1);
        assert_eq!(intents3[0].period_index, 1);
    }

    #[test]
    fn intent_id_is_deterministic() {
        let sched = Scheduler::new(vec![make_stream("s1", 3600, 1_000_000)]);
        let a = sched.tick(1_000_000);
        let b = sched.tick(1_000_000);
        assert_eq!(a[0].intent_id, b[0].intent_id);
    }
}
