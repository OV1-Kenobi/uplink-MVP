# ADR-U-008 — Automation Types + Work-Session Model

## Status
Accepted

## Date
2026-06-09

## Context
The pivot docs introduce a "key correction" to the streaming model: onsite-worker
payouts must be **session-gated**, not always-on. The original `uplink-scheduler`
treats every stream as an unconditional recurring flow — `tick(now)` pays any stream
whose period has elapsed. That is correct for subscriptions and tips, but wrong for
**in-office attendance streaming**, where a worker must be physically present (an open
work session) for each 6-minute interval to be payable, and payments must **stop
immediately** when the session closes (clock-out, geofence exit, suspension).

We also need to distinguish one-shot payments (e.g. a single bounty) from recurring
flows, and to offer human-friendly cadence presets instead of raw second counts.

This ADR governs `crates/uplink-scheduler` and the `SplitPaymentIntent` shape in
`crates/uplink-accounts`. It does **not** introduce backend verification — server-side
SDM/CMAC checks and the authoritative attendance state machine land in Phase 6
(ADR-U-011). The client model here is the deterministic, replayable core those phases
build on.

## Decision

### 1. `AutomationType` classifies every stream
```
OneTime            // pays once at period 0, then completes
StandardRecurring  // pays every cadence period (existing behavior; the default)
InOfficeStreaming  // session-gated, fixed 6-minute intervals
```
`AutomationType` is added to `StreamPolicy` with `#[serde(default)]` defaulting to
`StandardRecurring`, so existing persisted policies and the current wasm boundary
(`upsert_stream`) keep their behavior unchanged.

### 2. Cadence presets
A `Cadence` enum maps to a fixed `period_seconds` for UI selection:
`SixMin = 360`, `Daily = 86_400`, `Weekly = 604_800`, `Monthly = 2_592_000` (30 d),
`Annual = 31_536_000` (365 d). Month/year are fixed approximations for MVP scheduling;
calendar-exact billing is out of scope.

### 3. In-office is fixed at 6 minutes
`InOfficeStreaming` always uses `IN_OFFICE_PERIOD_SECONDS = 360` for period math,
regardless of any stored `period_seconds`. The cadence is **not** user-tunable — it is
the unit of attendance accrual.

### 4. `WorkSession` state machine
```
SessionStatus: Open | Closed | Suspended | AutoClosed
```
- `Open` — worker is clocked in; in-office intervals are payable.
- `Closed` — normal clock-out; no further payouts.
- `Suspended` — presence lost (e.g. geofence exit) with a grace window (Phase 7);
  not payable.
- `AutoClosed` — system-closed (max duration / missing close tap); not payable.

A `WorkSession` carries `session_id`, the gated `stream_id`, `opened_at_unix`, an
optional `closed_at_unix`, and `status`. **Only `Open` sessions gate payouts** —
`Suspended`, `AutoClosed`, `Closed`, and "no session" all block emission.

### 5. Session-gated `tick`
The `Scheduler` owns the active `WorkSession`s (at most one open per `stream_id`).
`tick(now)` emits an intent for a stream only when **both**:
1. `StreamPolicy::is_due_at(now)` (period/one-time/budget/status checks), **and**
2. the session gate is open — for `InOfficeStreaming`, an `Open` session exists for the
   stream; for `OneTime`/`StandardRecurring` the gate is always open.

Closing/suspending/auto-closing a session makes subsequent ticks emit nothing for that
stream **on the next tick** — payments stop immediately on close.

### 6. Intents are linked to their session
`SplitPaymentIntent` gains `session_id: Option<String>` (with `#[serde(default)]`,
backward compatible). In-office intents stamp the open session's id; one-time and
standard-recurring intents leave it `None`. This makes every in-office payout auditable
back to the session that authorized it (the audit trail the Phase 6 backend consumes).

### 7. Idempotency unchanged
The idempotency key remains `(intent_id, leg_index)` with `intent_id` derived from
`stream_id:period_index`. Session gating only decides **whether** an intent is emitted;
it never changes the key, so replay/idempotency invariants (AGENTS.md) hold.

## Consequences
- **Positive:** in-office payouts are presence-gated and stop on clock-out; one-time vs
  recurring is explicit; cadence selection is human-friendly; every in-office payout is
  traceable to a session; the scheduler stays threadless/clockless and deterministic.
- **Negative / deferred:** the session lifecycle is driven by callers (NFC taps in
  Phase 4, the backend state machine in Phase 6); this ADR does not yet expose session
  open/close across the wasm/Tauri boundary — that wiring lands with the NFC client.
  Month/year cadences are fixed-length approximations.
- **Migration:** additive only. `serde(default)` on both new fields keeps existing
  serialized policies/intents loading unchanged; the wasm `upsert_stream` continues to
  produce `StandardRecurring` streams until a later UI phase exposes the selector.

## References
- `docs/plans/uplink-pivot.md` — Phase 2 (Domain model redirect)
- `docs/adr/ADR-U-006-platform-pivot-tauri-foss-distribution.md` — platform pivot
- ADR-U-011 (forthcoming) — private relay + authoritative attendance state machine
- `AGENTS.md` — idempotency + no-`todo!()` invariants
