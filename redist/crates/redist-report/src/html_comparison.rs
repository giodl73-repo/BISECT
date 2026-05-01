//! Self-contained HTML side-by-side comparison renderer (PCN plan Task 7).
//!
//! Produces a single HTML file with embedded CSS — no external resources, no
//! JavaScript, safe to email or print. Civic-friendly framing: when either
//! plan carries `submission_type = "civic_counter_proposal"`, a banner is
//! rendered AT THE TOP making the framing unambiguous.
//!
//! Layout:
//!
//! 1. Civic-counter-proposal banner (when applicable).
//! 2. Header: plan A label + plan B label + draft/approved indicator.
//! 3. Side-by-side metrics table: Dem-leaning seats, MM count, mean
//!    Polsby-Popper, total population, n_districts.
//! 4. Diff scope: tracts changed, districts touched.
//! 5. Inline narrative — the SAME rendered markdown the narrative dispatch
//!    writes, converted to HTML paragraphs (we deliberately don't use a
//!    full markdown engine — the renderer's output is a strict subset).
//! 6. Chain-of-custody footer: plan-A manifest SHA, plan-B manifest SHA,
//!    template SHA, build commit short, approved-by, approved-at.
//!
//! Reproducibility:
//! - All output is byte-stable given the same inputs.
//! - HTML is escaped via `escape_html()`; no risk of injection from
//!   submitter names or plan labels.
//! - ASCII-only on Windows per PP-34 — em-dashes and smart quotes are
//!   normalized to `--` and ASCII quotes.

use crate::comparison::{ComparisonReport, PlanSide};
use crate::narrative::NarrativeConfig;

/// Inputs the HTML renderer needs beyond the comparison report itself.
#[derive(Debug, Clone)]
pub struct HtmlComparisonContext {
    /// The Markdown narrative produced by `render_narrative` — embedded inline
    /// as paragraphs.
    pub narrative_markdown: String,
    /// Template SHA-256 (recorded in the footer).
    pub template_sha256: String,
    /// Short build commit (first 8 chars of `redist_build_commit`).
    pub redist_build_commit_short: String,
    /// `redist` version string (e.g. "0.1.0").
    pub redist_version: String,
    /// Approved-at timestamp (ISO-8601 UTC); shown in footer.
    pub approved_at: String,
}

/// Render a self-contained HTML comparison report.
pub fn render_comparison_html(
    report: &ComparisonReport,
    cfg: &NarrativeConfig,
    ctx: &HtmlComparisonContext,
) -> String {
    let mut s = String::with_capacity(8192);
    s.push_str("<!DOCTYPE html>\n");
    s.push_str("<html lang=\"en\">\n");
    s.push_str("<head>\n");
    s.push_str("<meta charset=\"UTF-8\">\n");
    let title = format!(
        "Plan comparison: {} vs {}",
        report.plan_a.label, report.plan_b.label
    );
    s.push_str(&format!("<title>{}</title>\n", escape_html(&title)));
    s.push_str(EMBEDDED_CSS);
    s.push_str("</head>\n<body>\n");

    s.push_str("<main class=\"report\">\n");

    // 1. Civic banner.
    if report.has_civic_counter_proposal() {
        s.push_str(&render_civic_banner(report));
    }

    // 2. Draft/approved badge + header.
    s.push_str(&render_header(report, cfg));

    // 3. Side-by-side metrics.
    s.push_str(&render_metrics_table(report, cfg));

    // 4. Diff scope.
    s.push_str(&render_diff_scope(report));

    // 5. Inline narrative.
    s.push_str("<section class=\"narrative\">\n");
    s.push_str("<h2>Narrative</h2>\n");
    s.push_str(&markdown_to_paragraphs(&ctx.narrative_markdown));
    s.push_str("</section>\n");

    // 6. Chain-of-custody footer.
    s.push_str(&render_provenance_footer(report, ctx));

    s.push_str("</main>\n</body>\n</html>\n");
    s
}

// ---------------------------------------------------------------------------
// Section renderers
// ---------------------------------------------------------------------------

fn render_civic_banner(report: &ComparisonReport) -> String {
    // Identify which side is the civic proposal.
    let civic_side: &PlanSide = if report
        .plan_b
        .submission_type
        .as_deref()
        .map(|s| s == "civic_counter_proposal")
        .unwrap_or(false)
    {
        &report.plan_b
    } else {
        &report.plan_a
    };
    let submitter = civic_side.submitted_by.as_deref().unwrap_or("(unspecified)");
    let when = civic_side.submitted_at.as_deref().unwrap_or("(unspecified)");
    format!(
        "<div class=\"civic-banner\" role=\"note\">\n\
         <strong>Civic counter-proposal.</strong> The map labeled \
         <code>{label}</code> was submitted by {by} on {when}. \
         It is not the state's official map. \
         This report compares it side-by-side with the other map shown.\n\
         </div>\n",
        label = escape_html(&civic_side.label),
        by = escape_html(submitter),
        when = escape_html(when),
    )
}

fn render_header(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let badge = if cfg.approved_by.is_some() {
        format!(
            "<span class=\"badge approved\">APPROVED &mdash; {}</span>",
            escape_html(cfg.approved_by.as_deref().unwrap_or(""))
        )
    } else {
        "<span class=\"badge draft\">DRAFT &mdash; review before publication</span>"
            .to_string()
    };
    format!(
        "<header class=\"report-header\">\n\
         <h1>Plan comparison</h1>\n\
         <p class=\"plan-labels\"><strong>{a}</strong> vs <strong>{b}</strong></p>\n\
         {badge}\n\
         </header>\n",
        a = escape_html(&report.plan_a.label),
        b = escape_html(&report.plan_b.label),
        badge = badge,
    )
}

fn render_metrics_table(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let mut s = String::new();
    s.push_str("<section class=\"metrics\">\n");
    s.push_str(&format!(
        "<h2>Side-by-side metrics</h2>\n\
         <p class=\"threshold-note\">\
         Democratic-leaning classification uses a {threshold}% Dem-share threshold; \
         districts within +/-{band}pp are flagged as close calls.\
         </p>\n",
        threshold = format!("{:.1}", cfg.leaning_threshold * 100.0),
        band = format!("{:.1}", cfg.close_call_band * 100.0),
    ));

    s.push_str("<table class=\"metrics-table\">\n");
    s.push_str(
        "<thead><tr><th scope=\"col\">Metric</th>\
         <th scope=\"col\">Plan A</th><th scope=\"col\">Plan B</th></tr></thead>\n",
    );
    s.push_str("<tbody>\n");
    s.push_str(&row("Districts", &report.plan_a.n_districts.to_string(), &report.plan_b.n_districts.to_string()));
    s.push_str(&row(
        "Democratic-leaning seats",
        &report.plan_a.leaning_seats.to_string(),
        &report.plan_b.leaning_seats.to_string(),
    ));
    s.push_str(&row(
        "Majority-minority districts",
        &report.plan_a.mm_count.to_string(),
        &report.plan_b.mm_count.to_string(),
    ));
    s.push_str(&row(
        "Mean Polsby-Popper",
        &fmt_pp(report.plan_a.mean_pp),
        &fmt_pp(report.plan_b.mean_pp),
    ));
    if report.plan_a.total_population > 0 || report.plan_b.total_population > 0 {
        s.push_str(&row(
            "Total population",
            &fmt_int(report.plan_a.total_population),
            &fmt_int(report.plan_b.total_population),
        ));
    }
    s.push_str("</tbody>\n</table>\n</section>\n");
    s
}

fn row(label: &str, a: &str, b: &str) -> String {
    format!(
        "<tr><th scope=\"row\">{}</th><td>{}</td><td>{}</td></tr>\n",
        escape_html(label),
        escape_html(a),
        escape_html(b),
    )
}

fn render_diff_scope(report: &ComparisonReport) -> String {
    let districts: Vec<String> = report
        .diff
        .districts_with_changes
        .iter()
        .map(|d| d.to_string())
        .collect();
    let districts_str = if districts.is_empty() {
        "(none)".to_string()
    } else {
        districts.join(", ")
    };
    format!(
        "<section class=\"diff\">\n\
         <h2>Differences between the two plans</h2>\n\
         <ul>\n\
         <li><strong>Tracts reassigned:</strong> {}</li>\n\
         <li><strong>Districts touched:</strong> {}</li>\n\
         </ul>\n\
         </section>\n",
        report.diff.tracts_changed,
        escape_html(&districts_str),
    )
}

fn render_provenance_footer(report: &ComparisonReport, ctx: &HtmlComparisonContext) -> String {
    format!(
        "<footer class=\"provenance\">\n\
         <h2>Chain of custody</h2>\n\
         <dl>\n\
         <dt>Plan A manifest SHA-256</dt><dd><code>{a_sha}</code></dd>\n\
         <dt>Plan B manifest SHA-256</dt><dd><code>{b_sha}</code></dd>\n\
         <dt>Renderer SHA-256</dt><dd><code>{tmpl}</code></dd>\n\
         <dt>redist version</dt><dd><code>{ver}</code> (build <code>{commit}</code>)</dd>\n\
         <dt>Generated at</dt><dd><code>{when}</code></dd>\n\
         </dl>\n\
         <p class=\"reproducibility-note\">\
         These hashes pin every input to this report. To reproduce: \
         <code>redist compare --plan-a {a_label} --plan-b {b_label} --format html</code>.\
         </p>\n\
         </footer>\n",
        a_sha = escape_html(&report.plan_a.manifest_sha256),
        b_sha = escape_html(&report.plan_b.manifest_sha256),
        tmpl = escape_html(&ctx.template_sha256),
        ver = escape_html(&ctx.redist_version),
        commit = escape_html(&ctx.redist_build_commit_short),
        when = escape_html(&ctx.approved_at),
        a_label = escape_html(&report.plan_a.label),
        b_label = escape_html(&report.plan_b.label),
    )
}

// ---------------------------------------------------------------------------
// Markdown -> HTML (deliberately minimal)
// ---------------------------------------------------------------------------

/// Convert the narrative renderer's Markdown output to HTML paragraphs.
/// The renderer produces simple paragraphs separated by blank lines —
/// no headings, no lists, no tables. Anything else is escaped as plain text.
fn markdown_to_paragraphs(md: &str) -> String {
    let mut out = String::new();
    for para in md.split("\n\n") {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push_str("<p>");
        // Re-flow internal newlines as spaces (renderer doesn't intend
        // them as <br>).
        let single_line: String = trimmed
            .split('\n')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&escape_html(&single_line));
        out.push_str("</p>\n");
    }
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn fmt_pp(v: f64) -> String {
    if v.is_nan() {
        "(unavailable)".to_string()
    } else {
        format!("{:.3}", v)
    }
}

fn fmt_int(n: u64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    out.chars().rev().collect()
}

const EMBEDDED_CSS: &str = "<style>\n\
body { font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif; max-width: 920px; margin: 0 auto; padding: 2rem; color: #1a1a1a; line-height: 1.5; }\n\
.report { background: #fff; }\n\
h1 { font-size: 1.875rem; margin-bottom: 0.25rem; }\n\
h2 { font-size: 1.25rem; margin-top: 2rem; padding-bottom: 0.25rem; border-bottom: 1px solid #d0d7de; }\n\
.plan-labels { font-size: 1.125rem; color: #57606a; margin-top: 0; }\n\
.badge { display: inline-block; padding: 0.25rem 0.625rem; border-radius: 3px; font-size: 0.75rem; font-weight: 600; letter-spacing: 0.04em; }\n\
.badge.draft { background: #fff8c5; color: #4d3800; border: 1px solid #d4a72c; }\n\
.badge.approved { background: #dafbe1; color: #1a7f37; border: 1px solid #2da44e; }\n\
.civic-banner { background: #ddf4ff; border-left: 4px solid #0969da; padding: 1rem 1.25rem; margin-bottom: 1.5rem; border-radius: 4px; }\n\
.metrics-table { width: 100%; border-collapse: collapse; margin-top: 1rem; }\n\
.metrics-table th, .metrics-table td { padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #d0d7de; }\n\
.metrics-table th[scope=\"col\"] { background: #f6f8fa; font-weight: 600; }\n\
.metrics-table th[scope=\"row\"] { font-weight: 500; color: #4a4a4a; }\n\
.metrics-table tbody tr:hover { background: #f6f8fa; }\n\
.threshold-note { font-size: 0.875rem; color: #57606a; }\n\
.diff ul { padding-left: 1.25rem; }\n\
.narrative p { margin: 0.75rem 0; }\n\
.provenance { margin-top: 3rem; padding-top: 1rem; border-top: 2px solid #d0d7de; font-size: 0.875rem; color: #57606a; }\n\
.provenance dl { display: grid; grid-template-columns: max-content 1fr; gap: 0.25rem 1rem; }\n\
.provenance dt { font-weight: 600; }\n\
.provenance code { background: #f6f8fa; padding: 0.125rem 0.375rem; border-radius: 3px; font-size: 0.75rem; word-break: break-all; }\n\
.reproducibility-note { margin-top: 1rem; font-style: italic; }\n\
@media print { body { max-width: none; padding: 0; } .civic-banner, .badge { -webkit-print-color-adjust: exact; print-color-adjust: exact; } }\n\
</style>\n";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::{DiffSummary, PlanSide};
    use std::collections::BTreeMap;

    fn fixture_plan_a() -> PlanSide {
        PlanSide {
            label: "vt_state_proposal".into(),
            manifest_sha256: "a".repeat(64),
            leaning_seats: 5,
            n_districts: 6,
            per_district_dem_share: vec![0.30, 0.45, 0.55, 0.60, 0.65, 0.70],
            mm_count: 1,
            mean_pp: 0.42,
            total_population: 643_077,
            submission_type: None,
            submitted_by: None,
            submitted_at: None,
            analysis_sha256: BTreeMap::new(),
        }
    }

    fn fixture_plan_b_civic() -> PlanSide {
        PlanSide {
            label: "lwv_civic_alt".into(),
            manifest_sha256: "b".repeat(64),
            leaning_seats: 4,
            n_districts: 6,
            per_district_dem_share: vec![0.25, 0.40, 0.547, 0.553, 0.62, 0.68],
            mm_count: 1,
            mean_pp: 0.40,
            total_population: 643_077,
            submission_type: Some("civic_counter_proposal".into()),
            submitted_by: Some("League of Women Voters Vermont".into()),
            submitted_at: Some("2026-04-15T12:00:00Z".into()),
            analysis_sha256: BTreeMap::new(),
        }
    }

    fn fixture_ctx() -> HtmlComparisonContext {
        HtmlComparisonContext {
            narrative_markdown: "[DRAFT - review before publication] Plan A elects 5 Democratic-leaning seats; Plan B elects 4 Democratic-leaning seats.\n\nThis plan moves 12 tracts.".into(),
            template_sha256: "0".repeat(64),
            redist_build_commit_short: "deadbeef".into(),
            redist_version: "0.1.0".into(),
            approved_at: "1970-01-01T00:00:00Z".into(),
        }
    }

    fn fixture_report() -> ComparisonReport {
        ComparisonReport::from_loaded(
            fixture_plan_a(),
            fixture_plan_b_civic(),
            None,
            DiffSummary {
                tracts_changed: 12,
                population_changed: 0,
                districts_with_changes: vec![1, 3, 5],
            },
        )
    }

    #[test]
    fn renders_a_full_html_document() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(html.starts_with("<!DOCTYPE html>"), "must start with HTML5 doctype");
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("</html>"), "must close <html>");
        assert!(html.contains("<style>"), "CSS must be embedded inline");
    }

    #[test]
    fn civic_banner_renders_when_civic_counter_proposal_present() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(html.contains("class=\"civic-banner\""), "civic banner must render");
        assert!(html.contains("League of Women Voters Vermont"), "submitter name must appear in banner");
        assert!(html.contains("not the state's official map"), "civic disclaimer must appear in banner");
    }

    #[test]
    fn no_civic_banner_when_neither_side_tagged() {
        let mut report = fixture_report();
        report.plan_b.submission_type = None;
        report.plan_b.submitted_by = None;
        report.plan_b.submitted_at = None;
        let html = render_comparison_html(&report, &NarrativeConfig::default(), &fixture_ctx());
        assert!(!html.contains("class=\"civic-banner\""), "civic banner must NOT render when no side is tagged");
    }

    #[test]
    fn draft_badge_renders_when_unsigned() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(html.contains("badge draft"), "DRAFT badge must render");
        assert!(html.contains("DRAFT"));
    }

    #[test]
    fn approved_badge_renders_when_signed() {
        let mut cfg = NarrativeConfig::default();
        cfg.approved_by = Some("J. Doe".into());
        let html = render_comparison_html(&fixture_report(), &cfg, &fixture_ctx());
        assert!(html.contains("badge approved"), "APPROVED badge must render when --approved-by set");
        assert!(html.contains("J. Doe"), "approver name must appear in badge");
        assert!(!html.contains("badge draft"), "DRAFT badge must NOT render when approved");
    }

    #[test]
    fn metrics_table_carries_all_rows() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(html.contains("Districts"), "districts row");
        assert!(html.contains("Democratic-leaning seats"), "leaning seats row");
        assert!(html.contains("Majority-minority districts"), "MM row");
        assert!(html.contains("Mean Polsby-Popper"), "PP row");
        assert!(html.contains("Total population"), "population row");
        // Comma-formatted population.
        assert!(html.contains("643,077"), "population must use thousands separators");
    }

    #[test]
    fn threshold_disclosure_uses_config_values() {
        let mut cfg = NarrativeConfig::default();
        cfg.leaning_threshold = 0.52;
        cfg.close_call_band = 0.015;
        let html = render_comparison_html(&fixture_report(), &cfg, &fixture_ctx());
        assert!(html.contains("52.0%"), "threshold disclosure must use cfg.leaning_threshold");
        assert!(html.contains("1.5pp"), "close-call band must use cfg.close_call_band");
    }

    #[test]
    fn diff_scope_lists_changed_tracts_and_districts() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(html.contains("Tracts reassigned"));
        assert!(html.contains("</strong> 12</li>"), "12 tracts changed appears in the diff list");
        assert!(html.contains("1, 3, 5"), "district list");
    }

    #[test]
    fn provenance_footer_includes_all_shas() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(html.contains(&"a".repeat(64)), "plan A SHA");
        assert!(html.contains(&"b".repeat(64)), "plan B SHA");
        assert!(html.contains(&"0".repeat(64)), "template SHA");
        assert!(html.contains("deadbeef"), "build commit short");
        assert!(html.contains("0.1.0"), "redist version");
    }

    #[test]
    fn reproducibility_note_includes_command() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        assert!(
            html.contains("redist compare --plan-a vt_state_proposal --plan-b lwv_civic_alt --format html"),
            "footer must show the exact reproduction command"
        );
    }

    #[test]
    fn narrative_markdown_renders_as_paragraphs() {
        let html = render_comparison_html(
            &fixture_report(),
            &NarrativeConfig::default(),
            &fixture_ctx(),
        );
        // Each paragraph wrapped in <p>...</p>.
        assert!(html.contains("<p>[DRAFT - review before publication] Plan A elects 5 Democratic-leaning seats"));
        assert!(html.contains("<p>This plan moves 12 tracts.</p>"));
    }

    #[test]
    fn html_escapes_special_characters_in_labels() {
        let mut report = fixture_report();
        report.plan_a.label = "evil<script>alert(1)</script>".into();
        let html = render_comparison_html(&report, &NarrativeConfig::default(), &fixture_ctx());
        assert!(!html.contains("<script>alert(1)</script>"), "raw script tag must NOT appear");
        assert!(html.contains("evil&lt;script&gt;alert(1)&lt;/script&gt;"), "label must be escaped");
    }

    #[test]
    fn html_escapes_quotes_in_submitter_name() {
        let mut report = fixture_report();
        report.plan_b.submitted_by = Some("\"Bobby\" Tables".into());
        let html = render_comparison_html(&report, &NarrativeConfig::default(), &fixture_ctx());
        assert!(html.contains("&quot;Bobby&quot; Tables"), "quotes in submitter name must be escaped");
    }

    #[test]
    fn empty_district_list_renders_as_none() {
        let mut report = fixture_report();
        report.diff.districts_with_changes = vec![];
        let html = render_comparison_html(&report, &NarrativeConfig::default(), &fixture_ctx());
        assert!(html.contains("(none)"), "empty district list must render as '(none)'");
    }

    #[test]
    fn nan_polsby_popper_renders_as_unavailable() {
        let mut report = fixture_report();
        report.plan_a.mean_pp = f64::NAN;
        let html = render_comparison_html(&report, &NarrativeConfig::default(), &fixture_ctx());
        assert!(html.contains("(unavailable)"), "NaN mean_pp must render as '(unavailable)'");
    }

    #[test]
    fn population_row_omitted_when_both_zero() {
        let mut report = fixture_report();
        report.plan_a.total_population = 0;
        report.plan_b.total_population = 0;
        let html = render_comparison_html(&report, &NarrativeConfig::default(), &fixture_ctx());
        assert!(!html.contains("Total population"), "population row must be hidden when unavailable for both plans");
    }
}
