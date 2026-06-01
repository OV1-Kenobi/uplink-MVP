# ADR-U-004 — Delegation Token Format (Parent/Child Accounts)

## Status
Accepted

## Date
2026-06-01

## Context
Uplink supports parent/child wallet relationships where a parent can delegate bounded
spend authority to a child wallet. A child wallet may be operated by a human sub-account
or by an automated agent. Revocation must be immediate and verifiable on Nostr relays.

Design requirements:
1. The token must be cryptographically bound to the issuing parent's Nostr identity.
2. The token must be unreadable to third-party relay operators.
3. Revocation must not require synchronous communication with the parent.
4. The format must survive a relay outage (child can cache and verify locally).
5. Agent users must be able to verify a delegation without UI interaction.

## Decision

### Token format: signed Nostr event + NIP-44 encryption + NIP-59 gift wrap

1. **Inner event (kind TBD, tentatively 30402 or a private app kind):**
   The parent constructs a Nostr event containing the serialized `DelegationPolicy`
   (see `crates/uplink-nostr/src/delegation.rs`).
   - Content: `JSON(DelegationPolicy)`
   - Tags: `["p", child_npub_hex]`, `["token_id", uuid_hex]`, `["expires", unix_ts_str]`
   - Signed with the parent's nsec.

2. **NIP-44 encryption:** The inner event's JSON string is encrypted with NIP-44 v2
   using a shared secret derived from `parent_nsec × child_npub` (ECDH).
   Only the child can decrypt.

3. **NIP-59 gift wrap:** The encrypted payload is gift-wrapped (NIP-59) so that the
   relay event itself reveals neither parent nor child identity to observers.
   The gift-wrap event is published to the child's preferred relay.

4. **Local cache:** The child stores the decrypted, verified `DelegationToken` in
   `uplink-storage` (encrypted at rest). Verification is offline: check parent
   signature, expiry, and local revocation list.

### Revocation
- The parent publishes a kind-9902 event (defined in ADR-U-003) referencing the
  `token_id` of the delegation.
- The child's scheduler checks for revocation events before each `tick()`.
- The child's local cache marks the token `revoked = true` on first sight of a
  valid kind-9902; subsequent ticks skip all streams under that delegation.

### Why signed Nostr events (not a custom JWT or MACAROON)
- No additional crypto library required — Nostr signing is already in `uplink-nostr`.
- NIP-44 encryption gives forward secrecy and replay-resistance.
- NIP-59 gift wrap gives metadata privacy on relays.
- The revocation model (kind-9902) fits naturally into Nostr's event model.
- Agents and humans use the same verification path.

### Why NOT NIP-26 delegation tags
NIP-26 delegates Nostr *event signing* authority (publishing events as another key),
not *payment* authority. It is not suitable for wallet delegation.

## Consequences
- The delegation token cannot be forged without the parent's nsec.
- Revocation propagates in near-real-time via Nostr relay subscription.
- An offline child will not see revocation until it reconnects; the `expires_at`
  hard deadline provides a safety bound. Choose short expiry windows for high-value
  delegations.
- The implementation lives in `crates/uplink-nostr/src/delegation.rs`.

## References
- `crates/uplink-nostr/src/delegation.rs`
- `crates/uplink-accounts/src/extension.rs` (`ParentChildLink`, `DelegationPolicy`)
- NIP-44 v2 (encryption), NIP-59 (gift wrap)
