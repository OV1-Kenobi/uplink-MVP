//! Marketing-copy generation for the upgraded sample site.
//!
//! [`CopyGenerator`] is the inference seam: the bundled
//! [`TemplateCopyGenerator`] is a complete deterministic generator that runs
//! offline today, and a cheap Gemini/`LM`-backed generator can implement the
//! same trait later without touching the rest of the pipeline.

use serde::{Deserialize, Serialize};

use crate::brand::Brand;
use crate::pipeline::Industry;

/// Structured, brand-aware copy for the upgraded landing page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedCopy {
    pub seo_title: String,
    pub seo_description: String,
    pub hero_headline: String,
    pub hero_subhead: String,
    pub cta_primary: String,
    pub cta_secondary: String,
    pub value_props: Vec<ValueProp>,
    pub about_paragraph: String,
}

/// A single benefit block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueProp {
    pub title: String,
    pub body: String,
}

impl ValueProp {
    fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}

/// Inference seam for producing upgraded copy from a [`Brand`].
pub trait CopyGenerator {
    fn generate(&self, brand: &Brand, industry: Industry) -> GeneratedCopy;
}

/// Deterministic, offline copy generator. Complete (not a stub).
#[derive(Debug, Default, Clone, Copy)]
pub struct TemplateCopyGenerator;

impl CopyGenerator for TemplateCopyGenerator {
    fn generate(&self, brand: &Brand, industry: Industry) -> GeneratedCopy {
        match industry {
            Industry::PrivateSecurity => security_copy(brand),
            Industry::ConstructionTrades => trades_copy(brand),
            Industry::Generic => generic_copy(brand),
        }
    }
}

fn truncate(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", head.trim_end())
}

fn security_copy(brand: &Brand) -> GeneratedCopy {
    let name = &brand.name;
    GeneratedCopy {
        seo_title: format!("{name} | Verifiable Security Services"),
        seo_description: truncate(&brand.description, 155),
        hero_headline: "Security you can prove — not just promise.".to_string(),
        hero_subhead: format!(
            "{name} now gives every client a live, tamper-proof record of exactly \
             what our guards did, on every shift — generated automatically."
        ),
        cta_primary: "See a live accountability portal".to_string(),
        cta_secondary: "Book a 10-minute walkthrough".to_string(),
        value_props: vec![
            ValueProp::new(
                "Verifiable accountability",
                "Every patrol checkpoint and incident is captured and sealed with a \
                 tamper-evident signature your clients can verify in real time.",
            ),
            ValueProp::new(
                "Reports that write themselves",
                "Guards speak or snap a photo; clean, professional reports are ready \
                 in seconds — so your team stays on site, not buried in paperwork.",
            ),
            ValueProp::new(
                "Defensible when it counts",
                "When a claim or dispute arrives, you hand over an unbroken, \
                 time-stamped chain of evidence instead of a clipboard.",
            ),
        ],
        about_paragraph: format!(
            "{}. We pair seasoned officers with a verifiable accountability layer, so \
             the protection you pay for is the protection you can prove.",
            truncate(&brand.description, 180).trim_end_matches('.')
        ),
    }
}

fn trades_copy(brand: &Brand) -> GeneratedCopy {
    let name = &brand.name;
    GeneratedCopy {
        seo_title: format!("{name} | Licensed, On-Time, Accountable"),
        seo_description: truncate(&brand.description, 155),
        hero_headline: "Quotes in minutes. Receipts you can trust.".to_string(),
        hero_subhead: format!(
            "{name} responds to every job request fast, with branded proposals and a \
             tamper-evident record of every change order and payment."
        ),
        cta_primary: "Get an instant estimate".to_string(),
        cta_secondary: "Call us now".to_string(),
        value_props: vec![
            ValueProp::new(
                "Never lose a change-order dispute",
                "Every approval, change, and payment is time-stamped and sealed, so \
                 'you never told me' and 'you never paid me' stop costing you money.",
            ),
            ValueProp::new(
                "Win the job while it's hot",
                "Leads get a branded, professional proposal in minutes — not days — \
                 so you book work your slower competitors never reach.",
            ),
            ValueProp::new(
                "Look like the pro you are",
                "A modern site and clean paperwork signal the quality your crews \
                 already deliver on the job.",
            ),
        ],
        about_paragraph: format!(
            "{}. We back craftsmanship with a verifiable paper trail, so every job is \
             documented, defensible, and easy to get paid for.",
            truncate(&brand.description, 180).trim_end_matches('.')
        ),
    }
}

fn generic_copy(brand: &Brand) -> GeneratedCopy {
    let name = &brand.name;
    GeneratedCopy {
        seo_title: format!("{name} | Modern, Trusted, Accountable"),
        seo_description: truncate(&brand.description, 155),
        hero_headline: format!("{name}, upgraded for the AI era."),
        hero_subhead: truncate(&brand.description, 160),
        cta_primary: "Get started".to_string(),
        cta_secondary: "Talk to us".to_string(),
        value_props: vec![
            ValueProp::new(
                "Move faster",
                "Cheap, reliable AI handles the busywork so your team focuses on the \
                 work that actually wins customers.",
            ),
            ValueProp::new(
                "Build trust",
                "Tamper-evident records for payments and messages mean every \
                 commitment is documented and verifiable.",
            ),
            ValueProp::new(
                "Look the part",
                "A clean, modern presence that matches the quality of your service.",
            ),
        ],
        about_paragraph: truncate(&brand.description, 220),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brand::Brand;

    fn sample_brand() -> Brand {
        let mut b = Brand::placeholder("Acme Security");
        b.description = "Guarding and patrol services across the Bay Area".to_string();
        b
    }

    #[test]
    fn security_copy_is_brand_aware_and_accountability_led() {
        let copy = TemplateCopyGenerator.generate(&sample_brand(), Industry::PrivateSecurity);
        assert!(copy.seo_title.contains("Acme Security"));
        assert!(copy.hero_subhead.contains("Acme Security"));
        assert_eq!(copy.value_props.len(), 3);
        assert!(
            copy.value_props[0]
                .title
                .to_lowercase()
                .contains("accountab")
        );
    }

    #[test]
    fn generator_is_deterministic() {
        let a = TemplateCopyGenerator.generate(&sample_brand(), Industry::PrivateSecurity);
        let b = TemplateCopyGenerator.generate(&sample_brand(), Industry::PrivateSecurity);
        assert_eq!(a, b);
    }

    #[test]
    fn trades_and_generic_render() {
        let t = TemplateCopyGenerator.generate(&sample_brand(), Industry::ConstructionTrades);
        assert!(t.cta_primary.to_lowercase().contains("estimate"));
        let g = TemplateCopyGenerator.generate(&sample_brand(), Industry::Generic);
        assert_eq!(g.value_props.len(), 3);
    }
}
