//! Relay allowlist roles (ADR-U-011 §2). Each enrolled `relay_auth_keys` pubkey carries a
//! role that authorizes which attendance kinds it may write.

use serde::{Deserialize, Serialize};

/// Role of an enrolled relay key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// A worker: may author the user tap kind only.
    Worker,
    /// The backend writer: authors authoritative session/interval kinds.
    Backend,
    /// An admin: authors admin kinds.
    Admin,
}

impl Role {
    /// Parse a stored role discriminant.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "worker" => Some(Self::Worker),
            "backend" => Some(Self::Backend),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    /// The stable string discriminant for this role.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Worker => "worker",
            Self::Backend => "backend",
            Self::Admin => "admin",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_known_roles() {
        for r in [Role::Worker, Role::Backend, Role::Admin] {
            assert_eq!(Role::parse(r.as_str()), Some(r));
        }
        assert_eq!(Role::parse("nobody"), None);
    }
}
