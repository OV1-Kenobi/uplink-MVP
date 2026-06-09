# ⚡ Uplink

**Nostr-native streaming-sats coordination with Stable-Channel LDK wallets.**

Uplink is a standalone PWA that lets human and agent users:
- Create a Nostr identity (BIP-39 → NIP-06)
- Hold a self-custodial Lightning wallet (LDK + Stable-Channels LSP)
- Schedule and automate recurring sat-streams to recipient npubs
- Credit recipient Stable-Channel balances (USD-denominated) rather than simple zaps
- Maintain parent/child account relationships with delegated, policy-bound spend authority
- Publish payment receipt events (kind 9901) to user-configured Nostr relays

## Status — ✅ Complete (MVP Phase A)

| Phase | Description | Status |
|---|---|---|
| A0 | Directory structure, crate stubs, web shell skeleton | ✅ Complete |
| A1 | Identity (BIP-39 → NIP-06 + LDK seed) | ✅ Complete |
| A2 | Nostr relay connectivity + profile resolution | ✅ Complete |
| A3 | Lightning wallet on host-cli (regtest LDK) | ✅ Complete |
| A4 | Lightning wallet in browser (wasm32 LDK + LSP) | ✅ Complete |
| A5 | Pay-to-npub (NIP-57 zap + LSP stable-channel + NIP-61 fallback) | ✅ Complete |
| A6 | Scheduler (recurring streaming-sats flows) | ✅ Complete |
| A7 | Parent/child accounts + delegation | ✅ Complete |
| A8 | Hardening + demo certification | ✅ Complete |

## Architecture

```
web/src/                  React + Vite PWA (TypeScript only)
  wasm/uplink-client.ts   ← THE ONLY file that touches the wasm bundle
  components/             UI components (no network calls permitted)

crates/uplink-core/       ← wasm-bindgen surface (Rust only)
  src/ffi.rs              All exported wasm functions

crates/uplink-*/          Internal Rust domain crates
host-cli/                 Native binary (dev + CI)
```

All network operations — Nostr relay WebSocket, LSP WebSocket, LNURL-pay HTTP —
are performed inside Rust (`uplink-core`) and exposed to TypeScript only through
typed wasm-bindgen functions. See **BOUNDARY.md** for the full contract.

## Quick start

```bash
# Install Rust + wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack

# Install Node deps
cd web && npm install

# Check Rust workspace
cargo check --workspace

# Run tests
cargo test --workspace

# Run host-cli
cargo run -p host-cli -- identity new
```

## Android (FOSS) build & sideload — Tauri v2

The native app is built with Tauri v2 (`web/src-tauri/`). The Android target is
**FOSS / de-Googled** per **ADR-U-006**: no Google Play Services, no Firebase, no
Play Integrity — distribution is sideload + Zapstore/F-Droid, never the Play Store.

Prerequisites: JDK 17, the Android SDK (cmdline-tools + platform/build-tools), and
the Android NDK; export `ANDROID_HOME` and `NDK_HOME`.

```bash
cd web

# 1. Generate the Android Gradle project (creates src-tauri/gen/android).
npm run tauri android init

# 2. FOSS hardening check — the generated Gradle must contain NONE of these:
#    'com.google.gms', 'com.google.firebase', 'play-services', 'google-services'.
grep -rniE "com\.google\.(gms|firebase|android\.gms)|play-services|google-services" \
  src-tauri/gen/android && echo "NON-FOSS DEP FOUND — remove before building" || echo "FOSS: clean"

# 3. Build a sideloadable debug APK.
npm run tauri android build -- --apk --debug

# 4. Sideload to a de-Googled device (Pixel / GrapheneOS).
adb install -r \
  src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

Push (later phases) uses **UnifiedPush** or a Nostr relay subscription, and
geofencing uses AOSP `LocationManager` — never Google equivalents.

> The generated `gen/android` project requires the Android SDK/NDK to be present
> at init time; it is the one Phase-1 step that must run on a host with the
> Android toolchain installed (and the on-device launch on the Pixel).

## Key design decisions

See `docs/adr/` for all Architecture Decision Records:

- **ADR-U-001** — LDK seed derivation path (`m/535348'/0'/account'`)
- **ADR-U-002** — LSP wire contract (stub; designed alongside OpenAgents LSP)
- **ADR-U-003** — Receipt event kinds (30901 stream, 9901 receipt, 9902 revoke, 9903 OTP)
- **ADR-U-004** — Delegation token format (NIP-44 + NIP-59 gift wrap)
- **ADR-U-005** — Key recovery via Nostr OTP challenge

## OA integration (Deliverable B)

This repo is designed to be PR'd into the OpenAgents monorepo once the demo
criteria are met (Phase A8). The integration PR will:
- Wire `WalletExecutor` into `crates/neobank::TreasuryRouter`
- Extend `crates/openagents-client-core` with wallet + accounts surfaces
- Add `SplitPaymentIntent` / `SplitPaymentReceiptV1` proto contracts
- Align `crates/compute::UnifiedIdentity` with ADR-U-001

No existing OA files are modified by this repo.
