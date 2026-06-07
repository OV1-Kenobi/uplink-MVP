//! End-to-end test: run the pipeline over fixture HTML and emit a full
//! artifact bundle to a temp dir, then verify every artifact lands and the
//! accountability seal holds in the written `RECEIPT.json`.

use samplegen::{Industry, PipelineConfig, TemplateCopyGenerator, run_pipeline, write_bundle};

const FIXTURE_HTML: &str = r#"<!doctype html>
<html><head>
<title>Sentinel Guard Co. | Bay Area Security</title>
<meta property="og:site_name" content="Sentinel Guard Co.">
<meta name="description" content="Licensed guards, mobile patrol, and event security across the Bay Area.">
</head><body>
<h1>Protection you can prove.</h1>
<a href="tel:+14155550199">(415) 555-0199</a>
<ul>
  <li>Mobile Patrol</li>
  <li>Armed Guards</li>
  <li>Event Security</li>
</ul>
</body></html>"#;

#[test]
fn pipeline_emits_full_bundle_to_temp_dir() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let mut config = PipelineConfig::new("Fallback Co.");
    config.industry = Industry::PrivateSecurity;
    config.source_url = Some("https://sentinelguard.example".to_string());

    let bundle = run_pipeline(FIXTURE_HTML, &config, &TemplateCopyGenerator);

    // Extraction picked up the real brand, not the fallback.
    assert_eq!(bundle.brand.name, "Sentinel Guard Co.");
    assert!(bundle.accountability.verify());

    let written = write_bundle(&bundle, &config, dir.path()).expect("write bundle");

    // Every expected artifact exists on disk.
    for name in ["index.html", "portal.html", "sample.json", "RECEIPT.json"] {
        let path = dir.path().join(name);
        assert!(path.exists(), "missing artifact: {name}");
        let bytes = std::fs::read(&path).expect("read artifact");
        assert!(!bytes.is_empty(), "empty artifact: {name}");
    }
    assert_eq!(written.len(), 4, "expected 4 artifacts written");

    // The rendered landing page carries the brand and links the portal.
    let index = std::fs::read_to_string(dir.path().join("index.html")).expect("read index");
    assert!(index.contains("Sentinel Guard Co."));
    assert!(index.contains("portal.html"));

    // The receipt records a verified, matching accountability seal.
    let receipt_raw =
        std::fs::read_to_string(dir.path().join("RECEIPT.json")).expect("read receipt");
    let receipt: serde_json::Value = serde_json::from_str(&receipt_raw).expect("parse receipt");
    assert_eq!(receipt["schema"], "openagents.samplegen.receipt.v1");
    assert_eq!(receipt["client_name"], "Sentinel Guard Co.");
    assert_eq!(receipt["industry"], "private-security");
    assert_eq!(receipt["accountability_verified"], true);
    assert_eq!(
        receipt["accountability_seal_sha256"],
        serde_json::Value::String(bundle.accountability.canonical_json_sha256.clone())
    );
    assert_eq!(
        receipt["artifacts"].as_array().map(Vec::len),
        Some(3),
        "receipt should list the 3 content artifacts"
    );
}
