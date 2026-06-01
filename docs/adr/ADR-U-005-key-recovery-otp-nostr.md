# ADR-U-005 — Key Recovery via Nostr OTP Challenge

## Status
Accepted

## Date
2026-06-01

## Context
Browser IndexedDB storage is tied to the origin. If a user clears their browser's
site data, or accesses Uplink from a new device, the encrypted local state is lost.

The mnemonic is the primary recovery path (write it down). However, Uplink also
encrypts its database with a passphrase-derived KEK. If the user forgets the
passphrase but still controls their Nostr nsec, we can provide a secondary
account recovery flow.

## Decision

### Recovery threat model
- **Primary recovery:** Mnemonic phrase (the BIP-39 mnemonic is the single source
  of truth for all keys). User restores via `restoreIdentity(mnemonic)` on any device.
- **Secondary recovery (this ADR):** Passphrase recovery for users who have the
  mnemonic but lost the encrypted DB KEK or need to migrate to a new device.

### Nostr OTP challenge flow

1. User visits Uplink on new device and clicks "Recover account".
2. User enters their mnemonic OR their npub (if they prefer to prove control differently).
3. Uplink publishes a **kind-9903** event (`uplink_otp_recovery`) to the user's
   configured relays, NIP-44 encrypted to the user's npub, with content:
   ```json
   { "otp": "<6-digit code>", "challenge_id": "<uuid>", "expires_at": <unix_ts> }
   ```
4. The user opens **any other Nostr client** (Amethyst, Damus, Snort…) that holds
   their nsec. They decrypt the event (client must support NIP-44) and read the OTP.
5. The user enters the OTP in Uplink's recovery screen.
6. On match: Uplink considers the user's identity verified and proceeds with
   wallet re-derivation from the mnemonic + new device passphrase.

### What is recovered
- The mnemonic (user must provide or already have loaded it).
- A new AES KEK is derived from the new device + new passphrase.
- Stream policies (kind-30901 events) are re-fetched from Nostr relays.
- LDK channel state must be recovered from the LSP's record or swept on-chain.

### Security properties
- The OTP has a 6-digit code space (1,000,000) with a 5-minute TTL and single-use.
- Rate limiting: Uplink allows at most 3 OTP requests per npub per hour (client-enforced).
- The OTP event is NIP-44 encrypted; a relay operator cannot read it.
- Proving control of the Nostr nsec is the proof of identity (cannot impersonate
  without the private key).
- This does NOT recover a lost mnemonic. The mnemonic backup is irreplaceable.

### Why Nostr OTP (not email/SMS)
- Uplink has no server-side infrastructure in Deliverable A; everything is client-side.
- Nostr relays are the existing communication layer.
- Proving nsec control via any Nostr client is natural for the target user base.
- No phone number or email address is required — fully self-sovereign.

## Consequences
- The user MUST have their nsec accessible in at least one other Nostr client to
  use the secondary recovery path.
- If the user has no other Nostr client, primary recovery (mnemonic) is the only option.
- The recovery relay must be online and reachable.

## References
- `crates/uplink-nostr/src/kinds.rs` (KIND_OTP_RECOVERY = 9903)
- `crates/uplink-storage/src/crypto.rs` (KEK derivation)
- NIP-44 v2 (encryption)
