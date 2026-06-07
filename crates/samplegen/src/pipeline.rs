//! Pipeline orchestration: ingest -> generate -> differentiator -> emit.
//!
//! [`run_pipeline`] is pure and offline (operates on an HTML string), so it is
//! fully testable. Network ingestion lives in [`fetch_url`] and is called by the
//! CLI before handing HTML to the pipeline.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::accountability::AccountabilityDemo;
use crate::brand::{Brand, extract_brand_from_html};
use crate::generate::{CopyGenerator, GeneratedCopy};
use crate::site::{render_landing_page, render_portal_page};

/// Target industry; selects the copy angle and differentiator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Industry {
    /// Private security / guarding (beachhead).
    PrivateSecurity,
    /// Specialty construction trades.
    ConstructionTrades,
    /// Anything else.
    Generic,
}

impl Industry {
    fn as_str(self) -> &'static str {
        match self {
            Industry::PrivateSecurity => "private-security",
            Industry::ConstructionTrades => "construction-trades",
            Industry::Generic => "generic",
        }
    }
}

/// Inputs for a single sample-upgrade run.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub fallback_name: String,
    pub industry: Industry,
    pub source_url: Option<String>,
    pub reference: DateTime<Utc>,
}

impl PipelineConfig {
    pub fn new(fallback_name: impl Into<String>) -> Self {
        Self {
            fallback_name: fallback_name.into(),
            industry: Industry::PrivateSecurity,
            source_url: None,
            reference: default_reference(),
        }
    }
}

fn default_reference() -> DateTime<Utc> {
    match Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => Utc::now(),
    }
}

/// Everything produced for one prospect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleBundle {
    pub brand: Brand,
    pub copy: GeneratedCopy,
    pub accountability: AccountabilityDemo,
    pub landing_html: String,
    pub portal_html: String,
}

/// Run the full pipeline over already-fetched HTML. Pure and deterministic.
pub fn run_pipeline(
    html: &str,
    config: &PipelineConfig,
    generator: &dyn CopyGenerator,
) -> SampleBundle {
    let brand = extract_brand_from_html(html, &config.fallback_name, config.source_url.clone());
    let copy = generator.generate(&brand, config.industry);
    let accountability = AccountabilityDemo::sample(&brand, config.reference);
    let landing_html = render_landing_page(&brand, &copy, "portal.html");
    let portal_html = render_portal_page(&brand, &accountability);
    SampleBundle {
        brand,
        copy,
        accountability,
        landing_html,
        portal_html,
    }
}

/// Fetch a URL's HTML. Network access required.
pub async fn fetch_url(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("openagents-samplegen/0.1")
        .timeout(Duration::from_secs(20))
        .build()
        .context("build http client")?;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("fetch {url}"))?
        .error_for_status()
        .with_context(|| format!("non-success status from {url}"))?;
    resp.text().await.context("read response body")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Write the bundle's artifacts to `out_dir`, returning the paths written.
pub fn write_bundle(
    bundle: &SampleBundle,
    config: &PipelineConfig,
    out_dir: &Path,
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;

    let mut files: Vec<(String, String)> = vec![
        ("index.html".to_string(), bundle.landing_html.clone()),
        ("portal.html".to_string(), bundle.portal_html.clone()),
    ];
    let sample_json =
        serde_json::to_string_pretty(bundle).context("serialize sample bundle json")?;
    files.push(("sample.json".to_string(), sample_json));

    let artifacts: Vec<serde_json::Value> = files
        .iter()
        .map(|(name, content)| {
            json!({
                "path": name,
                "sha256": sha256_hex(content.as_bytes()),
                "bytes": content.len(),
            })
        })
        .collect();

    let receipt = json!({
        "schema": "openagents.samplegen.receipt.v1",
        "generated_at": config.reference.to_rfc3339(),
        "client_name": bundle.brand.name,
        "industry": config.industry.as_str(),
        "source_url": config.source_url,
        "accountability_seal_sha256": bundle.accountability.canonical_json_sha256,
        "accountability_verified": bundle.accountability.verify(),
        "artifacts": artifacts,
    });
    files.push((
        "RECEIPT.json".to_string(),
        serde_json::to_string_pretty(&receipt).context("serialize receipt")?,
    ));

    let mut written = Vec::new();
    for (name, content) in &files {
        let path = out_dir.join(name);
        std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}
