//! AES-256-GCM encryption helpers used by all KvStore adapters.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use sha2::Sha256;

use crate::kv::KvError;

/// Derive a 32-byte AES key from a passphrase using Argon2id.
pub fn derive_kek(passphrase: &str, salt: &[u8; 16]) -> [u8; 32] {
    use argon2::{Argon2, Params};

    let params = Params::new(16384, 2, 1, Some(32)).expect("static Argon2 params are valid");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .expect("Argon2 hashing is infallible for these params");
    key
}

/// Generate random bytes using OsRng.
pub fn rand_bytes(out: &mut [u8]) {
    OsRng.fill_bytes(out);
}

/// Encrypt `plaintext` with AES-256-GCM using the given 32-byte key.
/// Returns `nonce (12 bytes) || ciphertext`.
pub fn encrypt(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, KvError> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|_| KvError::Crypto)?;
    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a `nonce || ciphertext` blob produced by `encrypt`.
pub fn decrypt(key_bytes: &[u8; 32], blob: &[u8]) -> Result<Vec<u8>, KvError> {
    if blob.len() < 12 {
        return Err(KvError::Crypto);
    }
    let (nonce_bytes, ciphertext) = blob.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).map_err(|_| KvError::Crypto)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let key = [0u8; 32];
        let plaintext = b"hello uplink";
        let blob = encrypt(&key, plaintext).unwrap();
        let recovered = decrypt(&key, &blob).unwrap();
        assert_eq!(recovered, plaintext);
    }
}
