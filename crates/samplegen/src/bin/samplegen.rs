//! `samplegen` CLI: turn a prospect's website into a free branded sample
//! upgrade (modern landing page + accountability portal demo) on disk.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use samplegen::pipeline::fetch_url;
use samplegen::{Industry, PipelineConfig, TemplateCopyGenerator, run_pipeline, write_bundle};

#[derive(Parser)]
#[command(
    name = "samplegen",
    about = "Generate a free branded sample-upgrade (landing page + accountability portal demo)"
)]
struct Cli {
    /// URL of the prospect's existing website (fetched over the network).
    #[arg(long, conflicts_with = "html_file")]
    url: Option<String>,

    /// Local HTML file to use instead of fetching a URL.
    #[arg(long)]
    html_file: Option<PathBuf>,

    /// Fallback brand name used if extraction finds none.
    #[arg(long, default_value = "Your Company")]
    name: String,

    /// Target industry (selects copy angle + differentiator).
    #[arg(long, value_enum, default_value = "private-security")]
    industry: Industry,

    /// Output directory for the generated artifacts.
    #[arg(long, default_value = "sample-out")]
    out: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (html, source_url) = match (&cli.url, &cli.html_file) {
        (Some(url), _) => (fetch_url(url).await?, Some(url.clone())),
        (None, Some(path)) => (
            std::fs::read_to_string(path)
                .with_context(|| format!("read html file {}", path.display()))?,
            None,
        ),
        (None, None) => bail!("provide --url <URL> or --html-file <PATH>"),
    };

    let mut config = PipelineConfig::new(cli.name.clone());
    config.industry = cli.industry;
    config.source_url = source_url;

    let bundle = run_pipeline(&html, &config, &TemplateCopyGenerator);
    let written = write_bundle(&bundle, &config, &cli.out)?;

    println!("Generated sample upgrade for: {}", bundle.brand.name);
    println!(
        "Accountability seal (sha256): {}",
        bundle.accountability.canonical_json_sha256
    );
    println!("Artifacts:");
    for path in &written {
        println!("  {}", path.display());
    }
    println!("Open {}/index.html in a browser.", cli.out.display());
    Ok(())
}
