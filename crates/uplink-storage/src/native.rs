//! Native (host-cli) sled-backed encrypted KvStore.

use crate::crypto::{decrypt, encrypt};
use crate::kv::{KvError, KvStore};
use rand::RngCore;

pub struct SledStore {
    db: sled::Db,
    kek: [u8; 32],
}

impl SledStore {
    /// Open or create the database at `path`.
    ///
    /// If it's a new database, a salt is generated and stored.
    /// If existing, the salt is retrieved to derive the KEK from the passphrase.
    pub fn open(path: &std::path::Path, passphrase: &str) -> Result<Self, KvError> {
        let db = sled::open(path).map_err(|e| KvError::Backend(e.to_string()))?;

        // Use a special tree for unencrypted metadata
        let meta = db.open_tree("metadata").map_err(|e| KvError::Backend(e.to_string()))?;

        let salt = match meta.get("salt").map_err(|e| KvError::Backend(e.to_string()))? {
            Some(s) => {
                let mut salt = [0u8; 16];
                salt.copy_from_slice(&s);
                salt
            }
            None => {
                let mut salt = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut salt);
                meta.insert("salt", &salt).map_err(|e| KvError::Backend(e.to_string()))?;
                salt
            }
        };

        let kek = crate::crypto::derive_kek(passphrase, &salt);
        Ok(Self { db, kek })
    }
}

#[async_trait::async_trait]
impl KvStore for SledStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KvError> {
        match self.db.get(key).map_err(|e| KvError::Backend(e.to_string()))? {
            Some(blob) => Ok(Some(decrypt(&self.kek, &blob)?)),
            None => Ok(None),
        }
    }

    async fn put(&self, key: &str, value: &[u8]) -> Result<(), KvError> {
        let blob = encrypt(&self.kek, value)?;
        self.db
            .insert(key, blob.as_slice())
            .map_err(|e| KvError::Backend(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), KvError> {
        self.db
            .remove(key)
            .map_err(|e| KvError::Backend(e.to_string()))?;
        Ok(())
    }

    /// Existence check that does not decrypt — works without the correct
    /// passphrase, used for the "is an identity provisioned?" probe at launch.
    async fn exists(&self, key: &str) -> Result<bool, KvError> {
        self.db
            .contains_key(key)
            .map_err(|e| KvError::Backend(e.to_string()))
    }
}
