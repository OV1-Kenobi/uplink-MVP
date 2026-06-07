//! Branded website sample-upgrade generator.
//!
//! Turns a prospect's existing website into a free, deployable sample upgrade:
//! a modern branded landing page plus an industry differentiator demo. For the
//! private-security beachhead the differentiator is a **Verifiable
//! Accountability Layer**: a client portal showing a tamper-evident,
//! AI-generated incident report and patrol log tied to a company-issued
//! digital identity.
//!
//! Pipeline stages: ingest (brand) -> generate (copy) -> differentiator
//! (accountability demo) -> emit (HTML + receipt artifacts).
//!
//! Inference is abstracted behind [`generate::CopyGenerator`] so the cheap
//! Gemini/`LM` provider can be dropped in later; the bundled
//! [`generate::TemplateCopyGenerator`] is a complete, deterministic generator
//! that runs offline today.

pub mod accountability;
pub mod brand;
pub mod canon;
pub mod generate;
pub mod pipeline;
pub mod site;
pub mod trajectory;

pub use accountability::{AccountabilityDemo, DigitalIdentity, IncidentReport};
pub use brand::{Brand, extract_brand_from_html};
pub use generate::{CopyGenerator, GeneratedCopy, TemplateCopyGenerator};
pub use pipeline::{Industry, PipelineConfig, SampleBundle, run_pipeline, write_bundle};
pub use trajectory::{
    GeoPoint, LocationAttestation, MeshWitness, PatrolTrajectory, PositioningMethod,
};
