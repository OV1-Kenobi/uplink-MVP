//! Wasm-only localStorage-backed encrypted KvStore.

use crate::crypto::{decrypt, encrypt};
use crate::kv::{KvError, KvStore};
use wasm_bindgen::JsValue;
use web_sys::Window;

pub struct LocalStorageStore {
    storage: web_sys::Storage,
    kek: [u8; 32],
}

impl LocalStorageStore {
    /// Open the local storage and derive the KEK from the passphrase.
    pub fn open(passphrase: &str) -> Result<Self, KvError> {
        let window = web_sys::window().ok_or_else(|| KvError::Backend("no window".into()))?;
        let storage = window
            .local_storage()
            .map_err(|_| KvError::Backend("no local storage".into()))?
            .ok_or_else(|| KvError::Backend("local storage disabled".into()))?;

        // Handle salt (stored unencrypted in localStorage)
        let salt_key = "uplink_storage_salt";
        let salt_hex = storage.get_item(salt_key).map_err(|_| KvError::Backend("get salt failed".into()))?;
        
        let salt = match salt_hex {
            Some(hex_str) => {
                let bytes = hex::decode(hex_str).map_err(|_| KvError::Backend("invalid salt hex".into()))?;
                let mut salt = [0u8; 16];
                salt.copy_from_slice(&bytes);
                salt
            }
            None => {
                let mut salt = [0u8; 16];
                crate::crypto::rand_bytes(&mut salt);
                storage
                    .set_item(salt_key, &hex::encode(salt))
                    .map_err(|_| KvError::Backend("set salt failed".into()))?;
                salt
            }
        };

        let kek = crate::crypto::derive_kek(passphrase, &salt);
        Ok(Self { storage, kek })
    }
}

#[async_trait::async_trait]
impl KvStore for LocalStorageStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KvError> {
        match self.storage.get_item(key).map_err(|_| KvError::Backend("get failed".into()))? {
            Some(hex_str) => {
                let blob = hex::decode(hex_str).map_err(|_| KvError::Backend("invalid hex".into()))?;
                Ok(Some(decrypt(&self.kek, &blob)?))
            }
            None => Ok(None),
        }
    }

    async fn put(&self, key: &str, value: &[u8]) -> Result<(), KvError> {
        let blob = encrypt(&self.kek, value)?;
        self.storage
            .set_item(key, &hex::encode(blob))
            .map_err(|_| KvError::Backend("set failed".into()))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), KvError> {
        self.storage
            .remove_item(key)
            .map_err(|_| KvError::Backend("remove failed".into()))?;
        Ok(())
    }
}
