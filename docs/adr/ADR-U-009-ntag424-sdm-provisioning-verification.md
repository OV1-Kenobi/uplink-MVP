# ADR-U-009 — NTAG 424 DNA SDM Provisioning + Verification Contract

## Status
Accepted

## Date
2026-06-09

## Context
Uplink's attendance flow is driven by tapping a fixed NTAG 424 DNA tag at the office
(ADR-U-008 in-office sessions). The tag must produce a **Secure Dynamic Messaging (SDM /
SUN)** URL that proves, cryptographically, that *this physical tag* was tapped *now*
(fresh read counter), so a forwarded/replayed URL cannot open or extend a paid session.

ADR-U-006 forbids any companion app and any non-FOSS / GMS dependency: the de-Googled
Pixel must **read AND program** the tag entirely in-app. The earlier plan leaned on
`ntag424-java` / ACINQ `DnaCommunicator`; that is replaced here by a **pure-Rust crate**
so the same crypto runs identically on Android (Tauri/JNI `IsoDep`), desktop (PC/SC), and
in unit tests, with no Java and no platform crypto.

This ADR governs the new `crates/uplink-ntag424` and how it consumes the Phase-2 session
model. Server-side re-verification and the authoritative attendance state machine remain
Phase 6 (ADR-U-011); this is the **client-side** programming + local-verify contract.

## Decision

### 1. `uplink-ntag424` is platform-agnostic, transport-injected
The crate contains **no I/O**. All tag communication goes through a single shim:

```
pub trait Transceive { fn transceive(&mut self, apdu: &[u8]) -> Result<Vec<u8>, Ntag424Error>; }
```

The Android binding (Tauri command → JNI → `IsoDep.transceive`), a desktop PC/SC binding,
and the test `MockTransport` all implement `Transceive`. This mirrors the Phase-3
`Nip47Transport` / `LnurlClient` shim pattern and keeps the crypto unit-testable.

### 2. AES crypto core (NXP AN12196, NT4H2421Gx)
Implemented from primitives (`aes` + `cmac`), AES mode only for the MVP (LRP deferred):
- **AES-CMAC** (RFC 4493), verified against the RFC test vectors.
- **SDM session keys** from `SDMFileReadKey`: `SV2 = 3Ch C3h 00h 01h 00h 80h || UID ||
  SDMReadCtr`, `SesSDMFileReadMACKey = CMAC(KSDMFileRead, SV2)`.
- **SDMMAC** = `CMAC(SesSDMFileReadMACKey, mac_input)` truncated to the 8 odd-indexed
  bytes (NXP truncation).
- **PICCData decrypt**: `AES-CBC-Decrypt(KSDMMetaRead, IV=0, encPICCData)` →
  `PICCDataTag || UID(7) || SDMReadCtr(3, LSB-first) || padding`.

The end-to-end chain is pinned by a **known-answer acceptance test** against the public
AN12196 all-zero-key example (`picc_data=EF963FF7828658A599F3041510671E88`,
`cmac=94EED9EE65337086`): local verify must succeed and recover the UID + counter.

### 3. SDM URL model + local verifier
`SdmUrl::parse` extracts the `picc_data` (and optional `enc` + `cmac`) query parameters
and the byte offsets of the MAC input window. `SdmVerifier::verify(url, meta_key,
file_key)` runs decrypt → derive → CMAC-compare and returns `SdmVerification { uid,
read_ctr }` or `Ntag424Error::MacMismatch`. Counter monotonicity (replay rejection across
taps) is enforced by the caller against last-seen state (client hint now; authoritative in
Phase 6).

### 4. On-device provisioning sequence
`provision.rs` builds the ISO 7816-4 APDUs to turn a blank (factory all-zero) tag into an
SDM tag, sequenced over `Transceive`: `ISOSelectFile (NDEF app)` → `AuthenticateEV2First`
(AES, RndA/RndB, derive `SesAuthENC/MACKey`) → optional `ChangeKey` → `ChangeFileSettings`
(enable SDM: PICCData-encrypted + SDMMAC, set mirror offsets) → `WriteData` (the NDEF URL
template with `{picc}`/`{mac}` placeholders). APDU builders and the NDEF URL template are
pure functions; the authenticated-session sequencing is exercised with `MockTransport`.

### 5. Tap → session toggle (consumes ADR-U-008)
A verified tap maps to a work-session transition without the client deciding policy:
`tap_action(session_open: bool) -> TapAction { Open | Close }` toggles, and a helper drives
`Scheduler::open_session` / `close_session` (Phase-2 API). The backend remains the
authority in Phase 6; the client toggle is a UX convenience stamped with the verified
`(uid, read_ctr)`.

### 6. No `todo!()`; honest gating
Parts that require a live secure channel against hardware we cannot exercise in CI
(e.g. `ChangeKey` rollover) return typed `Ntag424Error` values, never `todo!()`/
`unimplemented!()` (AGENTS.md / ADR-U-007 §5).

## Consequences
- **Positive:** one pure-Rust crypto core for read + program across Android/desktop/tests;
  no Java, no companion app, FOSS-clean; local verify is a hardware-independent
  known-answer test; the verifier output feeds the Phase-2 session model directly.
- **Negative / deferred:** LRP mode, `ChangeKey` rollover, and full secure-channel
  command encryption are scaffolded/gated, not hardware-validated here; the JNI/PC-SC
  `Transceive` bindings live at the platform boundary and are added with the Android build.
- **Migration:** new crate; additive to the workspace. No existing crate changes except
  adding `uplink-ntag424` as a member and an optional dep where the tap toggle is wired.

## References
- NXP **AN12196** "NTAG 424 DNA and NTAG 424 DNA TagTamper features and hints"; NT4H2421Gx
  data sheet; RFC 4493 (AES-CMAC).
- `docs/plans/uplink-pivot.md` — Phase 4
- `docs/adr/ADR-U-006-platform-pivot-tauri-foss-distribution.md` — FOSS / self-contained tags
- `docs/adr/ADR-U-008-automation-types-work-session-model.md` — work-session model
- ADR-U-011 (forthcoming) — server-side SDM re-verification + attendance state machine
