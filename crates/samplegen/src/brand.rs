//! Dependency-light brand extraction from a prospect's existing HTML.
//!
//! Pure functions over an HTML string so ingestion is fully testable offline;
//! network fetching lives in [`crate::pipeline`].

use serde::{Deserialize, Serialize};

/// Brand profile distilled from an existing site.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Brand {
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub primary_color: String,
    pub accent_color: String,
    pub services: Vec<String>,
    pub location: Option<String>,
    pub phone: Option<String>,
    pub source_url: Option<String>,
}

impl Brand {
    /// Sensible defaults when a site yields little signal.
    pub fn placeholder(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tagline: "Trusted protection, proven every shift.".to_string(),
            description: "Professional security services for businesses that \
                          cannot afford surprises."
                .to_string(),
            primary_color: "#0f172a".to_string(),
            accent_color: "#f59e0b".to_string(),
            services: Vec::new(),
            location: None,
            phone: None,
            source_url: None,
        }
    }
}

/// Extract a [`Brand`] from raw HTML, falling back to `fallback_name`.
pub fn extract_brand_from_html(
    html: &str,
    fallback_name: &str,
    source_url: Option<String>,
) -> Brand {
    let lower = html.to_ascii_lowercase();
    let mut brand = Brand::placeholder(fallback_name);
    brand.source_url = source_url;

    if let Some(site) = meta_content(html, &lower, &["property=\"og:site_name\""]) {
        if !site.trim().is_empty() {
            brand.name = clean_text(&site);
        }
    } else if let Some(title) = tag_text(html, &lower, "title") {
        brand.name = clean_text(&split_title(&title));
    }

    if let Some(h1) = tag_text(html, &lower, "h1") {
        let t = clean_text(&h1);
        if !t.is_empty() {
            brand.tagline = t;
        }
    }

    if let Some(desc) = meta_content(
        html,
        &lower,
        &["name=\"description\"", "property=\"og:description\""],
    ) {
        let t = clean_text(&desc);
        if !t.is_empty() {
            brand.description = t;
        }
    }

    let colors: Vec<String> = hex_colors(html)
        .into_iter()
        .filter(|c| is_vivid(c))
        .collect();
    if let Some(c) = colors.first() {
        brand.primary_color = c.clone();
    }
    if let Some(c) = colors.get(1) {
        brand.accent_color = c.clone();
    }

    brand.services = extract_services(html, &lower);
    let stripped = strip_tags(html);
    brand.phone = find_phone(&stripped);
    brand
}

fn extract_services(html: &str, lower: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in ["li", "h2", "h3"] {
        for raw in all_tag_texts(html, lower, tag) {
            let t = clean_text(&raw);
            let len = t.chars().count();
            if (3..=48).contains(&len) && !out.iter().any(|e| e.eq_ignore_ascii_case(&t)) {
                out.push(t);
            }
            if out.len() >= 6 {
                return out;
            }
        }
    }
    out
}

fn split_title(title: &str) -> String {
    for sep in [" | ", " - ", " – ", " — ", ": "] {
        if let Some(idx) = title.find(sep) {
            if let Some(head) = title.get(..idx) {
                if head.trim().len() >= 2 {
                    return head.to_string();
                }
            }
        }
    }
    title.to_string()
}

fn tag_text(html: &str, lower: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = lower.find(&open)?;
    let after = start + open.len();
    let gt = lower.get(after..)?.find('>')? + after + 1;
    let close = format!("</{tag}>");
    let end = lower.get(gt..)?.find(&close)? + gt;
    let raw = html.get(gt..end)?;
    Some(strip_tags(raw))
}

fn all_tag_texts(html: &str, lower: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(&open)) {
        let start = cursor + rel;
        let after = start + open.len();
        let Some(gt_rel) = lower.get(after..).and_then(|s| s.find('>')) else {
            break;
        };
        let content_start = after + gt_rel + 1;
        let Some(end_rel) = lower.get(content_start..).and_then(|s| s.find(&close)) else {
            break;
        };
        let content_end = content_start + end_rel;
        if let Some(raw) = html.get(content_start..content_end) {
            out.push(strip_tags(raw));
        }
        cursor = content_end + close.len();
    }
    out
}

fn meta_content(html: &str, lower: &str, markers: &[&str]) -> Option<String> {
    for marker in markers {
        let Some(mpos) = lower.find(marker) else {
            continue;
        };
        let tag_start = match lower.get(..mpos).and_then(|s| s.rfind('<')) {
            Some(x) => x,
            None => continue,
        };
        let Some(gt_rel) = lower.get(mpos..).and_then(|s| s.find('>')) else {
            continue;
        };
        let tag_end = mpos + gt_rel + 1;
        if let Some(tag) = html.get(tag_start..tag_end) {
            if let Some(v) = attr_value(tag, "content") {
                return Some(v);
            }
        }
    }
    None
}

fn attr_value(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let key = format!("{attr}=");
    let idx = lower.find(&key)? + key.len();
    let rest = tag.get(idx..)?;
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = idx + quote.len_utf8();
    let after = tag.get(value_start..)?;
    let end = after.find(quote)?;
    Some(after.get(..end)?.to_string())
}

fn hex_colors(html: &str) -> Vec<String> {
    let chars: Vec<char> = html.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if chars[i] == '#' && i + 7 <= n {
            let candidate: String = chars[i + 1..i + 7].iter().collect();
            if candidate.chars().all(|c| c.is_ascii_hexdigit()) {
                let color = format!("#{}", candidate.to_ascii_lowercase());
                if !out.contains(&color) {
                    out.push(color);
                }
                i += 7;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn rgb(color: &str) -> Option<(u8, u8, u8)> {
    let h = color.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(h.get(0..2)?, 16).ok()?;
    let g = u8::from_str_radix(h.get(2..4)?, 16).ok()?;
    let b = u8::from_str_radix(h.get(4..6)?, 16).ok()?;
    Some((r, g, b))
}

fn is_vivid(color: &str) -> bool {
    let Some((r, g, b)) = rgb(color) else {
        return false;
    };
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let spread = u16::from(max) - u16::from(min);
    let brightness = (u16::from(r) + u16::from(g) + u16::from(b)) / 3;
    spread > 24 && (16..=244).contains(&brightness)
}

fn find_phone(text: &str) -> Option<String> {
    let mut run = String::new();
    let mut best: Option<String> = None;
    for ch in text.chars() {
        if ch.is_ascii_digit() || "()+-. ".contains(ch) {
            run.push(ch);
        } else {
            check_run(&run, &mut best);
            run.clear();
        }
    }
    check_run(&run, &mut best);
    best
}

fn check_run(run: &str, best: &mut Option<String>) {
    if best.is_some() {
        return;
    }
    let digits = run.chars().filter(char::is_ascii_digit).count();
    if (10..=11).contains(&digits) {
        let t = run.trim();
        if t.len() >= 10 {
            *best = Some(t.to_string());
        }
    }
}

fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    clean_text(&out)
}

fn clean_text(input: &str) -> String {
    let decoded = input
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&rsquo;", "'")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        <html><head>
        <title>Acme Security | Home</title>
        <meta name="description" content="Guarding &amp; patrol you can trust.">
        <style>.btn{background:#1d4ed8;color:#f59e0b}</style>
        </head><body>
        <h1>Protecting what matters, around the clock</h1>
        <h2>Mobile Patrol</h2><h2>Event Security</h2>
        <ul><li>Armed Guards</li><li>Alarm Response</li></ul>
        <p>Call us at (415) 555-0199 today.</p>
        </body></html>
    "#;

    #[test]
    fn extracts_core_brand_fields() {
        let b = extract_brand_from_html(SAMPLE, "Fallback Co", Some("https://acme.example".into()));
        assert_eq!(b.name, "Acme Security");
        assert_eq!(b.tagline, "Protecting what matters, around the clock");
        assert_eq!(b.description, "Guarding & patrol you can trust.");
        assert_eq!(b.source_url.as_deref(), Some("https://acme.example"));
    }

    #[test]
    fn extracts_colors_and_services_and_phone() {
        let b = extract_brand_from_html(SAMPLE, "Fallback Co", None);
        assert_eq!(b.primary_color, "#1d4ed8");
        assert_eq!(b.accent_color, "#f59e0b");
        assert!(b.services.iter().any(|s| s == "Armed Guards"));
        assert!(b.services.iter().any(|s| s == "Mobile Patrol"));
        assert!(b.phone.is_some());
    }

    #[test]
    fn falls_back_when_empty() {
        let b = extract_brand_from_html("<html></html>", "Fallback Co", None);
        assert_eq!(b.name, "Fallback Co");
        assert_eq!(b.primary_color, "#0f172a");
        assert!(b.services.is_empty());
    }
}
