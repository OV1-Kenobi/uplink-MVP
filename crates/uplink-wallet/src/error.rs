//! Wallet error types.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("LDK error: {0}")]
    Ldk(String),
    #[error("insufficient balance: need {needed_msats} msats, have {available_msats}")]
    InsufficientBalance { needed_msats: u64, available_msats: u64 },
    #[error("invoice decode error: {0}")]
    InvoiceDecode(String),
    #[error("payment failed: {0}")]
    PaymentFailed(String),
    #[error("LSP error: {0}")]
    Lsp(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("storage error: {0}")]
    Storage(String),
}
