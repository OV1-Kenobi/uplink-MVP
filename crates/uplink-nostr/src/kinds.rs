//! Custom Nostr event kinds defined by Uplink.
//!
//! Rationale and full tag schemas: docs/adr/ADR-U-003-receipt-event-kind.md

use nostr::Kind;

/// Kind 30901 — `stable_stream`
///
/// Parameterized replaceable event (NIP-33). Declares a recurring streaming-sats
/// policy that credits the recipient's Stable-Channel balance on each period tick.
///
/// `d` tag: stream_id (UUID hex)
/// Tags:
///   ["p",  <recipient_npub>]
///   ["amount", <msats_per_period>]
///   ["period", <seconds>]
///   ["currency", "USD"]              // target stable-channel denomination
///   ["lsp", <lsp_node_pubkey_hex>]   // LSP that executes the channel credit
///   ["start", <unix_ts>]
///   ["end",   <unix_ts>]             // optional
///   ["max_total_sats", <sats>]       // optional hard cap
pub const KIND_STABLE_STREAM: Kind = Kind::Custom(30901);

/// Kind 9901 — `stable_stream_receipt`
///
/// Regular (immutable) event published by the sender after each successful
/// period payment. References the stream declaration and the OA receipt hash.
///
/// Tags:
///   ["e",  <stream_declaration_event_id>, "", "root"]
///   ["p",  <recipient_npub>]
///   ["amount", <msats_paid>]
///   ["period_index", <u64>]
///   ["receipt_hash", <sha256_hex>]   // canonical receipt hash (ADR-U-003)
///   ["lsp_preimage", <hex>]          // payment preimage from LSP (proof of payment)
pub const KIND_STABLE_STREAM_RECEIPT: Kind = Kind::Custom(9901);

/// Kind 9902 — `stable_stream_revocation`
///
/// Published by the parent to revoke a child delegation or terminate a stream.
/// NIP-44 encrypted to recipient.
///
/// Tags:
///   ["e", <stream_declaration_event_id>]
///   ["p", <child_npub>]
///   ["reason", <human_readable>]
pub const KIND_STABLE_STREAM_REVOCATION: Kind = Kind::Custom(9902);

/// Kind 9903 — `uplink_otp_recovery`
///
/// Ephemeral-ish event used in the key-recovery flow (ADR-U-005).
/// Published by the recovery service; NIP-44 encrypted to the user's npub.
///
/// Tags:
///   ["p", <requesting_npub>]
///   ["challenge_id", <uuid_hex>]
pub const KIND_OTP_RECOVERY: Kind = Kind::Custom(9903);
