# ADR-U-010 — External-Credential Identity + Relay-as-Storage (Phase 5a)

**Status:** Accepted
**Date:** 2026-06-09
**Supersedes (in part):** the Phase 5 "Identity service" scope is split into 5a (this ADR)
and 5b (hosted vanity identity, deferred). See `docs/plans/uplink-pivot.md`.

**Amended 2026-06-09 (§2, §6):** the original §2 made NWC receive-only and LNC the sole
(gated) spend path. That stranded received sats in dead-end wallets with no withdrawal
method — an unacceptable pattern. §2 is superseded below: **NWC is now the MVP spend +
receive rail** (its NIP-47 `pay_invoice` transport is real and unit-tested; AlbyHub / Breez /
LND-via-Alby all expose it). LNC-direct spend remains gated (§4). §6 adds the single-unlock
session model.

## Context

The original Phase 5 stood up a Postgres-backed identity service that issued vanity
NIP-05 + LNURL-pay addresses from an OpenAgents-hosted `.well-known` domain, minting
receive invoices via a receive-only credential. That requires a backend, a hosted domain,
and per-user provisioning before a single user can onboard.

For the MVP we want the lowest-friction onboarding possible and no server dependency.
Most target users already have a Lightning address, an NWC connection string, an LND node
(Lightning Node Connect), and/or a Nostr identity (npub / NIP-05). We let them bring those.

## Decision

### 1. Bring-Your-Own-Credential onboarding (client-only)
Onboarding accepts existing credentials instead of forcing key generation:

- **Lightning Address (PRIMARY).** `you@wallet.com` — pure receive routing, zero custody,
  resolved through the existing `RecipientAddress` / LNURL-pay path. A local Nostr keypair
  may be generated silently for event signing without a forced backup screen.
- **NWC connection string** — receive + **spend** (the MVP spend rail).
- **Lightning Node Connect (LNC)** pairing phrase — direct-LND spend; persisted and
  capability-flagged, transport gated (see §4).
- **npub / NIP-05** — optional identity link.
- **Create new mnemonic / Restore** — retained, demoted below the BYOC options.

### 2. Capability split: NWC = spend + receive (MVP spend rail), LNC = spend (gated)
*(Supersedes the original receive-only-NWC decision — see the amendment note above.)*

`WalletCapabilities.spend_capable` (ADR-U-007 §4) is the enforcement point.

| Credential | receive | spend | secret | enforcement |
|---|---|---|---|---|
| Lightning Address | ✅ | ❌ | no | routing only |
| NIP-05 / npub | ❌ | ❌ | no | identity only |
| NWC | ✅ + balance | ✅ | yes | `spend_capable=true`; NIP-47 `pay_invoice` rail |
| LNC | ✅ | ✅ | yes | LND-direct; transport gated (see §4) |

NWC is the MVP spend rail: AlbyHub, Breez, and LND-via-Alby all expose NIP-47, so a connected
wallet can both receive and spend — no sats are stranded. An opt-in `connect_receive_only`
constructor still exists (forces `spend_capable=false`, `pay_invoice` → `Declined`) for users
who deliberately want to link a wallet for receive only.

### 3. Storage: no external DB for 5a (relay-as-storage path)
The local encrypted `PlatformStore` (AES-256-GCM, sled — `uplink-storage`) is the source of
truth for all credentials and routing. No Postgres is introduced in 5a.

Cross-device restore is provided by an **encrypted relay-backup** interface: a NIP-44
self-encrypted event published to the user's configured relays. Constraints:

- **Spend secrets (LNC pairing phrase) are local-only and MUST NEVER be published**, even
  encrypted. Only non-secret receive/identity routing (LN address, npub, NIP-05) and, at
  most, the NWC URI behind explicit opt-in may be backed up.
- Relays are user-configurable (see §5); public relays are the MVP default and will be
  swapped for self-hosted private relays before production.

The hosted identity service + Postgres (`/.well-known/nostr.json`, LNURL-pay minting) moves
to **Phase 5b**, architected as an additive resolver behind the same `RecipientAddress`
interface so adding it later changes no client call sites.

### 4. LNC-direct transport is stubbed behind a clean interface
Spend is **not** blocked on this — NWC (§2) is the live spend rail. This section only gates
the *direct-LND* LNC transport, for users who want to spend straight from their own LND node
(and, later, an LDK-mirrored variant) without routing through NWC.

No mature pure-Rust LNC client exists (reference is Go→WASM: mailbox proxy + PAKE/brontide
`Noise_XK_secp256k1_ChaChaPoly_SHA256` + gRPC-over-WebSocket). Therefore 5a:

- captures and encrypts the 10-word pairing phrase at rest (native side only);
- exposes a `LncProvider: WalletProvider` whose `pay_invoice` returns a structured
  `ProviderError::Unavailable("LNC transport not yet wired")` — mirroring the LSP/Nutzap
  gating pattern (ADR-U-002), never `todo!()`/`unimplemented!()`.

Implementing the LNC-direct transport (and its LDK mirror) is deferred to a later phase;
until then, LND/LDK users spend via NWC.

### 5. Default relays
The MVP default relay set becomes three general-purpose public relays:

```
wss://relay.damus.io
wss://relay.primal.net
wss://nos.lol
```

The `wss://relay.openagents.com` placeholder leaves the MVP default set (re-added when the
production relay is live). The set is fully reconfigurable from Settings, persisted in the
encrypted store via `RelayConfig`.

### 6. Single-unlock session model
The device passphrase is the KEK for the at-rest `PlatformStore`. Rather than re-prompting on
every credential operation, the native layer holds the passphrase in **Tauri managed state
(`Session`)** for the duration of the app session:

- It is set once on `create_identity` / `restore_identity` / `current_identity` (unlock).
- Post-unlock commands (`export_mnemonic`, `connect_*`, `set_lightning_address`,
  `link_identity`, `list_credentials`, `disconnect_credential`) read the passphrase from the
  session — the UI no longer passes it per call.
- `lock_session` clears it (sign-out / app lock).

The passphrase **never crosses to the UI**: it lives only in the Rust process, mirroring how
the wasm core already holds the unlocked identity in memory. This is the native counterpart to
the wasm session and keeps the custody boundary intact.

## Custody invariant (unchanged)
NWC URIs, LNC pairing phrases, and the session passphrase are bearer secrets: encrypted at
rest (or held only in native process memory), kept on the native/Rust side, and **never
returned to the UI in plaintext**. The UI receives only redacted `CredentialMeta` (kind,
label, capability flags, timestamps) — the same boundary discipline the mnemonic follows
(BOUNDARY.md, ADR-U-006).

## Consequences

- **+** A user with a Lightning address onboards in one field, no backend, no backup screen.
- **+** Receive **and spend** work immediately via NWC (AlbyHub / Breez / LND-via-Alby), plus
  receive on the LN-address and LNC rails — no received sats are stranded.
- **+** Single unlock: the passphrase is entered once per session and stays in the Rust layer.
- **+** No server, domain, or DB to operate for the MVP; relay set is swappable to private.
- **−** Direct-LND (LNC) spend is unavailable until its transport ships; those users spend via
  NWC in the meantime.
- **−** Relay-backup of the NWC URI trusts relay operators with ciphertext; gated behind
  explicit opt-in and disabled by default; spend secrets are never eligible.
- **−** Two relay-default sources (Rust `relay.rs`, `SettingsPage.tsx`) must be reconciled
  to one persisted `RelayConfig`.
- **−** The in-memory session passphrase persists until `lock_session`; a future hardening
  step may add an idle-timeout auto-lock.

## References
- ADR-U-002 — LSP wire contract (stub/gating pattern)
- ADR-U-006 — Tauri pivot + custody boundary
- ADR-U-007 — WalletProvider abstraction + NWC adapter + credential split
- `docs/plans/uplink-pivot.md` — Phase 5a / 5b split
- BOUNDARY.md — Wasm / Tauri native custody boundary
