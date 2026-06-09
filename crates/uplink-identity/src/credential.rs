//! External credentials brought by the user (ADR-U-010, Phase 5a).
//!
//! Capability-typed wrapper over the credentials a user can link during onboarding:
//! a Lightning Address (receive), an NWC connection string (receive + spend), a Lightning
//! Node Connect pairing phrase (spend; LND-direct, transport gated), or a Nostr identity
//! (npub / NIP-05).
//!
//! ## Custody (ADR-U-006, ADR-U-010)
//! NWC URIs and LNC pairing phrases are bearer secrets. They are encrypted at rest by
//! `uplink-storage` and MUST NOT cross to the UI. The UI receives only [`CredentialMeta`]
//! — a redacted, non-secret descriptor produced by [`ExternalCredential::meta`].

use nostr::PublicKey;
use serde::{Deserialize, Serialize};

use crate::IdentityError;

/// The kind of external credential a user has linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
    LightningAddress,
    Nip05,
    Npub,
    Nwc,
    Lnc,
}

impl CredentialKind {
    /// Whether this credential can receive funds.
    #[must_use]
    pub fn receive_capable(self) -> bool {
        matches!(self, Self::LightningAddress | Self::Nwc | Self::Lnc)
    }

    /// Whether this credential authorizes spending (NWC + LNC — ADR-U-010 §2).
    ///
    /// NWC carries a real NIP-47 `pay_invoice` rail, so it is the MVP spend path
    /// (AlbyHub / Breez / LND-via-Alby). LNC is spend-capable by design but its transport
    /// is gated (see `uplink-wallet::lnc`), so it cannot execute spends yet.
    #[must_use]
    pub fn spend_capable(self) -> bool {
        matches!(self, Self::Nwc | Self::Lnc)
    }

    /// Whether the credential payload is a bearer secret (encrypted, UI-hidden).
    #[must_use]
    pub fn is_secret(self) -> bool {
        matches!(self, Self::Nwc | Self::Lnc)
    }
}

/// A linked external credential, including any bearer secret.
///
/// **Never** serialize this to the UI — call [`ExternalCredential::meta`] for the redacted,
/// non-secret descriptor instead. `Debug` is implemented to redact secret payloads.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ExternalCredential {
    /// `you@wallet.com` — pure receive routing, no secret.
    LightningAddress(String),
    /// NIP-05 identifier `name@domain` — identity only.
    Nip05(String),
    /// Bech32 `npub1…` — identity only.
    Npub(String),
    /// NWC connection string `nostr+walletconnect://…` — receive + spend secret.
    Nwc(String),
    /// Lightning Node Connect 10-word pairing phrase — spend secret (LND-direct, gated).
    Lnc(String),
}

impl ExternalCredential {
    /// Parse + validate a Lightning Address (`user@domain`).
    pub fn lightning_address(addr: &str) -> Result<Self, IdentityError> {
        let s = addr.trim();
        match s.split_once('@') {
            Some((u, d)) if !u.is_empty() && d.contains('.') => Ok(Self::LightningAddress(s.into())),
            _ => Err(IdentityError::InvalidCredential(format!("not a Lightning address: {s}"))),
        }
    }

    /// Parse + validate a NIP-05 identifier (`name@domain`).
    pub fn nip05(id: &str) -> Result<Self, IdentityError> {
        let s = id.trim();
        match s.split_once('@') {
            Some((u, d)) if !u.is_empty() && d.contains('.') => Ok(Self::Nip05(s.into())),
            _ => Err(IdentityError::InvalidCredential(format!("not a NIP-05 id: {s}"))),
        }
    }

    /// Parse + validate a bech32 `npub1…`.
    pub fn npub(npub: &str) -> Result<Self, IdentityError> {
        let s = npub.trim();
        PublicKey::parse(s).map_err(|e| IdentityError::InvalidCredential(e.to_string()))?;
        Ok(Self::Npub(s.into()))
    }

    /// Parse + validate an NWC connection string.
    pub fn nwc(uri: &str) -> Result<Self, IdentityError> {
        let s = uri.trim();
        if s.starts_with("nostr+walletconnect://") {
            Ok(Self::Nwc(s.into()))
        } else {
            Err(IdentityError::InvalidCredential("not an NWC connection string".into()))
        }
    }

    /// Parse + validate a Lightning Node Connect 10-word pairing phrase.
    pub fn lnc(pairing_phrase: &str) -> Result<Self, IdentityError> {
        let words = pairing_phrase.split_whitespace().count();
        if words == 10 {
            Ok(Self::Lnc(pairing_phrase.split_whitespace().collect::<Vec<_>>().join(" ")))
        } else {
            Err(IdentityError::InvalidCredential(format!(
                "LNC pairing phrase must be 10 words, got {words}"
            )))
        }
    }

    /// The kind of this credential.
    #[must_use]
    pub fn kind(&self) -> CredentialKind {
        match self {
            Self::LightningAddress(_) => CredentialKind::LightningAddress,
            Self::Nip05(_) => CredentialKind::Nip05,
            Self::Npub(_) => CredentialKind::Npub,
            Self::Nwc(_) => CredentialKind::Nwc,
            Self::Lnc(_) => CredentialKind::Lnc,
        }
    }

    /// A redacted, human-readable label safe to show in the UI. Secret payloads are
    /// never included verbatim.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::LightningAddress(a) => a.clone(),
            Self::Nip05(a) => a.clone(),
            Self::Npub(p) => redact_mid(p),
            Self::Nwc(_) => "NWC wallet".into(),
            Self::Lnc(_) => "LND · Lightning Node Connect".into(),
        }
    }

    /// Build the redacted, non-secret descriptor returned to the UI.
    #[must_use]
    pub fn meta(&self, added_at_unix: u64) -> CredentialMeta {
        let kind = self.kind();
        CredentialMeta {
            kind,
            label: self.label(),
            receive_capable: kind.receive_capable(),
            spend_capable: kind.spend_capable(),
            added_at_unix,
        }
    }
}

/// Redact the middle of a long identifier (e.g. an npub) for display.
fn redact_mid(s: &str) -> String {
    if s.len() <= 20 {
        s.to_string()
    } else {
        format!("{}…{}", &s[..12], &s[s.len() - 6..])
    }
}

/// Non-secret credential descriptor returned to the UI (ADR-U-010 custody invariant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMeta {
    pub kind: CredentialKind,
    pub label: String,
    pub receive_capable: bool,
    pub spend_capable: bool,
    pub added_at_unix: u64,
}

/// Redacting `Debug`: secret payloads are never printed.
impl std::fmt::Debug for ExternalCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.kind().is_secret() {
            write!(f, "ExternalCredential::{:?}(<redacted>)", self.kind())
        } else {
            write!(f, "ExternalCredential::{:?}({})", self.kind(), self.label())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NWC: &str = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss://relay.example.com&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";

    #[test]
    fn capability_split_matches_adr_u_010() {
        assert!(CredentialKind::LightningAddress.receive_capable());
        assert!(!CredentialKind::LightningAddress.spend_capable());
        assert!(CredentialKind::Nwc.receive_capable());
        assert!(CredentialKind::Nwc.spend_capable(), "NWC is the MVP spend rail");
        assert!(CredentialKind::Lnc.spend_capable(), "LNC is spend-capable (gated)");
        assert!(!CredentialKind::Npub.receive_capable());
        assert!(CredentialKind::Nwc.is_secret() && CredentialKind::Lnc.is_secret());
        assert!(!CredentialKind::LightningAddress.is_secret());
    }

    #[test]
    fn parsers_validate_and_reject() {
        assert!(ExternalCredential::lightning_address("alice@wallet.com").is_ok());
        assert!(ExternalCredential::lightning_address("nope").is_err());
        assert!(ExternalCredential::nip05("alice@domain.io").is_ok());
        assert!(ExternalCredential::nwc(NWC).is_ok());
        assert!(ExternalCredential::nwc("https://example.com").is_err());
        assert!(ExternalCredential::lnc("one two three four five six seven eight nine ten").is_ok());
        assert!(ExternalCredential::lnc("too few words").is_err());
        use nostr::ToBech32;
        let npub = nostr::Keys::generate().public_key().to_bech32().unwrap();
        assert!(ExternalCredential::npub(&npub).is_ok());
        assert!(ExternalCredential::npub("npub1garbage").is_err());
    }

    #[test]
    fn meta_redacts_secrets_and_carries_capabilities() {
        let nwc = ExternalCredential::nwc(NWC).unwrap();
        let meta = nwc.meta(1_700_000_000);
        assert_eq!(meta.kind, CredentialKind::Nwc);
        assert!(meta.receive_capable && meta.spend_capable);
        assert!(!meta.label.contains("secret="), "NWC secret must never appear in the label");
        assert_eq!(meta.label, "NWC wallet");

        let lnc = ExternalCredential::lnc("one two three four five six seven eight nine ten").unwrap();
        assert!(lnc.meta(0).spend_capable);
    }

    #[test]
    fn debug_never_leaks_secret_payloads() {
        let nwc = ExternalCredential::nwc(NWC).unwrap();
        let dbg = format!("{nwc:?}");
        assert!(dbg.contains("<redacted>"));
        assert!(!dbg.contains("secret="));
        // Non-secret credential shows its (public) label.
        let la = ExternalCredential::lightning_address("alice@wallet.com").unwrap();
        assert!(format!("{la:?}").contains("alice@wallet.com"));
    }

    #[test]
    fn secret_value_survives_serde_roundtrip_for_at_rest_storage() {
        // The full enum (with secret) is what we encrypt at rest — round-trip must be lossless.
        let nwc = ExternalCredential::nwc(NWC).unwrap();
        let json = serde_json::to_string(&nwc).unwrap();
        let back: ExternalCredential = serde_json::from_str(&json).unwrap();
        match back {
            ExternalCredential::Nwc(uri) => assert_eq!(uri, NWC),
            other => panic!("wrong variant: {other:?}"),
        }
    }
}
