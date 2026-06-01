//! Profile resolution (NIP-01 metadata, NIP-05 verification).

use nostr::{Metadata, PublicKey};
use nostr_sdk::prelude::*;
use crate::relay::RelayPool;

/// A resolved user profile.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedProfile {
    pub npub: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub about: Option<String>,
    pub picture: Option<String>,
    pub nip05: Option<String>,
    pub nip05_verified: bool,
}

impl RelayPool {
    /// Resolve a profile for the given npub by querying the relay pool.
    pub async fn resolve_profile(&self, public_key: PublicKey) -> anyhow::Result<ResolvedProfile> {
        let client = self.client();

        // 1. Fetch metadata (kind 0)
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);

        let events = client.fetch_events(vec![filter]).timeout(std::time::Duration::from_secs(5)).await?;

        let mut profile = ResolvedProfile {
            npub: public_key.to_bech32()?,
            name: None,
            display_name: None,
            about: None,
            picture: None,
            nip05: None,
            nip05_verified: false,
        };

        if let Some(event) = events.first() {
            let metadata = Metadata::from_json(&event.content)?;
            profile.name = metadata.name;
            profile.display_name = metadata.display_name;
            profile.about = metadata.about;
            profile.picture = metadata.picture;
            profile.nip05 = metadata.nip05;

            // 2. NIP-05 verification (if present)
            if let Some(ref nip05) = profile.nip05 {
                profile.nip05_verified = verify_nip05(public_key, nip05).await.unwrap_or(false);
            }
        }

        Ok(profile)
    }
}

async fn verify_nip05(_pk: PublicKey, _nip05: &str) -> anyhow::Result<bool> {
    // Phase A2 stub: NIP-05 verification requires an HTTP client.
    // Since we're in a WASM-ready crate, we'll implement this later
    // or use the wasm-bindgen boundary for the HTTP check.
    Ok(false)
}
