/// html.rs — tera-based HTML rendering for commission reports.
///
/// Spec 6 requirements:
/// - Self-contained HTML: no CDN links, no external HTTP resources
/// - Images embedded as data URIs (base64 PNG)
/// - External analyzer disclaimer when external_analyzers present
/// - All 10 section headers present in output
use crate::report::Report;

/// Embedded HTML template (compile-time include so binary is self-contained).
const REPORT_TEMPLATE: &str = include_str!("../templates/report.html.j2");

/// Render a Report as a self-contained HTML string.
///
/// Board amendments:
/// - No <link href="http..."> or <script src="http...">
/// - Images embedded as data URIs (base64)
/// - External analyzer disclaimer present when has_external_analyzers=true
pub fn render_html_report(report: &Report) -> anyhow::Result<String> {
    let mut tera = tera::Tera::default();
    tera.add_raw_template("report.html", REPORT_TEMPLATE)
        .map_err(|e| anyhow::anyhow!("Tera template error: {e}"))?;

    let mut ctx = tera::Context::new();

    // Top-level fields
    ctx.insert("label", &report.label);
    ctx.insert("state", &report.state);
    ctx.insert("year", &report.year);
    ctx.insert("generated_at", &report.generated_at);
    ctx.insert("has_external_analyzers", &report.has_external_analyzers);

    // Sections
    let s = &report.sections;
    ctx.insert("exec_summary", &s.executive_summary);
    ctx.insert("population_equality", &s.population_equality);
    ctx.insert("geographic_constraints", &s.geographic_constraints);
    ctx.insert("partisan_fairness", &s.partisan_fairness);
    ctx.insert("vra_compliance", &s.vra_compliance);
    ctx.insert("compactness", &s.compactness);
    ctx.insert("comparison", &s.comparison);
    ctx.insert("nesting", &s.nesting);
    ctx.insert("audit", &s.audit);
    ctx.insert("maps", &s.maps);

    let html = tera
        .render("report.html", &ctx)
        .map_err(|e| anyhow::anyhow!("Tera render error: {e:#?}"))?;

    Ok(html)
}

// ---------------------------------------------------------------------------
// Tests — Task 5
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{assemble_report, ReportContext, REQUIRED_ANALYSIS_FILES};
    use crate::manifest::PlanManifest;
    use tempfile::TempDir;

    fn make_test_manifest(label: &str) -> PlanManifest {
        PlanManifest {
            label: label.to_string(),
            state_code: "VT".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 1,
            population_source: "total".into(),
            partition_mode: "edge-weighted".into(),
            seed: Some(42),
            binary_version: "0.1.0".into(),
            binary_sha256: "a".repeat(64),
            binary_download_url:
                "https://github.com/owner/redist/releases/download/v0.1.0/redist".into(),
            adjacency_file: "vt_adjacency_2020.adj.bin".into(),
            adjacency_sha256: "b".repeat(64),
            adjacency_build_command: "python scripts/data/generate_adj_bin.py".into(),
            adjacency_build_version: "0.1.0".into(),
            tiger_source_url:
                "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip".into(),
            tiger_sha256: Some("c".repeat(64)),
            created_at: "2026-04-26T00:00:00Z".into(),
            balance_tolerance_pct: 0.5,
            population_balance_valid: true,
            seats_per_district: 1,
            total_seats: 1,
            electoral_system: "single_member".into(),
            gpmetis_version: String::new(),
            ..Default::default()
        }
    }

    fn setup_full_plan_dir(tmp: &TempDir, label: &str) -> ReportContext {
        let plan_dir = tmp.path().join("plans").join(label);
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        for name in REQUIRED_ANALYSIS_FILES {
            std::fs::write(
                analysis_dir.join(name),
                serde_json::to_string(&serde_json::json!({"status": "ok", "file": name}))
                    .unwrap(),
            )
            .unwrap();
        }
        ReportContext::new(plan_dir, make_test_manifest(label))
    }

    #[test]
    fn test_html_report_contains_all_section_headers() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_full_plan_dir(&tmp, "vt_html_test");
        let report = assemble_report(&ctx).unwrap();
        let html = render_html_report(&report).unwrap();
        let headers = [
            "Executive Summary",
            "Population Equality",
            "Geographic Constraints",
            "Partisan Fairness",
            "Minority Representation",
            "Compactness",
            "Comparison vs Enacted Plan",
            "Nesting Compliance",
            "Audit Trail",
            "Maps",
        ];
        for header in &headers {
            assert!(
                html.contains(header),
                "HTML report missing section header: {header}"
            );
        }
    }

    #[test]
    fn test_html_is_self_contained_no_external_links() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_full_plan_dir(&tmp, "vt_selfcontained_test");
        let report = assemble_report(&ctx).unwrap();
        let html = render_html_report(&report).unwrap();
        assert!(
            !html.contains("http://"),
            "HTML must not reference external HTTP resources"
        );
        assert!(
            !html.contains("https://cdn"),
            "HTML must not load from CDN"
        );
        // If images present, must be data URIs
        if html.contains("<img") {
            assert!(
                html.contains("data:image/"),
                "images must be data URIs"
            );
        }
    }

    #[test]
    fn test_html_report_has_html_tag() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_full_plan_dir(&tmp, "vt_html_tag_test");
        let report = assemble_report(&ctx).unwrap();
        let html = render_html_report(&report).unwrap();
        assert!(
            html.to_lowercase().contains("<html"),
            "HTML output must contain <html tag"
        );
    }

    // ── Task 210: HTML report mobile overflow fix ────────────────────────────

    #[test]
    fn test_html_report_pre_has_overflow_auto() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_full_plan_dir(&tmp, "vt_overflow_test");
        let report = assemble_report(&ctx).unwrap();
        let html = render_html_report(&report).unwrap();
        assert!(
            html.contains("overflow-x: auto") || html.contains("overflow-x:auto"),
            "HTML report CSS must include overflow-x: auto for mobile scrolling"
        );
    }

    #[test]
    fn test_external_analyzer_disclaimer_in_html() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_full_plan_dir(&tmp, "vt_ext_analyzer_test");
        let report = assemble_report(&ctx).unwrap();
        // Manually set has_external_analyzers to simulate the flag
        let mut report = report;
        report.has_external_analyzers = true;
        let html = render_html_report(&report).unwrap();
        assert!(
            html.contains("external tools") || html.contains("External Analyzer"),
            "HTML must include external analyzer disclaimer when has_external_analyzers=true"
        );
        assert!(
            html.contains("reproducibility") || html.contains("independent reproducibility"),
            "HTML must note that external analyzers affect reproducibility"
        );
    }
}
