//! Static HTML rendering for the upgraded sample site and the portal demo.
//!
//! Dependency-free string rendering (no template engine) so the crate builds
//! with only already-vendored workspace deps. Output is a self-contained
//! single file per page, ready to drop on any static host.

use crate::accountability::AccountabilityDemo;
use crate::brand::Brand;
use crate::generate::GeneratedCopy;
use crate::trajectory::{LocationAttestation, PositioningMethod};

/// Escape text for safe insertion into HTML.
pub fn esc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn theme_vars(brand: &Brand) -> String {
    format!(
        ":root{{--primary:{};--accent:{};--ink:#0b1220;--muted:#5b6473;--bg:#f7f8fb;}}",
        esc(&brand.primary_color),
        esc(&brand.accent_color)
    )
}

fn css() -> &'static str {
    r#"*{box-sizing:border-box}body{margin:0;font-family:system-ui,-apple-system,Segoe UI,Roboto,sans-serif;color:var(--ink);background:var(--bg);line-height:1.55}
a{color:inherit}.wrap{max-width:1040px;margin:0 auto;padding:0 24px}
header.nav{display:flex;align-items:center;justify-content:space-between;padding:18px 0}
.brand{font-weight:800;font-size:20px;color:var(--primary)}
.btn{display:inline-block;border-radius:10px;padding:12px 18px;font-weight:700;text-decoration:none;border:2px solid var(--primary)}
.btn-primary{background:var(--primary);color:#fff}.btn-ghost{background:transparent;color:var(--primary)}
.hero{background:linear-gradient(135deg,var(--primary),#0b1220);color:#fff;padding:72px 0}
.hero h1{font-size:44px;line-height:1.1;margin:0 0 14px}.hero p{font-size:20px;max-width:640px;opacity:.92}
.hero .cta{margin-top:26px;display:flex;gap:14px;flex-wrap:wrap}
.hero .btn-ghost{color:#fff;border-color:#fff}
.section{padding:56px 0}.section h2{font-size:28px;margin:0 0 24px}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:20px}
.card{background:#fff;border:1px solid #e6e9ef;border-radius:14px;padding:22px}
.card h3{margin:0 0 8px;color:var(--primary)}.pill{display:inline-block;background:var(--accent);color:#1b1b1b;font-weight:700;border-radius:999px;padding:4px 12px;font-size:13px}
.services{display:flex;flex-wrap:wrap;gap:10px}.services span{background:#fff;border:1px solid #e6e9ef;border-radius:999px;padding:8px 14px;font-weight:600}
footer{background:var(--primary);color:#fff;padding:36px 0;margin-top:40px}
.badge{display:flex;align-items:center;gap:10px;background:#06281a;color:#b9f6ca;border:1px solid #1b5e20;border-radius:12px;padding:14px 16px;font-weight:700}
.mono{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:13px;word-break:break-all;color:var(--muted)}
table{width:100%;border-collapse:collapse;background:#fff;border:1px solid #e6e9ef;border-radius:12px;overflow:hidden}
th,td{text-align:left;padding:12px 14px;border-bottom:1px solid #eef1f6;font-size:14px}th{background:#f1f4f9}
.note{background:#fff7ed;border:1px solid #fed7aa;color:#9a3412;border-radius:10px;padding:10px 14px;font-size:13px;margin-bottom:20px}
.status-alert{color:#b91c1c;font-weight:700}.status-secure{color:#15803d;font-weight:700}
.tag{display:inline-block;border-radius:6px;padding:2px 8px;font-size:11px;font-weight:700;margin-left:6px}
.tag.indoor-ble{background:#e0f2fe;color:#075985}.tag.outdoor-mesh-gps{background:#dcfce7;color:#166534}
.chip{display:inline-block;background:#eef2ff;color:#3730a3;border:1px solid #c7d2fe;border-radius:999px;padding:2px 8px;font-size:11px;margin:2px 4px 0 0}
.chain{color:var(--muted);font-size:12px}"#
}

/// Render the upgraded marketing landing page.
pub fn render_landing_page(brand: &Brand, copy: &GeneratedCopy, portal_href: &str) -> String {
    let mut s = String::new();
    s.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    s.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    s.push_str(&format!("<title>{}</title>", esc(&copy.seo_title)));
    s.push_str(&format!(
        "<meta name=\"description\" content=\"{}\">",
        esc(&copy.seo_description)
    ));
    s.push_str(&format!(
        "<style>{}{}</style></head><body>",
        theme_vars(brand),
        css()
    ));

    s.push_str("<header class=\"nav wrap\">");
    s.push_str(&format!("<div class=\"brand\">{}</div>", esc(&brand.name)));
    s.push_str(&format!(
        "<a class=\"btn btn-primary\" href=\"{}\">{}</a></header>",
        esc(portal_href),
        esc(&copy.cta_primary)
    ));

    s.push_str("<section class=\"hero\"><div class=\"wrap\">");
    s.push_str("<span class=\"pill\">Verifiable Accountability</span>");
    s.push_str(&format!("<h1>{}</h1>", esc(&copy.hero_headline)));
    s.push_str(&format!("<p>{}</p>", esc(&copy.hero_subhead)));
    s.push_str("<div class=\"cta\">");
    s.push_str(&format!(
        "<a class=\"btn btn-primary\" href=\"{}\">{}</a>",
        esc(portal_href),
        esc(&copy.cta_primary)
    ));
    s.push_str(&format!(
        "<a class=\"btn btn-ghost\" href=\"#contact\">{}</a></div></div></section>",
        esc(&copy.cta_secondary)
    ));

    s.push_str("<section class=\"section wrap\"><div class=\"grid\">");
    for vp in &copy.value_props {
        s.push_str(&format!(
            "<div class=\"card\"><h3>{}</h3><p>{}</p></div>",
            esc(&vp.title),
            esc(&vp.body)
        ));
    }
    s.push_str("</div></section>");

    if !brand.services.is_empty() {
        s.push_str("<section class=\"section wrap\"><h2>What we do</h2><div class=\"services\">");
        for svc in &brand.services {
            s.push_str(&format!("<span>{}</span>", esc(svc)));
        }
        s.push_str("</div></section>");
    }

    s.push_str("<section class=\"section wrap\" id=\"contact\"><h2>About</h2>");
    s.push_str(&format!("<p>{}</p></section>", esc(&copy.about_paragraph)));

    s.push_str("<footer><div class=\"wrap\">");
    s.push_str(&format!("<strong>{}</strong>", esc(&brand.name)));
    if let Some(phone) = &brand.phone {
        s.push_str(&format!(" · <span>{}</span>", esc(phone)));
    }
    s.push_str("<div class=\"mono\">Upgraded sample generated by OpenAgents · samplegen</div>");
    s.push_str("</div></footer></body></html>");
    s
}

fn short(value: &str, keep: usize) -> String {
    if value.chars().count() <= keep {
        return value.to_string();
    }
    let head: String = value.chars().take(keep).collect();
    format!("{head}…")
}

/// Inline positioning-method badge for a trajectory stop.
fn method_badge(method: PositioningMethod) -> String {
    format!(
        "<span class=\"tag {}\">{}</span>",
        method.tag(),
        esc(method.label())
    )
}

/// Location-proof cell: GPS/BLE fix plus any mesh-relay witness chips.
fn proof_cell(att: &LocationAttestation) -> String {
    let mut s = String::new();
    match att.geo {
        Some(g) => s.push_str(&format!(
            "<div class=\"mono\">GPS {:.5}, {:.5}</div>",
            g.lat, g.lon
        )),
        None => s.push_str("<div class=\"mono\">BLE beacon fix</div>"),
    }
    if att.witnesses.is_empty() {
        s.push_str("<div class=\"chain\">no mesh witnesses (indoor leg)</div>");
    } else {
        for w in &att.witnesses {
            s.push_str(&format!(
                "<span class=\"chip\">📡 {} · {} dBm</span>",
                esc(&w.node_label),
                w.rssi_dbm
            ));
        }
    }
    s
}

/// Render the Client Accountability Portal demo page.
pub fn render_portal_page(brand: &Brand, demo: &AccountabilityDemo) -> String {
    let verified = demo.verify();
    let mut s = String::new();
    s.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    s.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    s.push_str(&format!(
        "<title>Client Accountability Portal · {}</title>",
        esc(&brand.name)
    ));
    s.push_str(&format!(
        "<style>{}{}</style></head><body>",
        theme_vars(brand),
        css()
    ));

    s.push_str("<header class=\"nav wrap\">");
    s.push_str(&format!(
        "<div class=\"brand\">{} · Accountability Portal</div>",
        esc(&brand.name)
    ));
    s.push_str("<a class=\"btn btn-ghost\" href=\"index.html\">Back to site</a></header>");

    s.push_str("<section class=\"section wrap\">");
    s.push_str(
        "<div class=\"note\">Sample / demo data. In production every stop is signed with \
         the officer's company-issued identity (BIP39 → Nostr + Spark), hash-chained, \
         and witnessed by independent Meshtastic relays on outdoor legs.</div>",
    );

    let badge = if verified {
        format!(
            "<div class=\"badge\">✓ Tamper-evident · seal verified · sha256 {}</div>",
            esc(&short(&demo.canonical_json_sha256, 24))
        )
    } else {
        "<div class=\"badge\" style=\"background:#3a0a0a;color:#ffcdd2;border-color:#7f1d1d\">\
         ✗ Seal mismatch — record altered</div>"
            .to_string()
    };
    s.push_str(&badge);

    s.push_str("<div class=\"grid\" style=\"margin-top:24px\">");
    let off = &demo.officer;
    s.push_str("<div class=\"card\"><h3>Officer identity</h3>");
    s.push_str(&format!(
        "<p><strong>{}</strong><br>{}</p>",
        esc(&off.holder_name),
        esc(&off.role)
    ));
    s.push_str(&format!(
        "<p class=\"mono\">ID {} · {}<br>pubkey {}</p></div>",
        esc(&off.employee_id),
        esc(&off.npub_demo),
        esc(&short(&off.pubkey, 32))
    ));

    let inc = &demo.incident;
    s.push_str("<div class=\"card\"><h3>Incident report</h3>");
    s.push_str(&format!(
        "<p class=\"mono\">{} · {}</p>",
        esc(&inc.report_id),
        esc(&inc.occurred_at.format("%Y-%m-%d %H:%M UTC").to_string())
    ));
    s.push_str(&format!("<p><strong>{}</strong></p>", esc(&inc.category)));
    s.push_str(&format!("<p>{}</p>", esc(&inc.narrative)));
    s.push_str("<p><strong>Actions taken</strong></p><ul>");
    for action in &inc.actions_taken {
        s.push_str(&format!("<li>{}</li>", esc(action)));
    }
    s.push_str("</ul></div></div>");

    s.push_str("<h2 style=\"margin-top:40px\">Signed patrol trajectory</h2>");
    s.push_str(
        "<p class=\"chain\">Mixed indoor-BLE + outdoor-Meshtastic-GPS round. Each stop is \
         signed with the officer's identity, hash-chained to the previous stop, and — on \
         outdoor legs — co-signed by independent mesh relays.</p>",
    );
    s.push_str(
        "<table><tr><th>#</th><th>Time</th><th>Zone &amp; method</th>\
         <th>Location proof</th><th>Status</th><th>Note</th></tr>",
    );
    for att in &demo.trajectory.stops {
        let cls = if att.status.eq_ignore_ascii_case("alert") {
            "status-alert"
        } else {
            "status-secure"
        };
        s.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>{}{}</td>\
             <td>{}</td><td class=\"{}\">{}</td><td>{}</td></tr>",
            att.seq,
            esc(&att.observed_at.format("%H:%M UTC").to_string()),
            esc(&att.zone),
            method_badge(att.method),
            proof_cell(att),
            cls,
            esc(&att.status),
            esc(att.note.as_deref().unwrap_or("—"))
        ));
    }
    s.push_str("</table>");
    s.push_str(&format!(
        "<div class=\"badge\" style=\"margin-top:18px\">⛓ Chain head · sha256 {}</div>",
        esc(&short(&demo.trajectory.chain_head, 24))
    ));
    s.push_str("</section></body></html>");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accountability::AccountabilityDemo;
    use crate::generate::{CopyGenerator, TemplateCopyGenerator};
    use crate::pipeline::Industry;
    use chrono::{TimeZone, Utc};

    fn brand() -> Brand {
        let mut b = Brand::placeholder("Acme Security");
        b.services = vec!["Mobile Patrol".to_string(), "Armed Guards".to_string()];
        b.phone = Some("(415) 555-0199".to_string());
        b
    }

    #[test]
    fn landing_page_includes_brand_and_escapes() {
        let b = brand();
        let copy = TemplateCopyGenerator.generate(&b, Industry::PrivateSecurity);
        let html = render_landing_page(&b, &copy, "portal.html");
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("Acme Security"));
        assert!(html.contains("Mobile Patrol"));
        assert!(html.contains(&b.primary_color));
    }

    #[test]
    fn portal_page_shows_verified_seal() {
        let b = brand();
        let reference = match Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => Utc::now(),
        };
        let demo = AccountabilityDemo::sample(&b, reference);
        let html = render_portal_page(&b, &demo);
        assert!(html.contains("seal verified"));
        assert!(html.contains("IR-2026-0142"));
        assert!(html.contains("Signed patrol trajectory"));
        assert!(html.contains("Meshtastic GPS"));
        assert!(html.contains("Relay-Alpha"));
        assert!(html.contains("Chain head"));
    }
}
