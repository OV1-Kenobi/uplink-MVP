//! SDM (SUN) URL model + local verifier (NXP AN12196, AES mode).
//!
//! Verifies the SDMMAC mirrored in an NTAG 424 NDEF URL and recovers the tag UID and
//! read counter. Server-side re-verification is Phase 6 (ADR-U-011); this is the on-device
//! local check that gates tap-to-sign-in/out.

use crate::crypto::{decrypt_picc_data, sdm_mac};
use crate::Ntag424Error;

/// Parsed SDM URL query parameters.
///
/// Accepts both the verbose (`picc_data`, `enc`, `cmac`) and short (`p`, `e`, `m`)
/// parameter spellings used by common NTAG 424 templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdmUrl {
    /// Encrypted PICCData, 16 bytes (32 ASCII hex chars).
    pub enc_picc: [u8; 16],
    /// Optional encrypted file-data ASCII bytes (the SDMMAC input window).
    pub enc_file_ascii: Vec<u8>,
    /// Truncated SDMMAC, 8 bytes (16 ASCII hex chars).
    pub cmac: [u8; 8],
}

fn query_param<'a>(query: &'a str, names: &[&str]) -> Option<&'a str> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if names.contains(&k) {
                return Some(v);
            }
        }
    }
    None
}

impl SdmUrl {
    /// Parse the SDM parameters out of a full `uplink://…` or `https://…` URL.
    pub fn parse(url: &str) -> Result<Self, Ntag424Error> {
        let query = url
            .split_once('?')
            .map(|(_, q)| q)
            .ok_or_else(|| Ntag424Error::InvalidUrl("no query string".into()))?;
        // Strip any fragment.
        let query = query.split('#').next().unwrap_or(query);

        let picc_hex = query_param(query, &["picc_data", "p"])
            .ok_or_else(|| Ntag424Error::InvalidUrl("missing picc_data".into()))?;
        let cmac_hex = query_param(query, &["cmac", "m"])
            .ok_or_else(|| Ntag424Error::InvalidUrl("missing cmac".into()))?;

        let enc_picc = decode_hex_array::<16>(picc_hex)?;
        let cmac = decode_hex_array::<8>(cmac_hex)?;
        let enc_file_ascii = query_param(query, &["enc", "e"])
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_default();

        Ok(Self { enc_picc, enc_file_ascii, cmac })
    }
}

/// Successfully verified SDM data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdmVerification {
    pub uid: [u8; 7],
    pub read_ctr: u32,
}

/// Local SDM verifier holding the application keys for one tag population.
#[derive(Clone)]
pub struct SdmVerifier {
    meta_read_key: [u8; 16],
    file_read_key: [u8; 16],
}

impl SdmVerifier {
    /// New verifier with separate `SDMMetaRead` (PICC decrypt) and `SDMFileRead`
    /// (session/MAC) keys.
    pub fn new(meta_read_key: [u8; 16], file_read_key: [u8; 16]) -> Self {
        Self { meta_read_key, file_read_key }
    }

    /// Convenience: both keys equal (the common single-key configuration).
    pub fn single_key(key: [u8; 16]) -> Self {
        Self::new(key, key)
    }

    /// Verify a parsed SDM URL: decrypt PICCData, recompute the SDMMAC over the MAC input
    /// window, and constant-time-compare it to the mirrored CMAC.
    pub fn verify(&self, url: &SdmUrl) -> Result<SdmVerification, Ntag424Error> {
        let picc = decrypt_picc_data(&self.meta_read_key, &url.enc_picc)?;
        let expected = sdm_mac(&self.file_read_key, &picc.uid, &picc.read_ctr, &url.enc_file_ascii);
        if !ct_eq(&expected, &url.cmac) {
            return Err(Ntag424Error::MacMismatch);
        }
        Ok(SdmVerification { uid: picc.uid, read_ctr: picc.read_ctr_u32() })
    }
}

/// Constant-time 8-byte comparison.
fn ct_eq(a: &[u8; 8], b: &[u8; 8]) -> bool {
    let mut diff = 0u8;
    for i in 0..8 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

fn decode_hex_array<const N: usize>(s: &str) -> Result<[u8; N], Ntag424Error> {
    let bytes = hex::decode(s).map_err(|e| Ntag424Error::InvalidUrl(e.to_string()))?;
    if bytes.len() != N {
        return Err(Ntag424Error::InvalidUrl(format!(
            "expected {N} bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; N];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // NXP AN12196 public worked example, factory (all-zero) keys. This is the Phase-4
    // acceptance criterion: local verify of a real SDM tag's mirrored URL must succeed.
    const AN12196_URL: &str =
        "https://uplink.example/?picc_data=EF963FF7828658A599F3041510671E88&cmac=94EED9EE65337086";

    #[test]
    fn an12196_local_verify_succeeds() {
        let url = SdmUrl::parse(AN12196_URL).unwrap();
        let verifier = SdmVerifier::single_key([0u8; 16]);
        let v = verifier.verify(&url).expect("AN12196 SDMMAC must verify");
        // Recovered identity is non-trivial and the counter decodes.
        assert_ne!(v.uid, [0u8; 7]);
        let _ = v.read_ctr;
    }

    #[test]
    fn wrong_file_key_fails_mac() {
        let url = SdmUrl::parse(AN12196_URL).unwrap();
        // Correct meta key (PICCData decrypts cleanly) but wrong file-read key → the
        // recomputed SDMMAC differs from the tag's, so verification rejects.
        let bad = SdmVerifier::new([0u8; 16], [0x11u8; 16]);
        assert_eq!(bad.verify(&url), Err(Ntag424Error::MacMismatch));
    }

    #[test]
    fn parse_rejects_missing_params() {
        assert!(SdmUrl::parse("https://x/").is_err());
        assert!(SdmUrl::parse("https://x/?picc_data=00").is_err());
    }
}
