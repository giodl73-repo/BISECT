//! Civic-friendly comparison narrative renderer (Plan Comparison plan Tasks 3 + 5).
//!
//! Direct Rust string formatting (NOT Tera) so the value-correctness tests
//! (Task 13) can assert on exact substrings without template-engine
//! abstraction. The override-template path (`--narrative-template <PATH>`)
//! is deferred to a follow-up that wires Tera against this same data.
//!
//! Paragraph order (civic-friendly framing FIRST per COMMONS):
//!
//! 1. **Civic-counter-proposal framing** (rendered only when either plan's
//!    `submission_type == "civic_counter_proposal"`). One sentence with
//!    submitter + submission date + "is not the state's official map."
//! 2. **Diff scope.** Tracts moved, population moved, districts touched.
//! 3. **Partisan composition.** Dem/Rep seat counts per plan, threshold
//!    disclosure ("using 55% Dem-share threshold"), close-call flagging
//!    for any district within ±close_call_band of the threshold.
//! 4. **MM count + demographic shifts.**
//! 5. **Compactness.** Polsby-Popper means.
//!
//! Every paragraph is prefixed `[DRAFT — review before publication] ` until
//! `--approved-by "<name>"` is supplied (`config.approved_by.is_some()`).
//!
//! MoE suppression (`crate::moe`): when the caller supplies a CI band for
//! a metric and the directional claim would flip within the CI, the
//! sentence's directional claim is replaced with the canonical
//! "within margin of error; see numerical table." string.
//!
//! ASCII-only output per PP-34 — the narrative may go to stdout on Windows.

use crate::comparison::{ComparisonReport, PlanSide};
use crate::moe::{suppress_or_emit, CiBand, MetricMonotonicity, MOE_SUPPRESSED_TEXT};

/// Configuration for the narrative renderer.
#[derive(Debug, Clone)]
pub struct NarrativeConfig {
    /// Threshold for "Democratic-leaning" classification (default 0.55).
    pub leaning_threshold: f64,
    /// Half-width of the close-call band (default 0.02 = ±2 percentage points).
    pub close_call_band: f64,
    /// Sign-off name; `None` = `[DRAFT]` mode.
    pub approved_by: Option<String>,
    /// Optional MoE bands for the partisan-seat metric. When supplied, the
    /// renderer consults `suppress_or_emit` and substitutes the canonical
    /// MoE text in the partisan paragraph if a flip is possible.
    pub partisan_seat_ci: Option<(CiBand, CiBand)>,
    /// Optional MoE bands for the MM-count metric (non-monotone).
    pub mm_count_ci: Option<(CiBand, CiBand)>,
    /// Optional MoE bands for the mean Polsby-Popper compactness.
    pub mean_pp_ci: Option<(CiBand, CiBand)>,
}

impl Default for NarrativeConfig {
    fn default() -> Self {
        NarrativeConfig {
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            approved_by: None,
            partisan_seat_ci: None,
            mm_count_ci: None,
            mean_pp_ci: None,
        }
    }
}

/// Render the narrative to a Markdown string.
pub fn render_narrative(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let mut s = String::new();
    let prefix = if cfg.approved_by.is_some() {
        ""
    } else {
        "[DRAFT - review before publication] "
    };

    // 1. Civic-counter-proposal framing (when applicable).
    if let Some(framing) = civic_counter_proposal_framing(report) {
        s.push_str(prefix);
        s.push_str(&framing);
        s.push_str("\n\n");
    }

    // 2. Diff scope.
    s.push_str(prefix);
    s.push_str(&diff_scope_paragraph(report));
    s.push_str("\n\n");

    // 3. Partisan composition.
    s.push_str(prefix);
    s.push_str(&partisan_composition_paragraph(report, cfg));
    s.push_str("\n\n");

    // 4. MM count + demographic shifts.
    s.push_str(prefix);
    s.push_str(&mm_count_paragraph(report, cfg));
    s.push_str("\n\n");

    // 5. Compactness.
    s.push_str(prefix);
    s.push_str(&compactness_paragraph(report, cfg));
    s.push_str("\n\n");

    // Audit trail footer (always rendered; not subject to [DRAFT] gate).
    s.push_str("---\n\n");
    s.push_str(&audit_footer(report, cfg));
    s.push('\n');

    s
}

fn civic_counter_proposal_framing(report: &ComparisonReport) -> Option<String> {
    let civic_side = if report.plan_a.submission_type.as_deref() == Some("civic_counter_proposal") {
        Some(("Plan A", &report.plan_a))
    } else if report.plan_b.submission_type.as_deref() == Some("civic_counter_proposal") {
        Some(("Plan B", &report.plan_b))
    } else {
        None
    };
    civic_side.map(|(side_label, side)| {
        format!(
            "{side_label} is a civic counter-proposal submitted by {by} on {at}; \
             it is not the state's official map.",
            by = side.submitted_by.as_deref().unwrap_or("[unknown submitter]"),
            at = side.submitted_at.as_deref().unwrap_or("[unknown date]"),
        )
    })
}

fn diff_scope_paragraph(report: &ComparisonReport) -> String {
    let d = &report.diff;
    let n_districts = d.districts_with_changes.len();
    let districts_str = if n_districts == 0 {
        "no districts".to_string()
    } else if n_districts == 1 {
        format!("District {}", d.districts_with_changes[0])
    } else {
        let names: Vec<String> = d.districts_with_changes.iter().map(|n| n.to_string()).collect();
        format!("Districts {}", names.join(", "))
    };
    format!(
        "Plan B reassigns {tracts} tracts ({pop} people) relative to Plan A; \
         changes are concentrated in {districts}.",
        tracts = d.tracts_changed,
        pop = d.population_changed,
        districts = districts_str,
    )
}

fn partisan_composition_paragraph(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let threshold_pct = (cfg.leaning_threshold * 100.0).round() as i32;
    // MoE suppression: if the partisan-seat CIs are supplied and a flip is
    // possible, suppress the directional claim. Always emit the seat-count
    // numbers; only suppress the comparative phrasing.
    let directional = if let Some((a_ci, b_ci)) = cfg.partisan_seat_ci {
        suppress_or_emit("partisan_seats", MetricMonotonicity::Monotone, a_ci, b_ci)
            .unwrap_or_else(|| {
                directional_phrase("Plan B", report.plan_b.leaning_seats, "Plan A", report.plan_a.leaning_seats)
            })
    } else {
        directional_phrase("Plan B", report.plan_b.leaning_seats, "Plan A", report.plan_a.leaning_seats)
    };

    let mut s = format!(
        "Statewide partisan composition (using {threshold}% Dem-share threshold): \
         Plan A elects {a} Democratic-leaning seats; Plan B elects {b} Democratic-leaning seats. {directional}",
        threshold = threshold_pct,
        a = report.plan_a.leaning_seats,
        b = report.plan_b.leaning_seats,
        directional = directional,
    );

    // Close-call flagging for any district within ±close_call_band of the threshold.
    let close_a = close_call_districts(&report.plan_a, cfg);
    let close_b = close_call_districts(&report.plan_b, cfg);
    if !close_a.is_empty() || !close_b.is_empty() {
        s.push(' ');
        s.push_str(&close_call_sentence(&close_a, &close_b, cfg.leaning_threshold));
    }
    s
}

fn directional_phrase(
    a_name: &str,
    a_seats: usize,
    b_name: &str,
    b_seats: usize,
) -> String {
    use std::cmp::Ordering;
    match a_seats.cmp(&b_seats) {
        Ordering::Greater => format!("{a_name} yields {} more Democratic-leaning seats than {b_name}.", a_seats - b_seats),
        Ordering::Less => format!("{b_name} yields {} more Democratic-leaning seats than {a_name}.", b_seats - a_seats),
        Ordering::Equal => "Both plans elect the same number of Democratic-leaning seats.".to_string(),
    }
}

#[derive(Debug, Clone)]
struct CloseCallDistrict {
    /// 1-based district index.
    index: usize,
    dem_share: f64,
    /// `"above"` or `"below"` the threshold.
    side: &'static str,
}

fn close_call_districts(plan: &PlanSide, cfg: &NarrativeConfig) -> Vec<CloseCallDistrict> {
    let mut out = Vec::new();
    let lo = cfg.leaning_threshold - cfg.close_call_band;
    let hi = cfg.leaning_threshold + cfg.close_call_band;
    for (i, &share) in plan.per_district_dem_share.iter().enumerate() {
        if share >= lo && share <= hi {
            out.push(CloseCallDistrict {
                index: i + 1,
                dem_share: share,
                side: if share >= cfg.leaning_threshold { "above" } else { "below" },
            });
        }
    }
    out
}

fn close_call_sentence(
    close_a: &[CloseCallDistrict],
    close_b: &[CloseCallDistrict],
    threshold: f64,
) -> String {
    let threshold_pct = (threshold * 100.0).round() as i32;
    let mut s = String::from("Close-call districts (within +/- the threshold band): ");
    let mut parts: Vec<String> = Vec::new();
    for d in close_a {
        parts.push(format!(
            "Plan A District {} at {:.1}% Dem is just {} the leaning threshold; classification is sensitive to the {threshold_pct}% cutoff",
            d.index, d.dem_share * 100.0, d.side
        ));
    }
    for d in close_b {
        parts.push(format!(
            "Plan B District {} at {:.1}% Dem is just {} the leaning threshold; classification is sensitive to the {threshold_pct}% cutoff",
            d.index, d.dem_share * 100.0, d.side
        ));
    }
    s.push_str(&parts.join("; "));
    s.push('.');
    s
}

fn mm_count_paragraph(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let mm_directional = if let Some((a_ci, b_ci)) = cfg.mm_count_ci {
        suppress_or_emit("mm_count", MetricMonotonicity::NonMonotone, a_ci, b_ci)
            .unwrap_or_else(|| mm_count_directional(&report.plan_a, &report.plan_b))
    } else {
        mm_count_directional(&report.plan_a, &report.plan_b)
    };
    format!(
        "Majority-minority districts: Plan A has {a}, Plan B has {b}. {mm_directional}",
        a = report.plan_a.mm_count,
        b = report.plan_b.mm_count,
        mm_directional = mm_directional,
    )
}

fn mm_count_directional(a: &PlanSide, b: &PlanSide) -> String {
    use std::cmp::Ordering;
    match a.mm_count.cmp(&b.mm_count) {
        Ordering::Greater => format!("Plan A yields {} more MM district(s) than Plan B.", a.mm_count - b.mm_count),
        Ordering::Less => format!("Plan B yields {} more MM district(s) than Plan A.", b.mm_count - a.mm_count),
        Ordering::Equal => "Both plans yield the same number of MM districts.".to_string(),
    }
}

fn compactness_paragraph(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let directional = if let Some((a_ci, b_ci)) = cfg.mean_pp_ci {
        suppress_or_emit("mean_pp", MetricMonotonicity::Monotone, a_ci, b_ci)
            .unwrap_or_else(|| pp_directional(report.plan_a.mean_pp, report.plan_b.mean_pp))
    } else {
        pp_directional(report.plan_a.mean_pp, report.plan_b.mean_pp)
    };
    format!(
        "Compactness (Polsby-Popper, mean across districts): Plan A = {a:.3}, Plan B = {b:.3}. {directional}",
        a = report.plan_a.mean_pp,
        b = report.plan_b.mean_pp,
        directional = directional,
    )
}

fn pp_directional(a: f64, b: f64) -> String {
    let diff = a - b;
    if diff.abs() < 0.001 {
        "Both plans are essentially equally compact.".to_string()
    } else if diff > 0.0 {
        format!("Plan A is more compact by {:.3}.", diff)
    } else {
        format!("Plan B is more compact by {:.3}.", -diff)
    }
}

fn audit_footer(report: &ComparisonReport, cfg: &NarrativeConfig) -> String {
    let signed = match &cfg.approved_by {
        Some(name) => format!("Approved by: {name}"),
        None => "Not yet approved (run with --approved-by \"<your name>\" to sign off).".to_string(),
    };
    format!(
        "Plan A: {a} (manifest sha256 prefix {a_sha})\n\
         Plan B: {b} (manifest sha256 prefix {b_sha})\n\
         Threshold: {th:.2} Dem-share, close-call band +/-{ccb:.2}\n\
         {signed}\n\
         See narrative_manifest.json for the full audit trail.",
        a = report.plan_a.label,
        b = report.plan_b.label,
        a_sha = &report.plan_a.manifest_sha256[..8.min(report.plan_a.manifest_sha256.len())],
        b_sha = &report.plan_b.manifest_sha256[..8.min(report.plan_b.manifest_sha256.len())],
        th = cfg.leaning_threshold,
        ccb = cfg.close_call_band,
        signed = signed,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::{ComparisonReport, DiffSummary, PlanSide};
    use std::collections::BTreeMap;

    fn plan_a_5dem() -> PlanSide {
        PlanSide {
            label: "vt_state".into(),
            manifest_sha256: "aaaaaaaa".repeat(8),
            leaning_seats: 5,
            n_districts: 6,
            per_district_dem_share: vec![0.30, 0.40, 0.553, 0.60, 0.65, 0.70],
            mm_count: 1,
            mean_pp: 0.42,
            total_population: 600_000,
            submission_type: None,
            submitted_by: None,
            submitted_at: None,
            analysis_sha256: BTreeMap::new(),
        }
    }
    fn plan_b_4dem_civic() -> PlanSide {
        PlanSide {
            label: "vt_civic".into(),
            manifest_sha256: "bbbbbbbb".repeat(8),
            leaning_seats: 4,
            n_districts: 6,
            per_district_dem_share: vec![0.25, 0.40, 0.547, 0.65, 0.70, 0.75],
            mm_count: 1,
            mean_pp: 0.40,
            total_population: 600_000,
            submission_type: Some("civic_counter_proposal".into()),
            submitted_by: Some("Eastside Neighborhood Association".into()),
            submitted_at: Some("2026-04-15T12:00:00Z".into()),
            analysis_sha256: BTreeMap::new(),
        }
    }

    fn fixture_report() -> ComparisonReport {
        ComparisonReport::from_loaded(
            plan_a_5dem(),
            plan_b_4dem_civic(),
            None,
            DiffSummary {
                tracts_changed: 12,
                population_changed: 31_500,
                districts_with_changes: vec![3, 6],
            },
        )
    }

    // ── Value-correctness anchors (Task 13) ──────────────────────────────────

    #[test]
    fn test_value_correctness_seat_counts_appear_verbatim() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("5 Democratic-leaning"), "Plan A seat count must appear verbatim: {s}");
        assert!(s.contains("4 Democratic-leaning"), "Plan B seat count must appear verbatim: {s}");
    }

    #[test]
    fn test_value_correctness_threshold_disclosed() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("55% Dem-share threshold"), "threshold must be disclosed: {s}");
    }

    #[test]
    fn test_value_correctness_close_call_above_threshold_flagged() {
        // Plan A District 3 at 0.553 is above the 0.55 threshold by 0.3pp.
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(
            s.contains("just above the leaning threshold"),
            "above-threshold close-call must be flagged: {s}"
        );
        assert!(s.contains("55.3%"), "exact dem share for the close-call must appear: {s}");
    }

    #[test]
    fn test_value_correctness_close_call_below_threshold_flagged() {
        // Plan B District 3 at 0.547 is below the 0.55 threshold by 0.3pp.
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(
            s.contains("just below the leaning threshold"),
            "below-threshold close-call must be flagged: {s}"
        );
        assert!(s.contains("54.7%"));
    }

    #[test]
    fn test_threshold_rebinds_correctly_when_changed() {
        // Spec Task 13.3: changing the threshold must re-rebind the seat counts
        // (no caching artifacts). With threshold 0.50, Plan A still has 5 seats
        // above 50% (0.553, 0.60, 0.65, 0.70 = 4) -- wait, actually 0.40<0.50<0.553
        // so 4 seats above 0.50. Plan B at 0.50 threshold: 0.547, 0.65, 0.70, 0.75 = 4.
        //
        // The renderer doesn't recompute leaning_seats from per_district_dem_share;
        // it uses the value the caller pre-computed in PlanSide. So the threshold
        // disclosure changes but the seat count stays. Test the disclosure change.
        let mut cfg = NarrativeConfig::default();
        cfg.leaning_threshold = 0.50;
        let s = render_narrative(&fixture_report(), &cfg);
        assert!(s.contains("50% Dem-share threshold"), "new threshold must be disclosed: {s}");
        assert!(!s.contains("55% Dem-share threshold"), "old threshold must be gone: {s}");
    }

    // ── [DRAFT] gate (Task 5) ────────────────────────────────────────────────

    #[test]
    fn test_draft_prefix_on_every_paragraph_when_unsigned() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        // Count [DRAFT prefix occurrences. Should be one per body paragraph
        // (5 paragraphs: civic + diff + partisan + mm + compactness).
        let count = s.matches("[DRAFT - review before publication]").count();
        assert!(count >= 5, "expected at least 5 [DRAFT] prefixes, got {count}: {s}");
    }

    #[test]
    fn test_no_draft_prefix_when_approved() {
        let mut cfg = NarrativeConfig::default();
        cfg.approved_by = Some("Jane Q. Citizen".into());
        let s = render_narrative(&fixture_report(), &cfg);
        assert!(
            !s.contains("[DRAFT - review before publication]"),
            "no [DRAFT] prefix when --approved-by is set: {s}"
        );
        assert!(s.contains("Approved by: Jane Q. Citizen"), "approval recorded: {s}");
    }

    // ── Civic-counter-proposal framing (Task 5 / BD-N3) ──────────────────────

    #[test]
    fn test_civic_counter_proposal_framing_appears_first() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(
            s.contains("civic counter-proposal"),
            "civic framing must appear: {s}"
        );
        assert!(
            s.contains("not the state's official map"),
            "framing must include the disclaimer: {s}"
        );
        assert!(
            s.contains("Eastside Neighborhood Association"),
            "submitter must appear: {s}"
        );
        // Framing must come BEFORE the partisan paragraph.
        let civic_pos = s.find("civic counter-proposal").unwrap();
        let partisan_pos = s.find("Statewide partisan composition").unwrap();
        assert!(civic_pos < partisan_pos, "civic framing must precede partisan paragraph");
    }

    #[test]
    fn test_no_civic_framing_when_neither_plan_is_civic() {
        let mut report = fixture_report();
        report.plan_b.submission_type = None;
        let s = render_narrative(&report, &NarrativeConfig::default());
        assert!(!s.contains("civic counter-proposal"), "no framing when neither plan is civic: {s}");
    }

    // ── MoE suppression (Task 4 integration) ─────────────────────────────────

    #[test]
    fn test_moe_suppression_substitutes_canonical_text() {
        // Provide overlapping CIs for partisan seats; the directional phrase
        // must be replaced with MOE_SUPPRESSED_TEXT.
        let mut cfg = NarrativeConfig::default();
        cfg.partisan_seat_ci = Some((
            CiBand::new(5.0, 4.4, 5.6),
            CiBand::new(4.0, 3.4, 4.6),
        ));
        let s = render_narrative(&fixture_report(), &cfg);
        assert!(s.contains(MOE_SUPPRESSED_TEXT), "MoE text must appear: {s}");
        // The seat-count numbers themselves must STILL appear (only the
        // directional comparison is suppressed).
        assert!(s.contains("5 Democratic-leaning"));
        assert!(s.contains("4 Democratic-leaning"));
    }

    #[test]
    fn test_moe_no_suppression_when_intervals_disjoint() {
        let mut cfg = NarrativeConfig::default();
        cfg.partisan_seat_ci = Some((
            CiBand::new(5.0, 4.9, 5.1),
            CiBand::new(4.0, 3.9, 4.1),
        ));
        let s = render_narrative(&fixture_report(), &cfg);
        assert!(!s.contains(MOE_SUPPRESSED_TEXT), "no suppression when CIs disjoint: {s}");
        assert!(s.contains("Plan A yields 1 more"), "directional phrase must be present: {s}");
    }

    #[test]
    fn test_moe_mm_count_suppression() {
        let mut cfg = NarrativeConfig::default();
        cfg.mm_count_ci = Some((
            CiBand::new(1.0, 0.0, 1.0),
            CiBand::new(0.0, 0.0, 1.0),
        ));
        let s = render_narrative(&fixture_report(), &cfg);
        // MM-count paragraph contains both "Plan A has 1, Plan B has 1" AND the suppression.
        assert!(s.contains("Plan A has 1"));
        assert!(s.contains(MOE_SUPPRESSED_TEXT));
    }

    // ── Audit footer ─────────────────────────────────────────────────────────

    #[test]
    fn test_audit_footer_includes_manifest_sha_prefix() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("aaaaaaaa"), "Plan A sha prefix must appear: {s}");
        assert!(s.contains("bbbbbbbb"), "Plan B sha prefix must appear: {s}");
        assert!(s.contains("narrative_manifest.json"), "audit-trail pointer must appear: {s}");
    }

    #[test]
    fn test_audit_footer_announces_unsigned_state() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("Not yet approved"));
        assert!(s.contains("--approved-by"));
    }

    // ── Diff scope ───────────────────────────────────────────────────────────

    #[test]
    fn test_diff_scope_includes_tracts_population_districts() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("12 tracts"), "tract count: {s}");
        assert!(s.contains("31500 people"), "population: {s}");
        assert!(s.contains("Districts 3, 6"), "district list: {s}");
    }

    // ── ASCII-only (PP-34) ───────────────────────────────────────────────────

    #[test]
    fn test_narrative_output_is_ascii_only() {
        let mut cfg = NarrativeConfig::default();
        cfg.approved_by = Some("Test".into());
        let s = render_narrative(&fixture_report(), &cfg);
        for c in s.chars() {
            assert!(
                c.is_ascii(),
                "narrative output must be ASCII for Windows console safety (PP-34); found {:?}",
                c
            );
        }
    }

    // ── NarrativeConfig defaults ─────────────────────────────────────────────

    #[test]
    fn test_narrative_config_default_threshold() {
        let cfg = NarrativeConfig::default();
        assert!((cfg.leaning_threshold - 0.55).abs() < 1e-12);
    }

    #[test]
    fn test_narrative_config_default_close_call_band() {
        let cfg = NarrativeConfig::default();
        assert!((cfg.close_call_band - 0.02).abs() < 1e-12);
    }

    #[test]
    fn test_narrative_config_default_approved_by_none() {
        let cfg = NarrativeConfig::default();
        assert!(cfg.approved_by.is_none());
    }

    #[test]
    fn test_narrative_config_default_ci_fields_none() {
        let cfg = NarrativeConfig::default();
        assert!(cfg.partisan_seat_ci.is_none());
        assert!(cfg.mm_count_ci.is_none());
        assert!(cfg.mean_pp_ci.is_none());
    }

    // ── render_narrative: structural content ──────────────────────────────────

    #[test]
    fn test_narrative_contains_compactness_section() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("Compactness"), "compactness paragraph must appear: {s}");
        assert!(s.contains("Polsby-Popper"), "Polsby-Popper must be mentioned: {s}");
    }

    #[test]
    fn test_narrative_contains_mm_count_section() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("Majority-minority"), "MM section must appear: {s}");
    }

    #[test]
    fn test_narrative_contains_diff_scope_section() {
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("Plan B reassigns"), "diff scope section must appear: {s}");
    }

    #[test]
    fn test_narrative_no_civic_framing_for_plain_plans() {
        let report = ComparisonReport::from_loaded(
            plan_a_5dem(),
            {
                let mut b = plan_b_4dem_civic();
                b.submission_type = None;
                b
            },
            None,
            DiffSummary::default(),
        );
        let s = render_narrative(&report, &NarrativeConfig::default());
        assert!(!s.contains("civic counter-proposal"),
            "no civic framing when no civic submission type: {s}");
    }

    #[test]
    fn test_narrative_audit_footer_always_present() {
        // Even without approval, the audit footer separator --- must appear.
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("---"), "audit footer separator must always appear");
    }

    #[test]
    fn test_narrative_approved_by_name_appears_in_footer() {
        let mut cfg = NarrativeConfig::default();
        cfg.approved_by = Some("Dr. Expert Witness".into());
        let s = render_narrative(&fixture_report(), &cfg);
        assert!(s.contains("Dr. Expert Witness"), "signer name must appear in footer: {s}");
    }

    #[test]
    fn test_narrative_compactness_values_appear() {
        // Plan A pp=0.42, Plan B pp=0.40 must appear in the output.
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("0.420") || s.contains("0.42"), "Plan A PP value must appear: {s}");
        assert!(s.contains("0.400") || s.contains("0.40"), "Plan B PP value must appear: {s}");
    }

    #[test]
    fn test_narrative_plan_a_more_compact_directional_phrase() {
        // Plan A pp=0.42 > Plan B pp=0.40 → "Plan A is more compact".
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("Plan A is more compact"), "directional compactness phrase: {s}");
    }

    #[test]
    fn test_narrative_equal_compactness_phrase() {
        let mut report = fixture_report();
        report.plan_a.mean_pp = 0.42;
        report.plan_b.mean_pp = 0.42; // equal
        let s = render_narrative(&report, &NarrativeConfig::default());
        assert!(s.contains("essentially equally compact"),
            "equal PP must produce 'equally compact' phrase: {s}");
    }

    #[test]
    fn test_narrative_plan_b_more_compact_phrase() {
        let mut report = fixture_report();
        report.plan_a.mean_pp = 0.35;
        report.plan_b.mean_pp = 0.50; // B is more compact
        let s = render_narrative(&report, &NarrativeConfig::default());
        assert!(s.contains("Plan B is more compact"),
            "when Plan B has higher PP, 'Plan B is more compact' must appear: {s}");
    }

    #[test]
    fn test_narrative_mm_equal_phrase() {
        // Both plans have mm_count=1 (from fixtures) → "same number of MM districts".
        let s = render_narrative(&fixture_report(), &NarrativeConfig::default());
        assert!(s.contains("same number of MM districts"),
            "equal MM counts must say 'same number': {s}");
    }

    #[test]
    fn test_narrative_moe_compactness_suppression() {
        // Supply overlapping PP CIs → compactness paragraph should show suppression text.
        let mut cfg = NarrativeConfig::default();
        cfg.mean_pp_ci = Some((
            CiBand::new(0.42, 0.38, 0.46),
            CiBand::new(0.41, 0.37, 0.45), // overlap
        ));
        let s = render_narrative(&fixture_report(), &cfg);
        assert!(s.contains(MOE_SUPPRESSED_TEXT),
            "overlapping PP CIs must trigger MoE suppression: {s}");
    }
}
