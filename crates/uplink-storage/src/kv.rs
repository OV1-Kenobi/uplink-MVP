//! `KvStore` trait — the single abstraction over all platform persistence adapters.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KvError {
    #[error("key not found: {0}")]
    NotFound(String),
    #[error("storage backend error: {0}")]
    Backend(String),
    #[error("encryption/decryption error")]
    Crypto,
    #[error("serialization error: {0}")]
    Serde(String),
}

/// An asynchronous, byte-oriented, encrypted key-value store.
///
/// Implementations must encrypt every value before persistence and
/// decrypt on retrieval. The encryption key is held by the store instance.
#[async_trait::async_trait]
pub trait KvStore: Send + Sync {
    /// Fetch and decrypt a value by key. Returns `None` if not found.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KvError>;

    /// Encrypt and persist a value. Overwrites existing.
    async fn put(&self, key: &str, value: &[u8]) -> Result<(), KvError>;

    /// Delete a key-value pair.
    async fn delete(&self, key: &str) -> Result<(), KvError>;

    /// Check existence without decrypting.
    async fn exists(&self, key: &str) -> Result<bool, KvError> {
        Ok(self.get(key).await?.is_some())
    }
}
