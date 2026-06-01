# ADR-U-003 — Nostr Receipt Event Kinds for Stable-Stream Payments

## Status
Accepted

## Date
2026-06-01

## Context
Uplink needs Nostr event kinds to:
1. Declare a recurring streaming-sats flow (addressable, replaceable when policy changes).
2. Record a per-period payment completion as an immutable audit receipt.
3. Signal stream revocation from a parent account.
4. Support key recovery OTP delivery.

Existing kinds (NIP-57 zap 9735, NIP-61 nutzap) are NOT used as receipts because:
- Kind 9735 is published by the recipient's LNURL provider, not the sender.
- Uplink payments land as Stable-Channel balance credits, not as LNURL-resolved invoices.
- Re-using kind 9735 would conflate Uplink's receipt audit trail with NIP-57 zaps.

## Decision

### Kind 30901 — `stable_stream` (parameterized replaceable)
Declares a recurring streaming-sats flow. Addressable via `d` tag = stream UUID hex.

**Mandatory tags:**
| Tag | Value |
|---|---|
| `d` | stream UUID hex (stable address) |
| `p` | recipient npub hex |
| `amount` | msats per period (string) |
| `period` | period in seconds (string) |
| `currency` | `"USD"` (Stable-Channel denomination) |
| `lsp` | LSP Lightning node pubkey hex |
| `start` | Unix timestamp of first period (string) |

**Optional tags:** `end`, `max_total_sats`, `memo`

### Kind 9901 — `stable_stream_receipt` (regular, immutable)
Records a single completed period payment. One event per leg per period.

**Mandatory tags:**
| Tag | Value |
|---|---|
| `e` | stream declaration event ID (root reference) |
| `p` | recipient npub hex |
| `amount` | msats paid (string) |
| `period_index` | period number (string, 0-based) |
| `receipt_hash` | SHA-256 of the canonical receipt (see `uplink-receipts`) |
| `lsp_preimage` | Lightning payment preimage hex (proof of payment) |

### Kind 9902 — `stable_stream_revocation`
Published by parent to revoke a stream or delegation. NIP-44 encrypted to recipient.

**Tags:** `e` (stream event ID), `p` (child npub), `reason` (human-readable string)

### Kind 9903 — `uplink_otp_recovery`
OTP delivery event for key recovery flow (ADR-U-005). NIP-44 encrypted to user's npub.

**Tags:** `p` (requesting npub), `challenge_id` (UUID hex)

## Canonical receipt hash
The SHA-256 receipt hash in kind-9901's `receipt_hash` tag uses this format:

```
SHA-256( idempotency_key ":" stream_id ":" period_index ":" leg_index ":" msats_paid ":" preimage_hex )
```

This matches the `PaymentAttemptReceiptV1` canonicalization in OA's `crates/neobank`
so the Deliverable B PR is a format-compatible swap.

## Consequences
- Relays must store kind-30901 events (addressable) and kind-9901 events (regular).
- The kind numbers (30901, 9901, 9902, 9903) are application-specific and do not
  conflict with any published NIPs as of 2026-06-01.
- These kinds will be submitted as a NIP draft once the wire contract stabilizes.

## References
- `crates/uplink-nostr/src/kinds.rs`
- `crates/uplink-receipts/src/lib.rs`
- NIP-33 (addressable events), NIP-57 (zaps), NIP-61 (nutzaps)
