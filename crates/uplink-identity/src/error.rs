//! Identity derivation errors.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("key derivation failed: {0}")]
    Derivation(String),

    #[error("storage error: {0}")]
    Storage(String),
}
