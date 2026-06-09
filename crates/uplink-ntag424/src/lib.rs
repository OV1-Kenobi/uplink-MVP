//! # uplink-ntag424
//!
//! Pure-Rust NTAG 424 DNA (Secure Dynamic Messaging / SUN) client for Uplink.
//!
//! Reads *and* programs NTAG 424 DNA tags with no companion app and no Java/native
//! crypto, so the same code runs on Android (Tauri/JNI `IsoDep`), desktop (PC/SC), and in
//! unit tests. All tag I/O goes through the [`apdu::Transceive`] shim; the crypto is
//! implemented from `aes` + `cmac` primitives (NXP AN12196 / NT4H2421Gx / RFC 4493).
//!
//! ADR: docs/adr/ADR-U-009-ntag424-sdm-provisioning-verification.md

#![forbid(unsafe_code)]

pub mod crypto;
pub mod apdu;
pub mod sdm;
pub mod provision;
#[cfg(feature = "session")]
pub mod session_link;

pub use apdu::{Transceive, Apdu, StatusWord};
pub use crypto::PiccData;
pub use sdm::{SdmUrl, SdmVerification, SdmVerifier};

/// Errors produced by the NTAG 424 client.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum Ntag424Error {
    /// The underlying transport (`IsoDep` / PC/SC / mock) failed.
    #[error("transceive failed: {0}")]
    Transport(String),
    /// The card returned a non-success ISO 7816 status word.
    #[error("card error: status word {0:#06x}")]
    StatusWord(u16),
    /// The card response was shorter than the protocol requires.
    #[error("response too short: need {needed} bytes, have {have}")]
    ShortResponse { needed: usize, have: usize },
    /// The SDMMAC in the read NDEF/URL did not match the recomputed value.
    #[error("SDMMAC verification failed")]
    MacMismatch,
    /// The decrypted PICCData tag byte is malformed (possible counterfeit/corruption).
    #[error("invalid PICCData tag byte: {0:#04x}")]
    InvalidPiccDataTag(u8),
    /// A malformed SDM URL or query parameter.
    #[error("invalid SDM URL: {0}")]
    InvalidUrl(String),
    /// A feature that requires a live secure channel against hardware not available here.
    #[error("not supported in this build: {0}")]
    NotSupported(&'static str),
}
