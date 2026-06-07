//! Signed, hash-chained, mesh-witnessed patrol trajectory (deterministic mock).
//!
//! Models a mixed indoor/outdoor patrol round where each stop is a
//! `LocationAttestation`:
//!
//! * **Indoor** legs are located by **BLE beacons** (no GPS fix).
//! * **Outdoor** legs are located by **Meshtastic GPS** and corroborated by
//!   independent mesh relay nodes that co-sign the guard's packet
//!   ("mesh-witnessed").
//!
//! Every stop is signed with the officer's company-issued guard key, then
//! hash-chained to the previous stop (`prev_hash`) so the ordering is
//! tamper-evident. This is a **demo**: keys are derived deterministically via
//! SHA-256 (an HMAC-style construction) so sales samples reproduce byte-for-byte
//! and need no hardware. In production these become real `UnifiedIdentity`
//! signatures (BIP39 -> Nostr) over the same canonical payloads.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::canon::{canonicalize, sha256_hex};

const GENESIS: &str = "GENESIS";

/// How a stop's location was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PositioningMethod {
    /// Indoor zone resolution from a company BLE beacon.
    IndoorBle,
    /// Outdoor GPS fix carried over the Meshtastic mesh.
    OutdoorMeshGps,
}

impl PositioningMethod {
    /// Short tag used in the canonical payload / CSS hook.
    pub fn tag(self) -> &'static str {
        match self {
            Self::IndoorBle => "indoor-ble",
            Self::OutdoorMeshGps => "outdoor-mesh-gps",
        }
    }

    /// Human-readable label for the portal.
    pub fn label(self) -> &'static str {
        match self {
            Self::IndoorBle => "Indoor · BLE beacon",
            Self::OutdoorMeshGps => "Outdoor · Meshtastic GPS",
        }
    }
}

/// A GPS fix (only present on outdoor mesh legs).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

/// An independent mesh relay node that co-signs (witnesses) a guard packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshWitness {
    pub node_label: String,
    pub node_id: String,
    pub rssi_dbm: i32,
    pub cosignature: String,
}

/// One signed, witnessed, hash-chained stop on the patrol round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationAttestation {
    pub seq: u64,
    pub observed_at: DateTime<Utc>,
    pub zone: String,
    pub method: PositioningMethod,
    pub geo: Option<GeoPoint>,
    pub status: String,
    pub note: Option<String>,
    pub prev_hash: String,
    pub guard_sig: String,
    pub witnesses: Vec<MeshWitness>,
    pub entry_hash: String,
}

/// The full signed trajectory plus the chain head (last entry hash).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatrolTrajectory {
    pub stops: Vec<LocationAttestation>,
    pub chain_head: String,
}

/// Derive the deterministic demo guard-signing key for a company + employee.
fn guard_secret(company: &str, employee_id: &str) -> String {
    sha256_hex(format!("{company}|{employee_id}|guard-signing-key").as_bytes())
}

/// Derive a deterministic mesh-node public id + secret from its label.
fn node_keys(node_label: &str) -> (String, String) {
    let node_id = sha256_hex(format!("mesh-node|{node_label}").as_bytes());
    let node_secret = sha256_hex(format!("mesh-node-secret|{node_label}").as_bytes());
    (node_id, node_secret)
}

/// Canonical core the guard signs: identity-bearing fields + chain link.
fn core_value(att: &LocationAttestation) -> Value {
    json!({
        "seq": att.seq,
        "observed_at": att.observed_at,
        "zone": att.zone,
        "method": att.method,
        "geo": att.geo,
        "status": att.status,
        "note": att.note,
        "prev_hash": att.prev_hash,
    })
}

/// Guard signature over the canonical core (HMAC-style demo construction).
fn guard_sign(secret: &str, att: &LocationAttestation) -> String {
    sha256_hex(format!("{secret}|{}", canonicalize(&core_value(att))).as_bytes())
}

/// Witness co-signature over the guard signature + relay metadata.
fn witness_cosign(node_secret: &str, guard_sig: &str, rssi_dbm: i32) -> String {
    sha256_hex(format!("{node_secret}|{guard_sig}|{rssi_dbm}").as_bytes())
}

/// Entry hash over the whole attestation (sig + witnesses included), with the
/// `entry_hash` field itself excluded so it can seal everything else.
fn entry_hash_of(att: &LocationAttestation) -> String {
    let mut v = serde_json::to_value(att).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut v {
        map.remove("entry_hash");
    }
    sha256_hex(canonicalize(&v).as_bytes())
}

/// Declarative spec for one stop on the sample round.
struct StopSpec {
    minute: i64,
    zone: &'static str,
    method: PositioningMethod,
    geo: Option<GeoPoint>,
    status: &'static str,
    note: Option<&'static str>,
    /// Mesh relay labels that witness this stop (outdoor legs only).
    witnesses: &'static [&'static str],
}

impl PatrolTrajectory {
    /// Build a deterministic mixed indoor-BLE + outdoor-Meshtastic-GPS round,
    /// signed by `company` + `employee_id` and anchored at `reference`.
    pub fn sample(company: &str, employee_id: &str, reference: DateTime<Utc>) -> Self {
        let base = reference - Duration::hours(7);
        let g = |lat: f64, lon: f64| Some(GeoPoint { lat, lon });
        let specs = [
            StopSpec {
                minute: 0,
                zone: "Perimeter — North Gate",
                method: PositioningMethod::OutdoorMeshGps,
                geo: g(37.774_93, -122.419_42),
                status: "Secure",
                note: None,
                witnesses: &["Relay-Alpha", "Relay-Bravo"],
            },
            StopSpec {
                minute: 18,
                zone: "West Entrance Vestibule",
                method: PositioningMethod::IndoorBle,
                geo: None,
                status: "Secure",
                note: None,
                witnesses: &[],
            },
            StopSpec {
                minute: 22,
                zone: "Loading Dock B",
                method: PositioningMethod::IndoorBle,
                geo: None,
                status: "Alert",
                note: Some("Door-contact alert; see IR-2026-0142"),
                witnesses: &[],
            },
            StopSpec {
                minute: 48,
                zone: "Server Room Corridor",
                method: PositioningMethod::IndoorBle,
                geo: None,
                status: "Secure",
                note: None,
                witnesses: &[],
            },
            StopSpec {
                minute: 63,
                zone: "East Stairwell Exit",
                method: PositioningMethod::IndoorBle,
                geo: None,
                status: "Secure",
                note: None,
                witnesses: &[],
            },
            StopSpec {
                minute: 71,
                zone: "Parking Level P2 (open deck)",
                method: PositioningMethod::OutdoorMeshGps,
                geo: g(37.775_31, -122.418_77),
                status: "Secure",
                note: None,
                witnesses: &["Relay-Alpha", "Relay-Charlie"],
            },
        ];

        let secret = guard_secret(company, employee_id);
        let mut stops: Vec<LocationAttestation> = Vec::with_capacity(specs.len());
        let mut prev_hash = GENESIS.to_string();
        for (idx, spec) in specs.iter().enumerate() {
            let seq = (idx as u64) + 1;
            let mut att = LocationAttestation {
                seq,
                observed_at: base + Duration::minutes(spec.minute),
                zone: spec.zone.to_string(),
                method: spec.method,
                geo: spec.geo,
                status: spec.status.to_string(),
                note: spec.note.map(str::to_string),
                prev_hash: prev_hash.clone(),
                guard_sig: String::new(),
                witnesses: Vec::new(),
                entry_hash: String::new(),
            };
            att.guard_sig = guard_sign(&secret, &att);
            att.witnesses = spec
                .witnesses
                .iter()
                .enumerate()
                .map(|(hop, label)| {
                    let (node_id, node_secret) = node_keys(label);
                    let rssi_dbm = -68 - (hop as i32) * 9 - (seq as i32);
                    let cosignature = witness_cosign(&node_secret, &att.guard_sig, rssi_dbm);
                    MeshWitness {
                        node_label: (*label).to_string(),
                        node_id,
                        rssi_dbm,
                        cosignature,
                    }
                })
                .collect();
            att.entry_hash = entry_hash_of(&att);
            prev_hash = att.entry_hash.clone();
            stops.push(att);
        }
        let chain_head = stops
            .last()
            .map_or_else(|| GENESIS.to_string(), |s| s.entry_hash.clone());
        Self { stops, chain_head }
    }

    /// Recompute every signature, witness co-signature, and chain link to
    /// confirm the trajectory has not been altered.
    pub fn verify(&self, company: &str, employee_id: &str) -> bool {
        let secret = guard_secret(company, employee_id);
        let mut expected_prev = GENESIS.to_string();
        for att in &self.stops {
            if att.prev_hash != expected_prev {
                return false;
            }
            if att.guard_sig != guard_sign(&secret, att) {
                return false;
            }
            for w in &att.witnesses {
                let (node_id, node_secret) = node_keys(&w.node_label);
                if w.node_id != node_id
                    || w.cosignature != witness_cosign(&node_secret, &att.guard_sig, w.rssi_dbm)
                {
                    return false;
                }
            }
            if att.entry_hash != entry_hash_of(att) {
                return false;
            }
            expected_prev = att.entry_hash.clone();
        }
        self.chain_head == expected_prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn reference() -> DateTime<Utc> {
        match Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => Utc::now(),
        }
    }

    fn demo() -> PatrolTrajectory {
        PatrolTrajectory::sample("Acme Security", "G-04821", reference())
    }

    #[test]
    fn trajectory_is_deterministic() {
        let a = demo();
        let b = demo();
        assert_eq!(a, b);
        assert_eq!(a.stops.len(), 6);
    }

    #[test]
    fn mixed_methods_and_witnesses_present() {
        let t = demo();
        assert!(
            t.stops
                .iter()
                .any(|s| s.method == PositioningMethod::IndoorBle)
        );
        let outdoor: Vec<_> = t
            .stops
            .iter()
            .filter(|s| s.method == PositioningMethod::OutdoorMeshGps)
            .collect();
        assert!(!outdoor.is_empty());
        assert!(
            outdoor
                .iter()
                .all(|s| s.geo.is_some() && !s.witnesses.is_empty())
        );
    }

    #[test]
    fn chain_links_and_head_are_consistent() {
        let t = demo();
        assert_eq!(t.stops[0].prev_hash, GENESIS);
        for w in t.stops.windows(2) {
            assert_eq!(w[1].prev_hash, w[0].entry_hash);
        }
        assert_eq!(t.chain_head, t.stops.last().unwrap().entry_hash);
        assert!(t.verify("Acme Security", "G-04821"));
    }

    #[test]
    fn tampering_with_a_zone_breaks_verification() {
        let mut t = demo();
        t.stops[2].zone = "Somewhere else".to_string();
        assert!(!t.verify("Acme Security", "G-04821"));
    }

    #[test]
    fn forged_witness_breaks_verification() {
        let mut t = demo();
        if let Some(w) = t.stops[0].witnesses.first_mut() {
            w.cosignature = "0".repeat(64);
        }
        assert!(!t.verify("Acme Security", "G-04821"));
    }

    #[test]
    fn wrong_signer_fails() {
        let t = demo();
        assert!(!t.verify("Acme Security", "G-99999"));
        assert!(!t.verify("Other Co", "G-04821"));
    }
}
