//! NIP-05 `/.well-known/nostr.json` response builder (Phase 5b).
//!
//! Pure: maps an optional resolved record into the NIP-05 `names` document. A miss (or a
//! revoked identity) yields an empty `names` map — never an error and never a leak.

use serde_json::{json, Value};

use crate::store::IdentityRecord;

/// Build the NIP-05 response for a `?name=<username>` query.
///
/// Returns `{"names": {username: pubkey}}` for a live record, else `{"names": {}}`.
#[must_use]
pub fn nip05_response(record: Option<&IdentityRecord>) -> Value {
    match record {
        Some(r) if r.is_live() => json!({
            "names": { r.username.as_str(): r.pubkey.as_str() }
        }),
        _ => json!({ "names": {} }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{PubkeyHex, ReceiveRouting};
    use crate::username::NormalizedUsername;

    fn record(revoked_at: Option<i64>) -> IdentityRecord {
        IdentityRecord {
            username: NormalizedUsername::parse("alice").unwrap(),
            pubkey: PubkeyHex::parse(&"a".repeat(64)).unwrap(),
            routing: ReceiveRouting::LightningAddress {
                user: "alice".into(),
                domain: "ex.com".into(),
            },
            created_at: 0,
            revoked_at,
        }
    }

    #[test]
    fn live_record_yields_name_map() {
        let r = record(None);
        let v = nip05_response(Some(&r));
        assert_eq!(v["names"]["alice"], "a".repeat(64));
    }

    #[test]
    fn miss_and_revoked_yield_empty_map() {
        assert_eq!(nip05_response(None), json!({ "names": {} }));
        let revoked = record(Some(10));
        assert_eq!(nip05_response(Some(&revoked)), json!({ "names": {} }));
    }
}
