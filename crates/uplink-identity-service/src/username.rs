//! Username normalization for hosted vanity identities (Phase 5b).
//!
//! A username is the local part of both the NIP-05 name and the LNURL-pay / LUD-16 address,
//! so it must satisfy the LUD-16 local-part charset. Normalization is deterministic and pure
//! (no I/O) so it is byte-identical on the client and the backend.

use crate::error::IdentityServiceError;

/// Minimum username length (characters).
pub const MIN_USERNAME_LEN: usize = 1;
/// Maximum username length (characters).
pub const MAX_USERNAME_LEN: usize = 32;

/// Names that may never be registered (root NIP-05 id, infra / route collisions).
const RESERVED: &[&str] = &[
    "_", "admin", "root", "support", "abuse", "postmaster", "webmaster",
    "api", "well-known", "nostr", "lnurlp", "lnurl", "openagents", "uplink",
];

/// A validated, normalized username (lowercase, LUD-16-safe local part).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedUsername(String);

impl NormalizedUsername {
    /// Normalize and validate free-form input into a username.
    ///
    /// Rules: trimmed, lowercased; charset `[a-z0-9._-]`; length in
    /// `[MIN_USERNAME_LEN, MAX_USERNAME_LEN]`; must start and end with an alphanumeric;
    /// not a reserved name.
    pub fn parse(input: &str) -> Result<Self, IdentityServiceError> {
        let s = input.trim().to_lowercase();
        if s.len() < MIN_USERNAME_LEN || s.len() > MAX_USERNAME_LEN {
            return Err(IdentityServiceError::UsernameInvalid);
        }
        let bytes = s.as_bytes();
        let is_alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
        if !is_alnum(bytes[0]) || !is_alnum(bytes[bytes.len() - 1]) {
            return Err(IdentityServiceError::UsernameInvalid);
        }
        let charset_ok = s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));
        if !charset_ok {
            return Err(IdentityServiceError::UsernameInvalid);
        }
        if RESERVED.contains(&s.as_str()) {
            return Err(IdentityServiceError::UsernameInvalid);
        }
        Ok(Self(s))
    }

    /// The normalized username as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NormalizedUsername {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_case_and_whitespace() {
        let u = NormalizedUsername::parse("  Alice  ").unwrap();
        assert_eq!(u.as_str(), "alice");
    }

    #[test]
    fn allows_lud16_charset() {
        assert_eq!(NormalizedUsername::parse("a.b_c-1").unwrap().as_str(), "a.b_c-1");
    }

    #[test]
    fn rejects_bad_charset_and_edges() {
        assert!(NormalizedUsername::parse("space bar").is_err());
        assert!(NormalizedUsername::parse(".leading").is_err());
        assert!(NormalizedUsername::parse("trailing-").is_err());
        assert!(NormalizedUsername::parse("uni\u{20ac}ode").is_err());
    }

    #[test]
    fn rejects_length_bounds() {
        assert!(NormalizedUsername::parse("").is_err());
        let long = "a".repeat(MAX_USERNAME_LEN + 1);
        assert!(NormalizedUsername::parse(&long).is_err());
    }

    #[test]
    fn rejects_reserved() {
        assert!(NormalizedUsername::parse("_").is_err());
        assert!(NormalizedUsername::parse("Admin").is_err());
        assert!(NormalizedUsername::parse("nostr").is_err());
    }
}
