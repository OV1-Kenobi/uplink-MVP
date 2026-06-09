# ADR-U-006 — Platform Pivot to Tauri v2 + FOSS / De-Googled Distribution

## Status
Accepted

## Date
2026-06-09

## Context
The MVP (Phases A0–A8) shipped as a Wasm + React PWA. Two product requirements make a
PWA insufficient:

1. **NTAG 424 DNA** attendance tags require raw ISO-DEP/APDU exchange for Secure Dynamic
   Messaging (SDM) provisioning and AES-CMAC verification. The Web NFC API (`NDEFReader`)
   exposes only NDEF records — it cannot transceive APDUs, so a PWA can neither **program**
   nor cryptographically **verify** these tags.
2. Reliable background **geofencing** and native NFC are not available to web contexts.

Additional hard constraints from the target deployment:

- **Distribution will NOT use the Google Play Store.** Primary channel is **Zapstore**
  (Nostr-native APK signing), with **direct APK download** and **F-Droid** as additional
  channels.
- The test/reference device is a **de-Googled Pixel 10 (GrapheneOS-class)**: **Google Play
  Services are absent**. Anything depending on Play Services (FCM push, Fused Location,
  Play Integrity, Maps) is unavailable.
- The final deliverable must **read and program tags self-contained** — no companion app,
  no proprietary provisioning tool.

## Decision

### 1. Adopt Tauri v2 as the application shell
Reuse the existing React UI (`web/`) and compile the `uplink-*` crates **natively** (the
`host-cli` already proves a native build). Tauri uses the system WebView (GrapheneOS ships
one), is fully open-source, and exposes native NFC/location via Rust plugins. The Netlify
web/desktop dashboard build is kept alive in parallel as a non-NFC test surface.

### 2. NTAG 424 logic lives in a pure-Rust crate (`uplink-ntag424`)
To be self-contained and FOSS, all NTAG 424 protocol logic — APDU construction, SDM key
derivation, AES-CMAC, SUN/SDM verification — is implemented in a **platform-agnostic Rust
crate** with **no proprietary dependencies**. The only platform-specific surface is a thin
`transceive(&[u8]) -> Vec<u8>` shim:

- **Android:** `android.nfc.tech.IsoDep` via the Tauri/JNI bridge.
- **iOS (later):** `NFCISO7816Tag` via Core NFC.

This **replaces** the earlier plan to depend on `ntag424-java` / ACINQ `DnaCommunicator`
as the primary implementation. Those remain optional cross-check references only. The crate
exposes both **read/verify** and **program/provision** so the app needs no other tool.

### 3. No Google Play Services anywhere
- **Geofencing:** AOSP `LocationManager` (GPS/network providers) with in-app proximity
  math — never Fused Location. Presence is stored as binary `inside`/`outside`/`unknown`
  (no GPS trail), per the privacy model deferred to ADR-U-012.
- **Push:** **UnifiedPush** (or Nostr relay subscription) — never FCM.
- **No** Play Integrity / SafetyNet, Maps SDK, or any `com.google.android.gms` dependency.

### 4. Distribution
- **Primary:** Zapstore — APKs are signed and the release event published to Nostr.
- **Secondary:** direct signed-APK download.
- **Stretch:** F-Droid. F-Droid requires reproducible builds from source with no non-free
  dependencies; the FOSS constraints above are necessary preconditions, so we keep the
  build F-Droid-eligible from the start even though Zapstore ships first.

## Consequences
- **Positive:** One Rust core across CLI, desktop, Android, and (later) iOS. Tag read +
  program are first-class and self-contained. The build stays installable on GrapheneOS and
  eligible for FOSS stores. No Google dependency to audit or remove later.
- **Negative / cost:** Tauri Android toolchain (Android SDK/NDK) added to CI. iOS later
  introduces the $99/yr Apple Developer cost (Phase 8). Reproducible-build hardening for
  F-Droid is deferred but must not be regressed by adding non-free deps.
- **Boundary change:** Under Tauri, network operations (Nostr/LSP/LNURL) run in the native
  Rust core, not behind a single Wasm wrapper file. `BOUNDARY.md` must be revised so its
  "no direct network from the UI" rule targets the Tauri command boundary instead of the
  wasm-bindgen file. The wasm/Netlify surface keeps the original contract.
- **Supersedes:** the PWA-only deployment stance implied by the A-phase README. This ADR
  does not change identity, wallet, or delegation contracts (ADR-U-001/003/004/005).

## References
- `docs/plans/uplink-pivot.md` — the sequenced 9-phase migration plan.
- `BOUNDARY.md` — to be revised for the Tauri native↔UI boundary (Phase 1).
- ADR-U-007..U-012 — per-phase decisions authored as each phase begins.
- NTAG 424 DNA SDM (NXP AN12196); NIP-47 (NWC); NIP-42 (relay auth); UnifiedPush.
