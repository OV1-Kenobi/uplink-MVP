//! Office tag directory (ADR-U-011 §4–5, §7 `office_tags`).
//!
//! Maps a verified NTAG 424 UID to its enrolled office and last-seen read counter. The
//! directory is *injected* read state: the Postgres edge loads the (small) `office_tags`
//! set into an implementation; the validator stays pure.

use std::collections::HashMap;

/// An enrolled office tag: which office it belongs to and its last accepted read counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficeTag {
    /// The tag's 7-byte UID (recovered by SDM verify).
    pub uid: [u8; 7],
    /// The office this tag gates.
    pub office_id: String,
    /// The last accepted SDM read counter for this UID; `None` if never tapped.
    pub last_read_ctr: Option<u32>,
}

/// Injected lookup of enrolled office tags by UID.
pub trait TagDirectory {
    /// The enrolled tag for `uid`, if any.
    fn office_tag(&self, uid: &[u8; 7]) -> Option<&OfficeTag>;
}

impl TagDirectory for HashMap<[u8; 7], OfficeTag> {
    fn office_tag(&self, uid: &[u8; 7]) -> Option<&OfficeTag> {
        self.get(uid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_directory_lookup() {
        let mut dir = HashMap::new();
        let uid = [1u8, 2, 3, 4, 5, 6, 7];
        dir.insert(
            uid,
            OfficeTag { uid, office_id: "hq".into(), last_read_ctr: Some(3) },
        );
        assert_eq!(dir.office_tag(&uid).unwrap().office_id, "hq");
        assert!(dir.office_tag(&[0u8; 7]).is_none());
    }
}
