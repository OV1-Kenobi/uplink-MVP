//! On-device tag reading + SDM provisioning command construction.
//!
//! Pure APDU/NDEF builders plus an unauthenticated read flow over the [`Transceive`] shim.
//! This module owns the plaintext command/NDEF byte layouts and the tap-read path. The
//! authenticated write sequence (EV2 secure channel: `AuthenticateEV2First` →
//! `ChangeFileSettings`) is completed at the hardware boundary during device bring-up; the
//! [`sdm_file_settings`] payload and [`write_data`] / [`iso_select_file`] builders here are
//! the inputs that sequence encrypts and sends.

use crate::apdu::{exchange, Apdu};
use crate::{Ntag424Error, Transceive};

/// NTAG 424 standard data file holding the NDEF message.
pub const NDEF_FILE_ID: u8 = 0x02;

/// An attendance SDM URL template with the byte offsets of its SDM placeholders.
///
/// Offsets are indices into `template` (the URI body). The tag mirrors 32 ASCII hex
/// chars at `picc_offset` and 16 at `mac_offset`. The MAC input window for this template
/// is empty (PICC-only mirror), matching the AN12196 single-mirror example.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceUri {
    pub template: String,
    pub picc_offset: usize,
    pub mac_offset: usize,
}

/// Build the attendance SDM URI template for an office/tag pair.
///
/// Produces `uplink://attendance?office=<o>&tag=<t>&picc_data=<32x0>&cmac=<16x0>` with the
/// placeholder regions zero-filled (the tag overwrites them with the live mirror).
pub fn attendance_uri(office: &str, tag_id: &str) -> AttendanceUri {
    let head = format!("uplink://attendance?office={office}&tag={tag_id}&picc_data=");
    let picc_offset = head.len();
    let mut template = head;
    template.push_str(&"0".repeat(32));
    let mid = "&cmac=";
    template.push_str(mid);
    let mac_offset = template.len();
    template.push_str(&"0".repeat(16));
    AttendanceUri { template, picc_offset, mac_offset }
}

/// `ISOSelectFile` by 2-byte file id (P1=0x00, P2=0x0C: select, no response data).
pub fn iso_select_file(file_id: u16) -> Apdu {
    Apdu::new(0x00, 0xA4, 0x00, 0x0C)
        .with_data(file_id.to_be_bytes().to_vec())
        .le(0x00)
}

/// NTAG native `ReadData` (INS 0xAD): file no, 3-byte LE offset, 3-byte LE length.
pub fn read_data(file_no: u8, offset: u32, length: u32) -> Apdu {
    let off = offset.to_le_bytes();
    let len = length.to_le_bytes();
    Apdu::native(0xAD, vec![file_no, off[0], off[1], off[2], len[0], len[1], len[2]])
}

/// NTAG native `WriteData` (INS 0x8D): file no, 3-byte LE offset, 3-byte LE length, data.
pub fn write_data(file_no: u8, offset: u32, data: &[u8]) -> Apdu {
    let off = offset.to_le_bytes();
    let len = (data.len() as u32).to_le_bytes();
    let mut buf = vec![file_no, off[0], off[1], off[2], len[0], len[1], len[2]];
    buf.extend_from_slice(data);
    Apdu::native(0x8D, buf)
}

/// Plaintext `ChangeFileSettings` payload enabling SDM with an encrypted PICCData mirror
/// and SDMMAC (AN12196 §3 layout). `picc_off`/`mac_off` are file byte offsets.
///
/// Returned bytes are the command data that the EV2 secure channel encrypts before send.
pub fn sdm_file_settings(picc_off: u32, mac_off: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.push(0x40); // FileOption: CommMode.Plain + SDM enabled (bit6)
    p.extend_from_slice(&[0x00, 0xE0]); // AccessRights: Read=Free, Write/RW/Change=Key0
    // SDMOptions: UID mirror + SDMReadCtr + SDMENCFileData off + bit0 ASCII encoding.
    p.push(0xC1);
    // SDMAccessRights: MetaRead=Key0, FileRead=Key0, CtrRet=Free(0xF).
    p.extend_from_slice(&[0x00, 0xF0]);
    // PICCDataOffset (3-byte LE) and SDMMACOffset (3-byte LE); MAC input == MAC offset.
    p.extend_from_slice(&picc_off.to_le_bytes()[..3]);
    p.extend_from_slice(&mac_off.to_le_bytes()[..3]);
    p.extend_from_slice(&mac_off.to_le_bytes()[..3]);
    p
}

/// Read the NDEF file and extract the stored URI (unauthenticated read).
///
/// Selects the NDEF application, selects the NDEF file, reads `len` bytes, and parses the
/// NDEF URI record. Used for the tap-read path (deep-link into attendance mode).
pub fn read_ndef_url(transport: &mut dyn Transceive, len: u32) -> Result<String, Ntag424Error> {
    exchange(transport, &Apdu::select_ndef_application())?;
    exchange(transport, &iso_select_file(0xE104))?; // NDEF file ISO FID
    let (data, _sw) = exchange(transport, &read_data(NDEF_FILE_ID, 0, len))?;
    parse_ndef_uri(&data)
}

/// Parse a single URI record (NDEF type 'U') out of a standard NDEF file body.
///
/// Layout: `NLEN(2) || 0xD1 0x01 <plen> 'U' <abbrev> <uri-bytes>`.
pub fn parse_ndef_uri(file: &[u8]) -> Result<String, Ntag424Error> {
    let bad = |m: &str| Ntag424Error::InvalidUrl(m.to_string());
    if file.len() < 7 {
        return Err(bad("NDEF file too short"));
    }
    // Skip the 2-byte NLEN, then the record header 0xD1 0x01 <plen> 0x55('U').
    let body = &file[2..];
    if body[0] & 0x07 != 0x01 || body[3] != b'U' {
        return Err(bad("not a URI NDEF record"));
    }
    let plen = body[2] as usize;
    if plen == 0 || 4 + plen > body.len() + 1 {
        return Err(bad("URI payload length out of range"));
    }
    let abbrev = body[4];
    let prefix = uri_abbrev_prefix(abbrev);
    let uri_bytes = &body[5..4 + plen];
    let tail = core::str::from_utf8(uri_bytes).map_err(|_| bad("URI is not UTF-8"))?;
    Ok(format!("{prefix}{tail}"))
}

/// NDEF URI identifier-code prefix table (subset; 0x00 = no prefix).
fn uri_abbrev_prefix(code: u8) -> &'static str {
    match code {
        0x01 => "http://www.",
        0x02 => "https://www.",
        0x03 => "http://",
        0x04 => "https://",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apdu::Transceive;

    #[test]
    fn attendance_uri_offsets_point_at_placeholders() {
        let u = attendance_uri("hq", "tag1");
        assert_eq!(&u.template[u.picc_offset..u.picc_offset + 32], &"0".repeat(32));
        assert_eq!(&u.template[u.mac_offset..u.mac_offset + 16], &"0".repeat(16));
        assert!(u.template.starts_with("uplink://attendance?office=hq&tag=tag1&"));
    }

    #[test]
    fn read_data_apdu_layout() {
        let a = read_data(0x02, 0, 256).to_bytes();
        // 90 AD 00 00 07 [02 000000 000100] 00
        assert_eq!(hex::encode(a), "90ad0000070200000000010000");
    }

    /// Mock card: returns a canned NDEF URI file for any ReadData, OK for selects.
    struct MockCard;
    impl Transceive for MockCard {
        fn transceive(&mut self, apdu: &[u8]) -> Result<Vec<u8>, Ntag424Error> {
            if apdu.get(1) == Some(&0xAD) {
                // NLEN=000F, D1 01 0B 55 04 "uplink.x/" (abbrev 0x04 = https://)
                let uri = b"uplink.x/";
                let mut f = vec![0x00, (4 + uri.len()) as u8, 0xD1, 0x01, (uri.len() + 1) as u8, 0x55, 0x04];
                f.extend_from_slice(uri);
                f.extend_from_slice(&[0x91, 0x00]);
                Ok(f)
            } else {
                Ok(vec![0x90, 0x00])
            }
        }
    }

    #[test]
    fn read_ndef_url_round_trips() {
        let mut card = MockCard;
        let url = read_ndef_url(&mut card, 64).unwrap();
        assert_eq!(url, "https://uplink.x/");
    }
}
