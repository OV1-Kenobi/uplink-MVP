# ADR-U-011 — Private Relay (NIP-42) + Authoritative Attendance State Machine

## Status
Accepted

## Date
2026-06-09

## Context
ADR-U-008 (work-session model) and ADR-U-009 (NTAG 424 SDM provisioning) both build a
deterministic, replayable **client** core and explicitly defer the **authority** to this
ADR: the client tap toggle and local SDM verify are UX conveniences; the *backend* must be
the single source of truth for whether a work session is `Open` and therefore payable.

Phase 6 introduces that backend: a private OpenAgents Nostr relay (authenticated ingress)
plus an attendance service that re-verifies each tap server-side, runs the authoritative
`WorkSession` state machine, and gates session-bound 6-minute payouts. A forwarded or
replayed SDM URL must never open or extend a paid session, and only the backend — not the
client — decides in/out.

**Dependency note (resolved).** The pivot plan lists Phase 6 as depending on "Phase 5
(identity/Postgres)". Phase 5b (hosted identity service) was deferred; the decision is to
**land 5b's Postgres first and run a single shared backend** that hosts both the 5b identity
resolver and the Phase 6 attendance service over **one Postgres instance**. Attendance keys
on worker **npub + SDM tag UID**; 5b adds the identity/wallet tables to the same database.
Phase 5b therefore lands **before** the attendance edge. See "Sequencing" below.

This ADR governs the new backend crate(s) and service edges. It does **not** change the
custody boundary (ADR-U-010, BOUNDARY.md): the backend gates and audits; it never holds
spend keys.

## Decision

### 1. Service topology — deterministic core, injected I/O edges
The authoritative logic lives in a new **`crates/uplink-attendance`** with **no I/O**: the
7-step validator and the `WorkSession` transition function are pure functions over injected
state, reusing `uplink-ntag424` (SDM verify), `uplink-scheduler` (session gate / 6-min
interval math, ADR-U-008), and `uplink-receipts` (canonical receipt hash). The **shared
backend** service member hosts both the 5b identity resolver endpoints and the attendance
service; relay ingress, the single Postgres, and NIP-42 AUTH live at the **binary/edge**
behind storage and transport traits — mirroring the `Transceive` (ADR-U-009) and
`Nip47Transport` (Phase 3) shim pattern. This keeps the state machine unit-testable,
deterministic, and replayable (AGENTS.md), and isolates the Postgres dependency to the edge.

### 2. Private relay — NIP-42 auth, allowlist, kind policy, retention
- **NIP-42 AUTH required** before any `EVENT`/`REQ`; unauthenticated connections may not
  read or write attendance data.
- **Pubkey allowlist** (`relay_auth_keys`): each enrolled key carries a **role**
  (`worker` | `backend` | `admin`).
- **Kind policy by role (writer authorization):** workers may write only the user-authored
  tap kind; the backend/admin keys author the authoritative session/interval/admin kinds.
  The relay rejects role-violating writes — users can never forge a session state.
- **Raw-event retention:** every accepted signed event is retained verbatim in
  `attendance_events_raw` for audit/replay; nothing is mutated in place.

### 3. Attendance event kinds (schema-versioned)
Allocated to avoid collision with existing kinds (9900 delegation, 9901 receipt, 9902
revocation, 9903 OTP, 30901 stream). Every new kind carries a `v` (schema version) tag.

| Kind | Name | Type | Writer |
|---|---|---|---|
| 9910 | `attendance_tap` | regular, immutable | worker |
| 30910 | `attendance_session` | param. replaceable (`d`=session_id) | backend |
| 9911 | `attendance_interval` | regular, immutable | backend |
| 9912 | `attendance_admin` | regular, immutable | admin |

`attendance_tap` carries the SDM URL / `picc_data`+`cmac`, office hint, and `v`. Presence
kinds are **reserved for Phase 7 / ADR-U-012** and intentionally not defined here.

### 4. Server-side SDM re-verification (authoritative)
On each `attendance_tap`, the backend re-runs `uplink-ntag424` `SdmVerifier::verify(&SdmUrl)`
using the **server-held** office tag keys (never the client's), recovering
`SdmVerification { uid, read_ctr }`. `MacMismatch` is rejected. The server — not the client
— is the authority for **counter monotonicity** (replay rejection), tracked per UID in
`office_tags`.

### 5. The 7-step validation order (authoritative, ordered, fail-closed)
Applied to every inbound `attendance_tap`; any failure rejects and is recorded raw:
1. **Auth/allowlist** — NIP-42-authenticated sender is an enrolled `worker`; else reject.
2. **Well-formedness** — valid signature (relay), kind allowed for the sender's role,
   required tags + supported `v` present; else reject.
3. **SDM verify** — `SdmVerifier::verify` with server tag keys → `(uid, read_ctr)`; reject
   on `MacMismatch`.
4. **Tag enrollment** — `uid` maps to a known `office_tags` row (office_id); reject unknown.
5. **Monotonic counter** — `read_ctr` strictly greater than last-seen for that `uid`;
   reject replays/forwards (then advance last-seen on accept).
6. **Session transition** — toggle the authoritative `WorkSession` for (worker, stream):
   none/closed → `Open`; open → `Closed`; enforce single-open-per-stream (ADR-U-008 §4–5).
7. **Persist + emit** — append `attendance_events_raw`, upsert `attendance_sessions`, author
   the backend-signed `attendance_session` (30910). Idempotent on `(uid, read_ctr)`: a
   duplicate delivery is a no-op returning the original transition.

### 6. Session-gated payout scheduler (server-side)
The backend runs the `uplink-scheduler` session gate (ADR-U-008 §5) authoritatively: for an
`InOfficeStreaming` stream it accrues a `stream_intervals` row and authorizes one payout per
fixed `IN_OFFICE_PERIOD_SECONDS = 360` only while an `Open` session exists, **re-checking the
session state before each interval**. `Suspended` / `AutoClosed` / `Closed` / "no session"
emit nothing. The backend authorizes intents; it **does not hold spend keys** — the payer's
wallet executes them through the `WalletProvider` (NWC spend, ADR-U-007/010), keyed
idempotently by `(intent_id, leg_index)` with `intent_id = stream_id:period_index`
(ADR-U-008 §7). Each authorized interval stamps its `session_id` for audit.

### 7. Postgres schema (shared backend datastore)
One Postgres instance hosts both 5b identity and Phase 6 attendance, all migration-managed
and additive. **5b identity/wallet tables** (per ADR-U-010 §3): normalized username, pubkey,
receive-only routing fields, encrypted credential, revocation/timestamps — the identity
backend can never spend. **Phase 6 attendance tables (five):** `attendance_events_raw`
(verbatim signed events + parsed `(uid, read_ctr)`), `attendance_sessions` (`session_id`,
worker npub, `stream_id`, `status`, `opened_at`/`closed_at`), `stream_intervals`
(`session_id`, `period_index`, `intent_id`, payout status), `office_tags` (`uid`,
`office_id`, encrypted tag keys, `last_read_ctr`), `relay_auth_keys` (pubkey, role,
enrolled/revoked timestamps).

### 8. Custody, secrets, and boundary (unchanged invariants)
Office tag keys and the backend writer key are bearer secrets: encrypted at rest, never
logged, never returned to any client (ADR-U-010, BOUNDARY.md). Consistent with the
identity-backend rule, the **attendance backend can never spend** — it only gates and
audits. No raw SDM keys or writer secrets appear in error messages or telemetry.

### 9. Determinism, replay, and honest gating
State transitions are pure functions of `(raw event, prior state)`; retained
`attendance_events_raw` lets the full session history be replayed and re-hashed for audit
(AGENTS.md replayability). Parts that need a live relay/hardware channel not exercisable in
CI return typed errors, never `todo!()`/`unimplemented!()` (ADR-U-007 §5, ADR-U-009 §6).

## Sequencing (decided)
Phase 5b lands **first**: stand up the shared backend + single Postgres with the 5b identity
schema and resolver endpoints, then add the Phase 6 attendance tables, relay NIP-42 ingress,
and state machine on the same service. Live relay/Postgres secrets + deployment are handled
separately (deferred operational concern); the deterministic core and migrations land first.

## Consequences
- **Positive:** the backend is the single attendance authority; replay/forwarded taps are
  rejected (server-side monotonic counter); payouts stop immediately on close and never run
  for non-`Open` sessions; the deterministic core is unit-testable and replayable; custody
  and the no-spend backend invariant are preserved.
- **Negative / deferred:** presence/geofence suspension is Phase 7 (ADR-U-012); the relay's
  live NIP-42/secure-channel paths are validated outside CI; introduces operational Postgres
  + a long-running relay service (the MVP's first always-on backend).
- **Migration:** new crate `uplink-attendance` + a new service edge and DB migrations;
  additive to the workspace. Existing crates gain `uplink-attendance` as a member only.

## References
- `docs/plans/uplink-pivot.md` — Phase 6
- `docs/adr/ADR-U-003-receipt-event-kind.md` — existing kind allocations
- `docs/adr/ADR-U-008-automation-types-work-session-model.md` — `WorkSession` + session gate
- `docs/adr/ADR-U-009-ntag424-sdm-provisioning-verification.md` — SDM verify contract
- `docs/adr/ADR-U-010-external-credential-identity-and-relay-storage.md` — custody + relays
- `BOUNDARY.md`, `AGENTS.md` — boundary, idempotency, replay, no-`todo!()` invariants
