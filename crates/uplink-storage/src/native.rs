//! Native (host-cli) sled-backed encrypted KvStore.

use crate::crypto::{decrypt, encrypt};
use crate::kv::{KvError, KvStore};

pub struct SledStore {
    db: sled::Db,
    kek: [u8; 32],
}

impl SledStore {
    /// Open or create the database at `path` with the given 32-byte key-encryption key.
    pub fn open(path: &std::path::Path, kek: [u8; 32]) -> Result<Self, KvError> {
        let db = sled::open(path).map_err(|e| KvError::Backend(e.to_string()))?;
        Ok(Self { db, kek })
    }
}

impl KvStore for SledStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KvError> {
        match self.db.get(key).map_err(|e| KvError::Backend(e.to_string()))? {
            Some(blob) => Ok(Some(decrypt(&self.kek, &blob)?)),
            None => Ok(None),
        }
    }

    fn put(&self, key: &str, value: &[u8]) -> Result<(), KvError> {
        let blob = encrypt(&self.kek, value)?;
        self.db
            .insert(key, blob.as_slice())
            .map_err(|e| KvError::Backend(e.to_string()))?;
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), KvError> {
        self.db
            .remove(key)
            .map_err(|e| KvError::Backend(e.to_string()))?;
        Ok(())
    }
}
