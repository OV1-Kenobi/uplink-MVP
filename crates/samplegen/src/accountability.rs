//! Verifiable Accountability Layer demo (the private-security differentiator).
//!
//! Produces a company-issued digital identity, an AI-style incident report, and
//! a signed, hash-chained patrol trajectory, then seals them with a
//! tamper-evident `sha256`-over-canonical-JSON digest — mirroring the repo's
//! `canonical_json_sha256` convention.
//!
//! This is a **demo** artifact for sales samples: identities here are derived
//! deterministically for display. In production these are real `UnifiedIdentity`
//! keys (BIP39 -> Nostr NIP-06 + Spark BIP44) and receipts are signed.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::brand::Brand;
use crate::canon::{canonicalize, sha256_hex};
use crate::trajectory::PatrolTrajectory;

/// Company-issued digital identity for a person or agent (demo form).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DigitalIdentity {
    pub holder_name: String,
    pub role: String,
    pub employee_id: String,
    pub pubkey: String,
    pub npub_demo: String,
    pub issued_at: DateTime<Utc>,
}

impl DigitalIdentity {
    /// Issue a deterministic demo identity bound to `company` + `employee_id`.
    pub fn issue(
        company: &str,
        holder_name: &str,
        role: &str,
        employee_id: &str,
        issued_at: DateTime<Utc>,
    ) -> Self {
        let seed = format!("{company}|{employee_id}|{holder_name}");
        let pubkey = sha256_hex(seed.as_bytes());
        let npub_demo = format!("npub1demo{}", pubkey.get(..40).unwrap_or(&pubkey));
        Self {
            holder_name: holder_name.to_string(),
            role: role.to_string(),
            employee_id: employee_id.to_string(),
            pubkey,
            npub_demo,
            issued_at,
        }
    }
}

/// AI-style incident report tied to an officer identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentReport {
    pub report_id: String,
    pub site: String,
    pub occurred_at: DateTime<Utc>,
    pub officer_name: String,
    pub officer_pubkey: String,
    pub category: String,
    pub summary: String,
    pub narrative: String,
    pub actions_taken: Vec<String>,
}

/// The sealed accountability bundle shown in the client portal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountabilityDemo {
    pub client_name: String,
    pub officer: DigitalIdentity,
    pub incident: IncidentReport,
    pub trajectory: PatrolTrajectory,
    pub canonical_json_sha256: String,
    pub generated_at: DateTime<Utc>,
}

impl AccountabilityDemo {
    /// Build a deterministic sample for `brand` anchored at `reference`.
    pub fn sample(brand: &Brand, reference: DateTime<Utc>) -> Self {
        let officer = DigitalIdentity::issue(
            &brand.name,
            "J. Rivera",
            "Mobile Patrol Officer",
            "G-04821",
            reference - Duration::days(120),
        );
        let site = brand
            .location
            .clone()
            .unwrap_or_else(|| format!("{} — Client Site, Loading Dock B", brand.name));
        let occurred_at = reference - Duration::hours(6);
        let incident = IncidentReport {
            report_id: "IR-2026-0142".to_string(),
            site,
            occurred_at,
            officer_name: officer.holder_name.clone(),
            officer_pubkey: officer.pubkey.clone(),
            category: "Unauthorized Access Attempt".to_string(),
            summary: "Individual attempted entry through Loading Dock B; denied and \
                      escorted off the premises without incident."
                .to_string(),
            narrative: "At 02:14 a male subject approached Loading Dock B and tried the \
                        service door. Officer Rivera responded within 40 seconds, \
                        verified the subject had no credentials, and denied entry per \
                        the site access policy. The subject was escorted to the property \
                        line and the site contact was notified via a logged message. No \
                        property loss or injuries observed."
                .to_string(),
            actions_taken: vec![
                "Responded to door-contact alert".to_string(),
                "Verified subject credentials (none presented)".to_string(),
                "Denied entry per access policy".to_string(),
                "Escorted subject to property line".to_string(),
                "Notified site contact via logged message".to_string(),
                "Filed report and sealed record".to_string(),
            ],
        };
        let trajectory = PatrolTrajectory::sample(&brand.name, &officer.employee_id, reference);
        let canonical_json_sha256 = seal(&incident, &trajectory, &officer, &brand.name);
        Self {
            client_name: brand.name.clone(),
            officer,
            incident,
            trajectory,
            canonical_json_sha256,
            generated_at: reference,
        }
    }

    /// Recompute the seal and confirm the bundle has not been altered.
    pub fn verify(&self) -> bool {
        self.trajectory
            .verify(&self.client_name, &self.officer.employee_id)
            && seal(
                &self.incident,
                &self.trajectory,
                &self.officer,
                &self.client_name,
            ) == self.canonical_json_sha256
    }
}

fn seal(
    incident: &IncidentReport,
    trajectory: &PatrolTrajectory,
    officer: &DigitalIdentity,
    client: &str,
) -> String {
    let payload = json!({
        "client": client,
        "officer_pubkey": officer.pubkey,
        "incident": incident,
        "trajectory_head": trajectory.chain_head,
        "trajectory_len": trajectory.stops.len(),
    });
    sha256_hex(canonicalize(&payload).as_bytes())
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

    fn demo() -> AccountabilityDemo {
        AccountabilityDemo::sample(&Brand::placeholder("Acme Security"), reference())
    }

    #[test]
    fn seal_is_deterministic_and_verifies() {
        let a = demo();
        let b = demo();
        assert_eq!(a.canonical_json_sha256, b.canonical_json_sha256);
        assert_eq!(a.canonical_json_sha256.len(), 64);
        assert!(a.verify());
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut d = demo();
        d.incident.summary = "Nothing happened.".to_string();
        assert!(!d.verify());
    }

    #[test]
    fn identity_is_bound_to_company_and_employee() {
        let d = demo();
        assert_eq!(d.officer.employee_id, "G-04821");
        assert!(d.officer.npub_demo.starts_with("npub1demo"));
        assert_eq!(d.incident.officer_pubkey, d.officer.pubkey);
    }
}
