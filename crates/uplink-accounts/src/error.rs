//! Account layer errors.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("user not found: {0}")]
    UserNotFound(String),
    #[error("wallet not found: {0}")]
    WalletNotFound(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("delegation invalid or expired")]
    DelegationInvalid,
    #[error("storage error: {0}")]
    Storage(String),
}
