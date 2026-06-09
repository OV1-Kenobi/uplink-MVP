//! Hosted vanity-identity service core (Phase 5b): deterministic, receive-only.
//!
//! Provides NIP-05 resolution and LNURL-pay (LUD-06) address shaping over an injected
//! [`IdentityStore`]. The core never holds spend keys and performs no I/O of its own — the
//! Postgres/HTTP edge supplies the store and mints invoices (ADR-U-010, ADR-U-011). All
//! business logic here is pure and unit-testable; only the store is injected.

pub mod error;
pub mod lnurl;
pub mod nip05;
pub mod store;
pub mod username;

pub use error::IdentityServiceError;
pub use store::{IdentityRecord, IdentityStore, PubkeyHex, ReceiveRouting};
pub use username::NormalizedUsername;

/// Register a new vanity identity: validate inputs, enforce availability, persist.
///
/// Fails with [`IdentityServiceError::UsernameTaken`] if a **live** record already holds the
/// name, or [`IdentityServiceError::Revoked`] if a revoked record holds it (re-registration
/// of a revoked name is disallowed for now — ADR-U-011 §7 audit posture).
pub async fn register_identity(
    store: &dyn IdentityStore,
    username: &str,
    pubkey: &str,
    routing: ReceiveRouting,
    now: i64,
) -> Result<IdentityRecord, IdentityServiceError> {
    let username = NormalizedUsername::parse(username)?;
    let pubkey = PubkeyHex::parse(pubkey)?;
    if let Some(existing) = store.get_by_username(&username).await? {
        return Err(if existing.is_live() {
            IdentityServiceError::UsernameTaken
        } else {
            IdentityServiceError::Revoked
        });
    }
    let record = IdentityRecord {
        username,
        pubkey,
        routing,
        created_at: now,
        revoked_at: None,
    };
    store.insert(&record).await?;
    Ok(record)
}

/// Resolve a NIP-05 `?name=<username>` query into the `names` document.
///
/// An unparseable or unknown name yields an empty `names` map (no error, no leak).
pub async fn resolve_nip05(
    store: &dyn IdentityStore,
    username: &str,
) -> Result<serde_json::Value, IdentityServiceError> {
    let Ok(username) = NormalizedUsername::parse(username) else {
        return Ok(nip05::nip05_response(None));
    };
    let record = store.get_by_username(&username).await?;
    Ok(nip05::nip05_response(record.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockStore {
        rows: Mutex<HashMap<String, IdentityRecord>>,
    }

    #[async_trait]
    impl IdentityStore for MockStore {
        async fn get_by_username(
            &self,
            username: &NormalizedUsername,
        ) -> Result<Option<IdentityRecord>, IdentityServiceError> {
            Ok(self.rows.lock().unwrap().get(username.as_str()).cloned())
        }

        async fn insert(&self, record: &IdentityRecord) -> Result<(), IdentityServiceError> {
            self.rows
                .lock()
                .unwrap()
                .insert(record.username.as_str().to_string(), record.clone());
            Ok(())
        }
    }

    fn la() -> ReceiveRouting {
        ReceiveRouting::LightningAddress { user: "alice".into(), domain: "ex.com".into() }
    }

    #[tokio::test]
    async fn registers_then_rejects_taken() {
        let store = MockStore::default();
        let pk = "a".repeat(64);
        let rec = register_identity(&store, "Alice", &pk, la(), 100).await.unwrap();
        assert_eq!(rec.username.as_str(), "alice");

        let err = register_identity(&store, "alice", &pk, la(), 200).await.unwrap_err();
        assert_eq!(err.code(), "username_taken");
    }

    #[tokio::test]
    async fn rejects_invalid_username_and_pubkey() {
        let store = MockStore::default();
        let pk = "a".repeat(64);
        assert_eq!(
            register_identity(&store, "bad name", &pk, la(), 0).await.unwrap_err().code(),
            "username_invalid"
        );
        assert_eq!(
            register_identity(&store, "bob", "xyz", la(), 0).await.unwrap_err().code(),
            "pubkey_invalid"
        );
    }

    #[tokio::test]
    async fn resolves_nip05_hit_and_miss() {
        let store = MockStore::default();
        let pk = "a".repeat(64);
        register_identity(&store, "alice", &pk, la(), 0).await.unwrap();

        let hit = resolve_nip05(&store, "alice").await.unwrap();
        assert_eq!(hit["names"]["alice"], pk);

        let miss = resolve_nip05(&store, "nobody").await.unwrap();
        assert_eq!(miss["names"].as_object().unwrap().len(), 0);

        // Unparseable name must not error.
        let bad = resolve_nip05(&store, "bad name").await.unwrap();
        assert_eq!(bad["names"].as_object().unwrap().len(), 0);
    }
}
