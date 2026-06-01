//! BIP-32 derivation helpers.
//!
//! All derivation paths are specified in ADR-U-001.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::Network;
use bip39::Mnemonic;
use sha2::{Digest, Sha512};

/// NIP-06 path: m/44'/1237'/account'/0/0
pub fn nip06_path(account: u32) -> DerivationPath {
    format!("m/44'/1237'/{}'/0/0", account)
        .parse()
        .expect("static NIP-06 path is valid")
}

/// BIP-44 on-chain path: m/44'/0'/account'/0/0
pub fn bip44_onchain_path(account: u32) -> DerivationPath {
    format!("m/44'/0'/{}'/0/0", account)
        .parse()
        .expect("static BIP-44 path is valid")
}

/// LDK KeysManager seed path: m/535348'/0'/account'
/// 535348 = ASCII "SSH" — chosen per ADR-U-001 (ecosystem alignment with sample wallets).
pub fn ldk_seed_path(account: u32) -> DerivationPath {
    format!("m/535348'/0'/{}'", account)
        .parse()
        .expect("static LDK seed path is valid")
}

/// Derive the 32-byte LDK `KeysManager` seed for the given account index.
pub fn derive_ldk_seed(mnemonic: &Mnemonic, account: u32) -> [u8; 32] {
    let seed_bytes = mnemonic.to_seed("");
    let root = Xpriv::new_master(Network::Bitcoin, &seed_bytes)
        .expect("mnemonic seed yields valid root xpriv");
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let path = ldk_seed_path(account);
    let child = root
        .derive_priv(&secp, &path)
        .expect("LDK derivation path is hardened and valid");

    // Hash the derived secret key bytes into 32 bytes for KeysManager.
    let mut hasher = Sha512::new();
    hasher.update(child.private_key.secret_bytes());
    let hash = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash[..32]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ldk_seed_is_32_bytes_and_deterministic() {
        let m: Mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
            .parse()
            .unwrap();
        let seed_a = derive_ldk_seed(&m, 0);
        let seed_b = derive_ldk_seed(&m, 0);
        assert_eq!(seed_a, seed_b);
        assert_ne!(seed_a, [0u8; 32]);
    }

    #[test]
    fn different_accounts_yield_different_ldk_seeds() {
        let m: Mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
            .parse()
            .unwrap();
        let seed_0 = derive_ldk_seed(&m, 0);
        let seed_1 = derive_ldk_seed(&m, 1);
        assert_ne!(seed_0, seed_1);
    }
}
