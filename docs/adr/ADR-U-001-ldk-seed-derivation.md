# ADR-U-001 — LDK Seed Derivation Path and Key Domain Separation

## Status
Accepted

## Date
2026-06-01

## Context
Uplink derives all keys from a single BIP-39 mnemonic. Three distinct key consumers
exist: NIP-06 Nostr identity, BIP-44 on-chain Bitcoin, and LDK `KeysManager`.

Using the same BIP-32 derivation path for two consumers would produce key reuse,
violating cryptographic security properties. All three paths must be hardened to
prevent cross-domain derivation.

## Decision

All key slots are derived from the BIP-39 mnemonic via BIP-32 using these hardened paths:

| Key slot | Path | Consumer | Notes |
|---|---|---|---|
| NIP-06 Nostr keypair | `m/44'/1237'/account'/0/0` | Nostr identity, DMs, event signing | NIP-06 standard |
| BIP-44 on-chain Bitcoin | `m/44'/0'/account'/0/0` | BDK wallet; receive address for funding | Standard BIP-44 BTC |
| LDK `KeysManager` seed | `m/535348'/0'/account'` | Lightning node, channel signing, HTLC routing | See rationale below |

### LDK seed path rationale: `m/535348'/0'/account'`
- `535348` = ASCII "SSH" — matches conventions used in several LDK sample wallets
  and BDK Lightning integration examples.
- This path is hardened at every component, preventing public-key-space derivation
  across domain boundaries.
- The 32-byte `KeysManager` seed is the SHA-512 truncated first half of the
  derived child private key bytes. See `crates/uplink-identity/src/derivation.rs`.
- The `account` index comes from `UplinkIdentity::account_index()`, which defaults
  to 0 for the primary identity and increments for child wallets.

### Why NOT the NIP-06 or BIP-44 paths for LDK
Re-using `m/44'/1237'` (NIP-06) or `m/44'/0'` (BIP-44) for LDK would create:
- Identical secret material for two systems with different threat models.
- Unpredictable cross-protocol key inference if either standard is extended.

## Consequences
- A single BIP-39 mnemonic backs all three key domains. Backup = mnemonic only.
- On-chain funds (BIP-44) are always recoverable even if the LDK channel state
  is lost (sweep from on-chain).
- The Spark/Breez BIP-44 path (`m/44'/0'`) is retained for on-chain during
  the migration window; once Spark is fully deprecated the BIP-44 slot becomes
  the sole on-chain consumer.
- Uplink does NOT share this ADR with the main OA repo — OA's equivalent is
  ADR-0010 (pending). Deliverable B integration PR aligns the OA `UnifiedIdentity`
  with this decision.

## References
- `crates/uplink-identity/src/derivation.rs` (implementation)
- BIP-32, BIP-39, BIP-44
- NIP-06 (Nostr key derivation from BIP-39)
- LDK sample wallet: `lightningdevkit/ldk-sample`
