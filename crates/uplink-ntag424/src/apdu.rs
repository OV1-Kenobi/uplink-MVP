//! ISO 7816-4 APDU construction + the `Transceive` transport shim.
//!
//! NTAG 424 DNA native commands are carried in ISO "wrapped" APDUs: `CLA=0x90`,
//! `INS=<cmd>`, `P1=P2=0x00`, short `Lc || data`, and `Le=0x00`. The card replies with
//! `data || SW1 SW2`, where `SW=0x9100` means OK and `SW=0x91AF` means "additional frame".

use crate::Ntag424Error;

/// The single hardware boundary: exchange one command APDU for one response APDU.
///
/// Implemented by the Android `IsoDep` (Tauri/JNI) binding, a desktop PC/SC binding, and
/// the test `MockTransport`. The crate performs no other I/O.
pub trait Transceive {
    fn transceive(&mut self, apdu: &[u8]) -> Result<Vec<u8>, Ntag424Error>;
}

/// ISO 7816 status word (`SW1 SW2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusWord(pub u16);

impl StatusWord {
    /// ISO success.
    pub const ISO_OK: u16 = 0x9000;
    /// NTAG native "operation OK".
    pub const NATIVE_OK: u16 = 0x9100;
    /// NTAG native "additional frame expected".
    pub const ADDITIONAL_FRAME: u16 = 0x91AF;

    pub fn is_ok(self) -> bool {
        matches!(self.0, Self::ISO_OK | Self::NATIVE_OK)
    }

    pub fn is_additional_frame(self) -> bool {
        self.0 == Self::ADDITIONAL_FRAME
    }
}

/// A command APDU.
#[derive(Debug, Clone)]
pub struct Apdu {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
    pub le: Option<u8>,
}

impl Apdu {
    pub fn new(cla: u8, ins: u8, p1: u8, p2: u8) -> Self {
        Self { cla, ins, p1, p2, data: Vec::new(), le: None }
    }

    pub fn with_data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.data = data.into();
        self
    }

    pub fn le(mut self, le: u8) -> Self {
        self.le = Some(le);
        self
    }

    /// Serialize to the short-APDU byte form.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = vec![self.cla, self.ins, self.p1, self.p2];
        if !self.data.is_empty() {
            out.push(self.data.len() as u8); // Lc (short form; native cmds are < 256 bytes)
            out.extend_from_slice(&self.data);
        }
        if let Some(le) = self.le {
            out.push(le);
        }
        out
    }

    /// Build an NTAG native command wrapped in an ISO APDU (`CLA=0x90`, `Le=0x00`).
    pub fn native(ins: u8, data: impl Into<Vec<u8>>) -> Self {
        Apdu::new(0x90, ins, 0x00, 0x00).with_data(data).le(0x00)
    }

    /// `ISOSelectFile` of the NTAG 424 NDEF application by DF name `D2760000850101`.
    pub fn select_ndef_application() -> Self {
        Apdu::new(0x00, 0xA4, 0x04, 0x00)
            .with_data([0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01])
            .le(0x00)
    }
}

/// Split a response APDU into `(data, status_word)`.
pub fn split_response(resp: &[u8]) -> Result<(&[u8], StatusWord), Ntag424Error> {
    if resp.len() < 2 {
        return Err(Ntag424Error::ShortResponse { needed: 2, have: resp.len() });
    }
    let split = resp.len() - 2;
    let sw = u16::from_be_bytes([resp[split], resp[split + 1]]);
    Ok((&resp[..split], StatusWord(sw)))
}

/// Transceive `apdu`, returning the response data and requiring a success/AF status word.
pub fn exchange(
    transport: &mut dyn Transceive,
    apdu: &Apdu,
) -> Result<(Vec<u8>, StatusWord), Ntag424Error> {
    let raw = transport.transceive(&apdu.to_bytes())?;
    let (data, sw) = split_response(&raw)?;
    if !sw.is_ok() && !sw.is_additional_frame() {
        return Err(Ntag424Error::StatusWord(sw.0));
    }
    Ok((data.to_vec(), sw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_application_apdu_is_well_formed() {
        let bytes = Apdu::select_ndef_application().to_bytes();
        assert_eq!(
            hex::encode(&bytes),
            "00a4040007d276000085010100"
        );
    }

    #[test]
    fn native_command_wraps_with_cla_90_and_le_00() {
        let bytes = Apdu::native(0xF5, [0x02]).to_bytes(); // GetFileSettings(file 2)
        assert_eq!(hex::encode(&bytes), "90f500000102" .to_string() + "00");
    }

    #[test]
    fn split_response_extracts_status_word() {
        let (data, sw) = split_response(&[0xDE, 0xAD, 0x91, 0x00]).unwrap();
        assert_eq!(data, &[0xDE, 0xAD]);
        assert!(sw.is_ok());
        assert!(split_response(&[0x90]).is_err());
    }
}
