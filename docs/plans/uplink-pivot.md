# Uplink Pivot — Tauri v2 + Remote-Wallet + Attendance Backend (Sequenced Plan)

**Status:** Active · **Governing ADR:** ADR-U-006 · **Date:** 2026-06-09

This plan migrates Uplink from the Wasm/React PWA (Phases A0–A8) to a native Tauri v2 app
plus a backend services tier, while reusing the existing `uplink-*` Rust crates and React UI.
Identity / new-service work is isolated into its own phases (Track 2) per direction.

## Hard constraints (from ADR-U-006)

- **No Google Play Store.** Distribution = **Zapstore** (primary) + **direct signed APK** +
  **F-Droid** (stretch). Keep the build FOSS / reproducible-eligible from day one.
- **De-Googled target (Pixel 10, GrapheneOS-class):** **no Google Play Services.** No FCM,
  no Fused Location, no Play Integrity, no Maps SDK, no `com.google.android.gms`.
- **Self-contained tags:** the app must **read AND program** NTAG 424 DNA with no other app.
- **FOSS substitutions:** geofencing via AOSP `LocationManager`; push via **UnifiedPush** or
  Nostr relay subscription; NTAG 424 logic in a **pure-Rust crate** (`uplink-ntag424`).

## Engineering invariants (carried from AGENTS.md / BOUNDARY.md)

- No wallet keys cross the UI boundary.
- No `todo!()` / `unimplemented!()` in production paths (currently violated in `zap.rs`,
  `lsp.rs`; resolved in Phase 3).
- Idempotent intent IDs for all payments.
- Each phase ships something testable on its own.

---

## Track 1 — Client pivot (in-repo; reuses Rust crates + React UI; $0 on Android)

### Phase 1 — Tauri v2 foundation
**Goal:** Replace the PWA shell with Tauri v2; compile `uplink-*` crates **natively** (no
Wasm constraints); keep the Netlify web/desktop dashboard alive in parallel.
- **Deliverables:** `src-tauri/` crate wrapping `web/`; native `uplink-core` wired to Tauri
  commands; desktop build runs; Android project generated with **FOSS Gradle config**
  (no GMS, no Firebase); sideloadable debug APK target documented.
- **Depends on:** nothing (start here).
- **ADRs:** ADR-U-006 (done); **revise `BOUNDARY.md`** for the Tauri native↔UI boundary.
- **Acceptance:** App launches on desktop and on the de-Googled Pixel; identity +
  scheduler-UI flows work against the native core.

### Phase 2 — Domain model redirect (the docs' "key correction")
**Goal:** Make in-office streaming **session-gated** instead of always-on.
- **Deliverables:** `automation_type` enum (`one_time` / `standard_recurring` /
  `in_office_streaming`); `WorkSession` model (`open` / `closed` / `suspended` /
  `auto_closed`) in `uplink-scheduler`; session-gated controller where in-office is **fixed
  at 6 minutes and only ticks while a session is open**; cadence presets (6-min / daily /
  weekly / monthly / annual); `Scheduler::tick` respects session state and links each
  interval to a `session_id`.
- **Depends on:** Phase 1.
- **ADRs:** ADR-U-008 — automation types + work-session model.
- **Acceptance:** standard recurring pays on cadence; in-office produces intervals only while
  a session is open and stops immediately on close (unit-tested).

### Phase 3 — WalletProvider abstraction + recipient resolver + stub cleanup
**Goal:** Decouple business logic from the concrete wallet; remove runtime `todo!()` panics.
- **Deliverables:**
  - `WalletProvider` trait: `get_info`, `get_balance`, `make_invoice`, `pay_invoice`,
    `lookup_invoice`, `list_transactions`, `is_available`, `get_capabilities`.
  - Existing LDK wrapped as one impl; **NWC / NIP-47 adapter** as a second impl.
  - Recipient resolver (NIP-05 / Lightning-address / LNURL-pay / npub / QR) — **resolves the
    `zap.rs` `todo!()`**.
  - Gate or implement remaining stubs (`uplink-cashu` nutzap, `uplink-wallet/lsp`).
  - Two-credential split scaffolding (spend-capable vs receive-only flag).
  - Correct the README "A5/A8 complete" overstatement.
- **Depends on:** Phase 1 (parallel to Phase 2).
- **ADRs:** ADR-U-007 — WalletProvider abstraction + NWC adapter.
- **Acceptance:** pay-to-recipient works through the trait via ≥1 adapter; grep shows no
  `todo!()` / `unimplemented!()` in production paths.

### Phase 4 — NFC client + on-device tag provisioning (pure-Rust, self-contained)
**Goal:** Read tags at tap-time and **program** NTAG 424 DNA SDM on-device, no other app, $0.
- **Deliverables:** new `crates/uplink-ntag424` (platform-agnostic: APDU build, SDM key
  derivation, AES-CMAC, SUN/SDM verify) with a `transceive` shim trait; Android `IsoDep`
  transceive binding via Tauri/JNI (no `ntag424-java` dependency — FOSS, self-contained);
  in-app provisioning screen (write NDEF URL `uplink://attendance?office=…&tag=…` + program
  SDM keys); tap-read deep-links into attendance mode.
- **Depends on:** Phase 1; consumes the session model from Phase 2.
- **ADRs:** ADR-U-009 — NTAG 424 SDM provisioning + verification contract.
- **Acceptance:** the Pixel programs a blank NTAG 424 with SDM; a second tap reads the
  SDM-mirrored URL; local verify passes (full server verify lands in Phase 6).

---

## Track 2 — New backend services (isolated phases)

### Phase 5a — External-credential identity *(client-only; ship first)*
**Goal:** Lowest-friction onboarding with no backend — users bring existing credentials.
- **Deliverables:** Bring-Your-Own-Credential onboarding (Lightning Address PRIMARY; NWC
  receive + **spend** [MVP spend rail]; LNC spend [LND-direct, transport gated]; optional
  npub / NIP-05 link; mnemonic create/restore demoted); external-credential model + redacted
  `CredentialMeta`; encrypted local persistence in `PlatformStore` (no external DB); NWC
  `spend_capable=true` via NIP-47 `pay_invoice` (opt-in `connect_receive_only` available);
  `LncProvider: WalletProvider` with `pay_invoice` gated behind `Unavailable`; persisted,
  user-configurable `RelayConfig` (default Damus / Primal / nos.lol); single-unlock session
  (passphrase held in native `Session` state, `lock_session` on sign-out); Settings wallet +
  relay hub. **Secrets never cross to the UI** (BOUNDARY.md, ADR-U-006).
- **Depends on:** Phase 3 (resolver + credential-split contracts).
- **ADRs:** ADR-U-010 — external-credential identity + relay-as-storage.
- **Acceptance:** a user onboards with a Lightning Address alone; NWC receive + balance +
  spend works (no stranded sats); an LNC pairing phrase persists and is capability-flagged
  spend-capable with the direct transport gated; relays are reconfigurable and persisted;
  the device password is entered once per session and never returned to the UI.

### Phase 5b — Hosted vanity identity service *(deferred; backend)*
**Goal:** Stand up the receive-routing identity backend for app-issued vanity addresses.
- **Deliverables:** Postgres + identity/wallet tables (normalized username, pubkey,
  receive-only routing fields, encrypted credential, timestamps, revocation);
  `POST /identity/register`; `GET /.well-known/nostr.json` (NIP-05);
  `GET /.well-known/lnurlp/<user>` (LNURL-pay); `GET /lnurl/callback` (mints BOLT11 via a
  **receive-only** credential); stable safe error codes; **identity backend can never spend.**
  Added as an additive resolver behind the same `RecipientAddress` interface (no client call-
  site changes).
- **Depends on:** Phase 5a (external-credential model + resolver interface).
- **ADRs:** ADR-U-010 (§3 5a/5b boundary); a follow-up ADR covers the hosted service schema.
- **Acceptance:** a username resolves via NIP-05 and an LNURL-pay invoice is minted by a
  credential with no spend authority (verified).

### Phase 6 — Private relay + attendance/payout backend
**Goal:** Authenticated ingress and the deterministic attendance/payout state machine.
- **Deliverables:** private OpenAgents relay with **NIP-42 auth**, pubkey allowlist, kind
  policy, raw-event retention, separate user-vs-backend writer authorization; relay
  consumer/normalizer; attendance service implementing the docs' 7-step validation order;
  **server-side SDM/CMAC verification** (M2); server-side stream scheduler (session-gated
  6-min payouts, re-check before each interval); attendance/presence/admin event kinds with
  schema versioning; tables `attendance_events_raw`, `attendance_sessions`, `stream_intervals`,
  `office_tags`, `relay_auth_keys`.
- **Depends on:** Phases 4 (provisioned tags) + 5 (identity/Postgres).
- **ADRs:** ADR-U-011 — private relay (NIP-42) + attendance event kinds.
- **Acceptance:** a valid tap opens a session; a second valid tap closes it; the backend (not
  the client) decides in/out; payouts run only for `open` sessions and never for
  `suspended` / `auto_closed` / `unknown`; raw events retained for audit.

### Phase 7 — Presence safety + admin
**Goal:** Anti-overpayment presence layer and operator tooling.
- **Deliverables:** presence service (binary `inside` / `outside` / `unknown`, no GPS trail);
  Android geofence in-app via **AOSP `LocationManager`** (no Fused Location);
  suspend-on-exit with grace period; no-location fallback guardrails (confirmation prompts);
  admin service (corrections, disputes, audit views); `presence_events`, `admin_corrections`
  tables.
- **Depends on:** Phase 6.
- **ADRs:** ADR-U-012 — presence/geofence privacy model.
- **Acceptance:** office exit suspends future intervals and notifies the worker; only binary
  presence state is stored.

---

## Track 3 — Platform + wallet completion (later; partially paid)

### Phase 8 — iOS enablement *(introduces $99/yr Apple Developer cost)*
- **Deliverables:** Tauri iOS build; NTAG 424 on iOS by adding an iOS `NFCISO7816Tag`
  transceive impl behind the **same `uplink-ntag424` shim** from Phase 4 (ACINQ
  `DnaCommunicator` only as a cross-check reference); Apple NFC entitlement; iOS
  CoreLocation geofencing.
- **Depends on:** Phase 4 (crate/shim) + Phase 7 (presence).
- **Acceptance:** an iOS device programs/reads/verifies a tag and runs the worker tap flow.

### Phase 9 — Wallet engine migration to LDK / MoneyDevKit
- **Deliverables:** swap/extend the wallet backend to an LDK + MoneyDevKit service satisfying
  the `WalletProvider` trait — **no changes to identity, relay, NFC, or payout-control
  layers** (the Phase 3 abstraction is what isolates this).
- **Depends on:** Phase 3.
- **Acceptance:** wallet backend swapped behind the trait with higher layers untouched.

---

## Dependency summary

```
P1 ─┬─ P2 ─┐
    ├─ P3 ─┼─ P5 ─ P6 ─ P7 ─ P8
    └─ P4 ─┘
           P3 ───────────────── P9
```

- **Identity (P5)** is isolated, gated on P3's contracts, feeding the relay/attendance
  backend (P6).
- **$0 milestone** = P1–P4 + P5–P7 on the de-Googled Android device. **Paid** only at P8 (iOS).
- Highest-value early test target: **P1 → P2 → P4** = a working Android tap-to-sign-in/out
  demo with local NTAG 424 verification before any backend exists.

## ADR map

| ADR | Title | Phase |
|---|---|---|
| ADR-U-006 | Platform pivot to Tauri v2 + FOSS distribution | (this plan) |
| ADR-U-007 | WalletProvider abstraction + NWC adapter | 3 |
| ADR-U-008 | Automation types + work-session model | 2 |
| ADR-U-009 | NTAG 424 SDM provisioning + verification contract | 4 |
| ADR-U-010 | Identity service + credential split | 5 |
| ADR-U-011 | Private relay (NIP-42) + attendance event kinds | 6 |
| ADR-U-012 | Presence / geofence privacy model | 7 |
