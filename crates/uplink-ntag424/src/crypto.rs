//! AES-CMAC + NTAG 424 DNA SDM (AES) crypto primitives (NXP AN12196, RFC 4493).
//!
//! Pure functions only — no I/O, no platform crypto. AES mode for the MVP (LRP deferred).

use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use cmac::{Cmac, Mac};

use crate::Ntag424Error;

/// Full 16-byte AES-CMAC over `data` under `key` (RFC 4493).
pub fn aes_cmac(key: &[u8; 16], data: &[u8]) -> [u8; 16] {
    let mut mac = <Cmac<Aes128> as Mac>::new_from_slice(key).expect("AES-128 key is 16 bytes");
    mac.update(data);
    let out = mac.finalize().into_bytes();
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&out);
    buf
}

/// NXP SDMMAC truncation: keep the 8 odd-indexed bytes (1,3,5,…,15) of a full CMAC.
pub fn sdm_truncate(full: &[u8; 16]) -> [u8; 8] {
    let mut out = [0u8; 8];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = full[2 * i + 1];
    }
    out
}

/// Derive `SesSDMFileReadMACKey` from `SDMFileReadKey`, `uid`, and `read_ctr` (AN12196).
///
/// `SV2 = 3Ch C3h 00h 01h 00h 80h || UID || SDMReadCtr(LSB-first)`, then
/// `SesSDMFileReadMACKey = CMAC(KSDMFileRead, SV2)`.
pub fn derive_sdm_session_mac_key(
    file_read_key: &[u8; 16],
    uid: &[u8; 7],
    read_ctr: &[u8; 3],
) -> [u8; 16] {
    let mut sv2 = Vec::with_capacity(16);
    sv2.extend_from_slice(&[0x3C, 0xC3, 0x00, 0x01, 0x00, 0x80]);
    sv2.extend_from_slice(uid);
    sv2.extend_from_slice(read_ctr);
    aes_cmac(file_read_key, &sv2)
}

/// Compute the truncated SDMMAC over `mac_input` for the given tag identity.
pub fn sdm_mac(
    file_read_key: &[u8; 16],
    uid: &[u8; 7],
    read_ctr: &[u8; 3],
    mac_input: &[u8],
) -> [u8; 8] {
    let session_key = derive_sdm_session_mac_key(file_read_key, uid, read_ctr);
    sdm_truncate(&aes_cmac(&session_key, mac_input))
}

/// Recovered tag identity from an encrypted PICCData blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiccData {
    pub tag: u8,
    pub uid: [u8; 7],
    pub read_ctr: [u8; 3],
}

impl PiccData {
    /// Read counter as a host integer (the on-tag encoding is LSB-first).
    pub fn read_ctr_u32(&self) -> u32 {
        u32::from_le_bytes([self.read_ctr[0], self.read_ctr[1], self.read_ctr[2], 0])
    }
}

/// Single-block AES-128 ECB encryption (used as the CBC primitive with IV=0).
fn aes_encrypt_block(key: &[u8; 16], block: &mut [u8; 16]) {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    cipher.encrypt_block(GenericArray::from_mut_slice(block));
}

/// AES-128-CBC decrypt of a single 16-byte block with IV = 0 (sufficient for the
/// 16-byte AES PICCData blob). Implemented via ECB to avoid an extra dependency.
fn aes_cbc_decrypt_block_iv0(key: &[u8; 16], block: &[u8; 16]) -> [u8; 16] {
    use aes::cipher::BlockDecrypt;
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut buf = *block;
    cipher.decrypt_block(GenericArray::from_mut_slice(&mut buf));
    // IV = 0 → no XOR needed for the first (and only) block.
    buf
}

/// Decrypt and parse a 16-byte encrypted PICCData blob (AES) under `meta_read_key`.
///
/// Layout: `PICCDataTag(1) || UID(7) || SDMReadCtr(3, LSB-first) || padding(5)`.
pub fn decrypt_picc_data(
    meta_read_key: &[u8; 16],
    enc_picc: &[u8; 16],
) -> Result<PiccData, Ntag424Error> {
    let plain = aes_cbc_decrypt_block_iv0(meta_read_key, enc_picc);
    let tag = plain[0];
    // Bit 7 (0x80) of the tag byte indicates UID is mirrored; low nibble is UID length.
    if tag & 0x80 == 0 || (tag & 0x0F) != 7 {
        return Err(Ntag424Error::InvalidPiccDataTag(tag));
    }
    let mut uid = [0u8; 7];
    uid.copy_from_slice(&plain[1..8]);
    let mut read_ctr = [0u8; 3];
    read_ctr.copy_from_slice(&plain[8..11]);
    Ok(PiccData { tag, uid, read_ctr })
}

/// Keep `aes_encrypt_block` available for IV/keystream derivations (enc file data, Phase 6).
#[allow(dead_code)]
pub(crate) fn ecb_encrypt(key: &[u8; 16], block: [u8; 16]) -> [u8; 16] {
    let mut b = block;
    aes_encrypt_block(key, &mut b);
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4493 AES-CMAC known-answer vectors (key 2b7e1516…).
    const K: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
        0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    #[test]
    fn rfc4493_empty_message() {
        assert_eq!(
            hex::encode(aes_cmac(&K, &[])),
            "bb1d6929e95937287fa37d129b756746"
        );
    }

    #[test]
    fn rfc4493_16_byte_message() {
        let m = hex::decode("6bc1bee22e409f96e93d7e117393172a").unwrap();
        assert_eq!(
            hex::encode(aes_cmac(&K, &m)),
            "070a16b46b4d4144f79bdd9dd04a287c"
        );
    }
}
