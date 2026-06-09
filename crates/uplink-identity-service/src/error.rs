//! Stable, safe error codes for the hosted identity service (Phase 5b).
//!
//! Each variant maps to a stable string `code()` returned to clients; internal detail is
//! never embedded in the client-facing code (no secret leakage — ADR-U-010 custody rule).

use thiserror::Error;

/// Errors surfaced by the identity service core.
#[derive(Debug, Error)]
pub enum IdentityServiceError {
    /// Username failed normalization/validation rules.
    #[error("username_invalid")]
    UsernameInvalid,
    /// Username already registered to a live (non-revoked) identity.
    #[error("username_taken")]
    UsernameTaken,
    /// Public key is not 64 lowercase-hex characters.
    #[error("pubkey_invalid")]
    PubkeyInvalid,
    /// No identity matches the request.
    #[error("not_found")]
    NotFound,
    /// Requested amount is outside the payable range.
    #[error("amount_out_of_range")]
    AmountOutOfRange,
    /// Identity exists but is revoked.
    #[error("revoked")]
    Revoked,
    /// Backend/storage failure; detail kept server-side only, never returned to clients.
    #[error("backend_error")]
    Backend(String),
}

impl IdentityServiceError {
    /// Stable, safe error code for client responses.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UsernameInvalid => "username_invalid",
            Self::UsernameTaken => "username_taken",
            Self::PubkeyInvalid => "pubkey_invalid",
            Self::NotFound => "not_found",
            Self::AmountOutOfRange => "amount_out_of_range",
            Self::Revoked => "revoked",
            Self::Backend(_) => "backend_error",
        }
    }
}
