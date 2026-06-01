//! `UplinkIdentity` — the root identity object for every Uplink user/agent.

use bip39::Mnemonic;
use nostr::{Keys, ToBech32};
use serde::{Deserialize, Serialize};

use crate::derivation::{derive_ldk_seed, nip06_path};
use crate::IdentityError;

/// A unified identity derived from a single BIP-39 mnemonic.
///
/// ## Custody rule (ADR-U-001)
/// The raw mnemonic and all derived secret material MUST remain inside Rust.
/// The wasm-bindgen surface in `uplink-core` never exposes secret bytes to JS.
#[derive(Clone)]
pub struct UplinkIdentity {
    mnemonic: Mnemonic,
    account: u32,
    /// NIP-06 keypair — Nostr identity.
    pub nostr_keys: Keys,
    /// 32-byte seed for LDK `KeysManager` at this account index.
    pub ldk_node_seed: [u8; 32],
}

impl UplinkIdentity {
    /// Generate a fresh random identity.
    pub fn generate(account: u32) -> Result<Self, IdentityError> {
        use rand::RngCore;
        let mut entropy = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| IdentityError::InvalidMnemonic(e.to_string()))?;
        Self::from_mnemonic(mnemonic, account)
    }

    /// Restore an identity from an existing BIP-39 mnemonic string.
    pub fn from_mnemonic_str(phrase: &str, account: u32) -> Result<Self, IdentityError> {
        let mnemonic: Mnemonic = phrase
            .parse()
            .map_err(|e: bip39::Error| IdentityError::InvalidMnemonic(e.to_string()))?;
        Self::from_mnemonic(mnemonic, account)
    }

    fn from_mnemonic(mnemonic: Mnemonic, account: u32) -> Result<Self, IdentityError> {
        // NIP-06: derive Nostr keypair from BIP-32
        let seed_bytes = mnemonic.to_seed("");
        let root = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed_bytes)
            .map_err(|e| IdentityError::Derivation(e.to_string()))?;
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let path = nip06_path(account);
        let child = root
            .derive_priv(&secp, &path)
            .map_err(|e| IdentityError::Derivation(e.to_string()))?;
        let nostr_keys = Keys::new(
            nostr::SecretKey::from_slice(&child.private_key.secret_bytes())
                .map_err(|e| IdentityError::Derivation(e.to_string()))?,
        );

        // LDK seed
        let ldk_node_seed = derive_ldk_seed(&mnemonic, account);

        Ok(Self { mnemonic, account, nostr_keys, ldk_node_seed })
    }

    /// The Nostr public key (npub) as a bech32 string.
    pub fn npub(&self) -> String {
        self.nostr_keys.public_key().to_bech32().unwrap_or_default()
    }

    /// The account index this identity was derived at.
    pub fn account_index(&self) -> u32 {
        self.account
    }

    /// The raw mnemonic phrase — **never** cross the wasm boundary.
    ///
    /// Callers must ensure this value is not logged or serialized across trust boundaries.
    pub fn mnemonic_phrase(&self) -> &str {
        self.mnemonic.to_string().leak() // intentional: bip39 Mnemonic doesn't expose &str
    }

    /// Export the mnemonic as a word array for backup display.
    pub fn mnemonic_words(&self) -> Vec<String> {
        self.mnemonic.words().map(str::to_string).collect()
    }
}

/// Safe debug: never prints secret material.
impl std::fmt::Debug for UplinkIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UplinkIdentity")
            .field("npub", &self.npub())
            .field("account", &self.account)
            .finish()
    }
}

/// Safe serialization: only public material.
#[derive(Debug, Serialize, Deserialize)]
pub struct UplinkIdentityPublic {
    pub npub: String,
    pub account: u32,
}

impl From<&UplinkIdentity> for UplinkIdentityPublic {
    fn from(id: &UplinkIdentity) -> Self {
        Self { npub: id.npub(), account: id.account }
    }
}
