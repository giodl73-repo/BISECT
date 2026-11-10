use serde_json::Value;
/// report.rs — 10-section report assembly.
///
/// Spec 6 / Board amendments:
/// - assemble_report() returns Err (not panic) when analysis files are missing
/// - check_required_analysis_files() lists which files are missing before assembly
/// - External analyzer disclaimer required in HTML when external_analyzers present
use std::path::{Path, PathBuf};

use crate::manifest::PlanManifest;

/// Context needed to assemble a report.
/// Points to the plan directory containing manifest.json and analysis/ subdirectory.
pub struct ReportContext {
    pub plan_dir: PathBuf,
    pub manifest: PlanManifest,
    pub map_dir: Option<PathBuf>,
}

impl ReportContext {
    pub fn new(plan_dir: PathBuf, manifest: PlanManifest) -> Self {
        let map_dir = Some(plan_dir.join("maps"));
        Self {
            plan_dir,
            manifest,
            map_dir,
        }
    }
}

/// Names of required analysis files (relative to plan_dir/analysis/).
/// Check which required analysis files are missing.
/// Returns list of missing filenames (relative to analysis/).
pub fn check_required_analysis_files(ctx: &ReportContext) -> Vec<&'static str> {
    let analysis_dir = ctx.plan_dir.join("analysis");
    REQUIRED_ANALYSIS_FILES
        .iter()
        .filter(|&&name| !analysis_dir.join(name).exists())
        .copied()
        .collect()
}

/// Read an analysis JSON file if present.
/// Injects `"status": "ok"` into the result so templates can always
/// check `.status == "unavailable"` regardless of file structure.
fn read_analysis_json(analysis_dir: &Path, name: &str) -> anyhow::Result<Option<Value>> {
    let path = analysis_dir.join(name);
    if !path.exists() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(&path).map_err(|e| {
        anyhow::anyhow!("[INPUT] cannot read analysis file {}: {e}", path.display())
    })?;
    let mut value = serde_json::from_str::<Value>(&text).map_err(|e| {
        anyhow::anyhow!(
            "[INPUT] cannot parse analysis file {} as JSON: {e}",
            path.display()
        )
    })?;
    if let Some(obj) = value.as_object_mut() {
        obj.entry("status")
            .or_insert_with(|| serde_json::json!("ok"));
    }
    Ok(Some(value))
}

pub use crate::report_assembly::{Report, ReportSections, REQUIRED_ANALYSIS_FILES, OPTIONAL_ANALYSIS_FILES};

/// Assemble a full 10-section report from a plan context.
///
/// Board amendment: returns Err if required analysis files are missing.
/// (Does NOT panic, does NOT produce a partial report.)
pub fn assemble_report(ctx: &ReportContext) -> anyhow::Result<Report> {
    let missing = check_required_analysis_files(ctx);
    if !missing.is_empty() {
        anyhow::bail!("Cannot assemble report: required analysis files missing: {}. Run 'bisect analyze' first.", missing.join(", "));
    }
    let analysis_dir = ctx.plan_dir.join("analysis");
    let mut analysis = std::collections::BTreeMap::new();
    for name in ["summary.json", "contiguity.json", "splits.json", "partisan.json", "vra_analysis.json", "compactness.json", "comparison.json", "nesting.json"] {
        if let Some(value) = read_analysis_json(&analysis_dir, name)? {
            analysis.insert(name.to_owned(), value);
        }
    }
    crate::report_assembly::assemble_report_from_values(
        &ctx.manifest, &analysis, build_audit_section(ctx, &ctx.manifest)?,
        build_maps_section(ctx), crate::paths::now_iso8601(),
    )
}

fn build_audit_section(ctx: &ReportContext, m: &PlanManifest) -> anyhow::Result<Value> {
    crate::report_assembly::build_audit_from_values(m, read_rplan_source_hashes(ctx, m)?, read_rplan_algorithm_lineage(ctx, m)?)
}

fn read_rplan_algorithm_lineage(ctx: &ReportContext, m: &PlanManifest) -> anyhow::Result<Value> {
    let Some(certificate_rel) = m.audit_certificate_path.as_deref() else {
        return Ok(serde_json::json!({
            "status": "unavailable",
            "reason": "audit_certificate_path not recorded in manifest",
        }));
    };
    let certificate_path = ctx.plan_dir.join(certificate_rel);
    let text = std::fs::read_to_string(&certificate_path).map_err(|e| {
        anyhow::anyhow!("[INPUT] cannot read audit certificate {certificate_rel}: {e}")
    })?;
    let certificate: Value = serde_json::from_str(&text).map_err(|e| {
        anyhow::anyhow!("[INPUT] cannot parse audit certificate {certificate_rel}: {e}")
    })?;
    Ok(match certificate.get("algorithm_lineage") {
        Some(Value::Null) | None => serde_json::json!({
            "status": "unavailable",
            "reason": "certificate has no algorithm_lineage",
        }),
        Some(lineage) => serde_json::json!({
            "status": "ok",
            "lineage": lineage,
        }),
    })
}

fn read_rplan_source_hashes(ctx: &ReportContext, m: &PlanManifest) -> anyhow::Result<Value> {
    let Some(rctx_rel) = m.rctx_path.as_deref() else {
        return Ok(serde_json::json!({
            "status": "unavailable",
            "reason": "rctx_path not recorded in manifest",
        }));
    };
    let rctx_path = ctx.plan_dir.join(rctx_rel);
    let text = std::fs::read_to_string(&rctx_path)
        .map_err(|e| anyhow::anyhow!("[INPUT] cannot read RCTX {rctx_rel}: {e}"))?;
    let context = rplan_io::read_rctx_str(&text)
        .map_err(|e| anyhow::anyhow!("[INPUT] cannot parse RCTX {rctx_rel}: {e}"))?;
    Ok(if context.source_hashes.entries.is_empty() {
        serde_json::json!({
            "status": "missing",
            "reason": "context source_hashes is empty",
        })
    } else {
        serde_json::json!({
            "status": "ok",
            "entries": context.source_hashes.entries,
        })
    })
}

/// Build the maps section (section 10) — embed PNG maps as base64 data URIs.
fn build_maps_section(ctx: &ReportContext) -> Value {
    let Some(ref map_dir) = ctx.map_dir else {
        return serde_json::json!({"status": "unavailable"});
    };

    // Look for common map file names
    let candidates = [
        "districts.png",
        "district_map.png",
        "political.png",
        "compactness.png",
    ];

    let mut maps_obj = serde_json::Map::new();
    for name in &candidates {
        let path = map_dir.join(name);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let key = name.trim_end_matches(".png").replace('.', "_");
                maps_obj.insert(key, serde_json::json!(b64));
            }
        }
    }

    if maps_obj.is_empty() {
        serde_json::json!({"status": "unavailable"})
    } else {
        Value::Object(maps_obj)
    }
}

// ---------------------------------------------------------------------------
// Tests — Task 5
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
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
            binary_download_url: "https://github.com/owner/BISECT/releases/download/v0.1.0/BISECT"
                .into(),
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

    fn setup_plan_dir_with_all_required(tmp: &TempDir, label: &str) -> ReportContext {
        let plan_dir = tmp.path().join("plans").join(label);
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        // Write all required files
        for name in REQUIRED_ANALYSIS_FILES {
            std::fs::write(
                analysis_dir.join(name),
                serde_json::to_string(&serde_json::json!({"status": "ok", "file": name})).unwrap(),
            )
            .unwrap();
        }
        let manifest = make_test_manifest(label);
        ReportContext::new(plan_dir, manifest)
    }

    fn setup_plan_dir_missing_file(tmp: &TempDir, label: &str, skip_file: &str) -> ReportContext {
        let plan_dir = tmp.path().join("plans").join(label);
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        for name in REQUIRED_ANALYSIS_FILES {
            if *name != skip_file {
                std::fs::write(
                    analysis_dir.join(name),
                    serde_json::to_string(&serde_json::json!({"status": "ok"})).unwrap(),
                )
                .unwrap();
            }
        }
        let manifest = make_test_manifest(label);
        ReportContext::new(plan_dir, manifest)
    }

    fn setup_plan_dir_missing_partisan(tmp: &TempDir, label: &str) -> ReportContext {
        // No partisan.json (it's optional)
        let ctx = setup_plan_dir_with_all_required(tmp, label);
        // Make sure partisan.json does NOT exist
        let _ = std::fs::remove_file(ctx.plan_dir.join("analysis").join("partisan.json"));
        ctx
    }

    #[test]
    fn test_report_json_contains_all_sections() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_test");
        let report = assemble_report(&ctx).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        let sections = &v["sections"];
        assert!(
            sections["population_equality"].is_object(),
            "section 2 missing"
        );
        assert!(
            sections["geographic_constraints"].is_object(),
            "section 3 missing"
        );
        assert!(
            sections["partisan_fairness"].is_object(),
            "section 4 missing"
        );
        assert!(sections["vra_compliance"].is_object(), "section 5 missing");
        assert!(sections["compactness"].is_object(), "section 6 missing");
        assert!(sections["audit"].is_object(), "section 9 missing");
    }

    #[test]
    fn test_report_version_is_1_0() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_test2");
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(report.report_version, "1.0");
    }

    #[test]
    fn test_audit_section_includes_verification_command() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_audit_test");
        let report = assemble_report(&ctx).unwrap();
        let cmd = report.sections.audit["verification_command"]
            .as_str()
            .unwrap_or("");
        assert!(
            !cmd.is_empty(),
            "audit section must contain verification_command"
        );
        assert!(
            cmd.contains("bisect state"),
            "verification command must contain 'bisect state'"
        );
    }

    #[test]
    fn test_report_validates_missing_analysis_files() {
        // R3 amendment: missing required file → Err listing the filename
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_missing_file(&tmp, "vt_missing", "contiguity.json");
        let missing = check_required_analysis_files(&ctx);
        assert!(
            missing.contains(&"contiguity.json"),
            "missing contiguity.json must be reported"
        );
    }

    #[test]
    fn test_assemble_report_returns_err_on_missing_required_file() {
        // Board amendment: assemble_report() must return Err when required files absent
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_missing_file(&tmp, "vt_err_test", "summary.json");
        let result = assemble_report(&ctx);
        assert!(
            result.is_err(),
            "assemble_report must return Err when required files missing"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("summary.json"),
            "error must mention the missing file, got: {msg}"
        );
    }

    #[test]
    fn test_partisan_json_missing_does_not_fail_assembly() {
        // partisan.json is optional — assembly should succeed without it
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_missing_partisan(&tmp, "vt_nopartisan");
        let result = assemble_report(&ctx);
        assert!(
            result.is_ok(),
            "assemble_report must succeed when partisan.json is absent (optional)"
        );
        let report = result.unwrap();
        let partisan = &report.sections.partisan_fairness;
        assert_eq!(
            partisan["status"].as_str().unwrap_or(""),
            "unavailable",
            "partisan_fairness must show 'unavailable' when file absent"
        );
    }

    #[test]
    fn test_assemble_report_rejects_malformed_required_analysis_json() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_bad_summary");
        std::fs::write(
            ctx.plan_dir.join("analysis").join("summary.json"),
            "{bad json",
        )
        .unwrap();

        let err = assemble_report(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]")
                && msg.contains("summary.json")
                && msg.contains("cannot parse analysis file"),
            "malformed required analysis JSON must fail explicitly: {msg}"
        );
    }

    #[test]
    fn test_assemble_report_rejects_malformed_optional_analysis_json() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_bad_partisan");
        std::fs::write(
            ctx.plan_dir.join("analysis").join("partisan.json"),
            "{bad json",
        )
        .unwrap();

        let err = assemble_report(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]")
                && msg.contains("partisan.json")
                && msg.contains("cannot parse analysis file"),
            "malformed optional analysis JSON must not be downgraded to unavailable: {msg}"
        );
    }

    #[test]
    fn test_check_required_files_missing_partisan_not_reported() {
        // partisan.json is NOT in REQUIRED_ANALYSIS_FILES
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_partisan_ok");
        // Remove partisan.json (it's optional, shouldn't matter)
        let _ = std::fs::remove_file(ctx.plan_dir.join("analysis").join("partisan.json"));
        let missing = check_required_analysis_files(&ctx);
        assert!(
            !missing.contains(&"partisan.json"),
            "partisan.json should not be listed as required missing"
        );
    }

    // ── Additional L0 tests ───────────────────────────────────────────────────

    #[test]
    fn test_required_analysis_files_contains_summary() {
        assert!(
            REQUIRED_ANALYSIS_FILES.contains(&"summary.json"),
            "summary.json must be in REQUIRED_ANALYSIS_FILES"
        );
    }

    #[test]
    fn test_required_analysis_files_contains_contiguity() {
        assert!(
            REQUIRED_ANALYSIS_FILES.contains(&"contiguity.json"),
            "contiguity.json must be in REQUIRED_ANALYSIS_FILES"
        );
    }

    #[test]
    fn test_required_analysis_files_contains_compactness() {
        assert!(
            REQUIRED_ANALYSIS_FILES.contains(&"compactness.json"),
            "compactness.json must be in REQUIRED_ANALYSIS_FILES"
        );
    }

    #[test]
    fn test_optional_analysis_files_contains_partisan() {
        assert!(
            OPTIONAL_ANALYSIS_FILES.contains(&"partisan.json"),
            "partisan.json must be in OPTIONAL_ANALYSIS_FILES"
        );
    }

    #[test]
    fn test_optional_analysis_files_contains_vra() {
        assert!(
            OPTIONAL_ANALYSIS_FILES.contains(&"vra_analysis.json"),
            "vra_analysis.json must be in OPTIONAL_ANALYSIS_FILES"
        );
    }

    #[test]
    fn test_optional_files_not_in_required() {
        for optional in OPTIONAL_ANALYSIS_FILES {
            assert!(
                !REQUIRED_ANALYSIS_FILES.contains(optional),
                "{optional} must not appear in both REQUIRED and OPTIONAL lists"
            );
        }
    }

    #[test]
    fn test_required_files_not_in_optional() {
        for required in REQUIRED_ANALYSIS_FILES {
            assert!(
                !OPTIONAL_ANALYSIS_FILES.contains(required),
                "{required} must not appear in both REQUIRED and OPTIONAL lists"
            );
        }
    }

    #[test]
    fn test_report_context_new_sets_map_dir() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("test");
        std::fs::create_dir_all(&plan_dir).unwrap();
        let manifest = make_test_manifest("test");
        let ctx = ReportContext::new(plan_dir.clone(), manifest);
        assert_eq!(ctx.map_dir, Some(plan_dir.join("maps")));
    }

    #[test]
    fn test_report_context_plan_dir_stored() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("stored_test");
        std::fs::create_dir_all(&plan_dir).unwrap();
        let manifest = make_test_manifest("stored_test");
        let ctx = ReportContext::new(plan_dir.clone(), manifest);
        assert_eq!(ctx.plan_dir, plan_dir);
    }

    #[test]
    fn test_check_required_files_all_present_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_full");
        let missing = check_required_analysis_files(&ctx);
        assert!(
            missing.is_empty(),
            "all required files present → missing must be empty"
        );
    }

    #[test]
    fn test_check_required_files_missing_compactness_reported() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_missing_file(&tmp, "vt_no_compact", "compactness.json");
        let missing = check_required_analysis_files(&ctx);
        assert!(
            missing.contains(&"compactness.json"),
            "missing compactness.json must be in missing list"
        );
    }

    #[test]
    fn test_check_required_files_missing_summary_reported() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_missing_file(&tmp, "vt_no_summary", "summary.json");
        let missing = check_required_analysis_files(&ctx);
        assert!(
            missing.contains(&"summary.json"),
            "missing summary.json must be in missing list"
        );
    }

    #[test]
    fn test_assemble_report_sets_label() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_label_check");
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(report.label, "vt_label_check");
    }

    #[test]
    fn test_assemble_report_sets_state() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_state_check");
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(report.state, "VT");
    }

    #[test]
    fn test_assemble_report_sets_year() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_year_check");
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(report.year, "2020");
    }

    #[test]
    fn test_assemble_report_generated_at_is_iso8601() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_ts_check");
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(
            report.generated_at.len(),
            20,
            "generated_at must be 20-char ISO 8601 string"
        );
        assert!(report.generated_at.contains('T'));
        assert!(report.generated_at.ends_with('Z'));
    }

    #[test]
    fn test_assemble_report_has_external_analyzers_false_by_default() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_ext_check");
        let report = assemble_report(&ctx).unwrap();
        assert!(
            !report.has_external_analyzers,
            "has_external_analyzers must be false by default"
        );
    }

    #[test]
    fn test_assemble_report_executive_summary_has_label() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_exec_check");
        let report = assemble_report(&ctx).unwrap();
        let label = report.sections.executive_summary["label"]
            .as_str()
            .unwrap_or("");
        assert_eq!(label, "vt_exec_check");
    }

    #[test]
    fn test_assemble_report_audit_section_has_binary_version() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_bin_check");
        let report = assemble_report(&ctx).unwrap();
        let ver = report.sections.audit["binary_version"]
            .as_str()
            .unwrap_or("");
        assert!(
            !ver.is_empty(),
            "audit section must have non-empty binary_version"
        );
    }

    #[test]
    fn test_assemble_report_audit_section_surfaces_rplan_source_hashes() {
        let tmp = TempDir::new().unwrap();
        let mut ctx = setup_plan_dir_with_all_required(&tmp, "vt_rplan_audit");
        ctx.manifest.rplan_path = Some("plan.rplan".to_string());
        ctx.manifest.rctx_path = Some("context.rctx".to_string());
        ctx.manifest.audit_certificate_path = Some("audit-certificate.json".to_string());
        ctx.manifest.audit_certificate_sha256 = Some("d".repeat(64));
        ctx.manifest.audit_certificate_content_hash =
            Some("sha256:certificate-content".to_string());
        ctx.manifest.audit_result = Some("pass".to_string());
        ctx.manifest.legal_profile_id = Some("generic-v1".to_string());
        ctx.manifest.context_hash = Some("sha256:context".to_string());

        let mut source_entries = std::collections::BTreeMap::new();
        source_entries.insert(
            "adjacency".to_string(),
            format!("sha256:{}", "b".repeat(64)),
        );
        source_entries.insert("geometry".to_string(), format!("sha256:{}", "c".repeat(64)));
        let context = rplan_core::RplanContext {
            rctx_version: rplan_core::RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units: rplan_core::PlanUnitIndex {
                unit_kind: rplan_core::UnitKind::Tract,
                state: Some("VT".to_string()),
                year: Some(2020),
                canonical_order: rplan_core::CanonicalOrder::ExplicitUnitIds,
                unit_ids: vec!["50001000100".to_string()],
                unit_universe_hash: "sha256:test-universe".to_string(),
                source_id: None,
            },
            graph: None,
            populations: Some(vec![100]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: rplan_core::SourceHashes {
                entries: source_entries,
            },
        };
        std::fs::write(
            ctx.plan_dir.join("context.rctx"),
            rplan_io::write_rctx_string(&context).unwrap(),
        )
        .unwrap();
        std::fs::write(
            ctx.plan_dir.join("audit-certificate.json"),
            serde_json::json!({
                "algorithm_lineage": {
                    "producer_crate": "bisect-ilp",
                    "producer_version": "0.1.0",
                    "method": "branch-and-cut",
                    "parent_plan_hashes": [],
                    "parameters_hash": "sha256:params",
                    "extra": {
                        "fallback": "metis",
                        "audit_summary_sha256": "d".repeat(64)
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let report = assemble_report(&ctx).unwrap();
        assert_eq!(report.sections.audit["audit_result"], "pass");
        assert_eq!(report.sections.audit["rplan_source_hashes"]["status"], "ok");
        assert_eq!(
            report.sections.audit["rplan_source_hashes"]["entries"]["adjacency"],
            format!("sha256:{}", "b".repeat(64))
        );
        assert_eq!(
            report.sections.audit["rplan_source_hashes"]["entries"]["geometry"],
            format!("sha256:{}", "c".repeat(64))
        );
        assert_eq!(
            report.sections.audit["algorithm_lineage"]["lineage"]["method"],
            "branch-and-cut"
        );
    }

    #[test]
    fn test_assemble_report_rejects_malformed_rctx_audit_reference() {
        let tmp = TempDir::new().unwrap();
        let mut ctx = setup_plan_dir_with_all_required(&tmp, "vt_bad_rctx_audit");
        ctx.manifest.rctx_path = Some("context.rctx".to_string());
        std::fs::write(ctx.plan_dir.join("context.rctx"), "{bad rctx").unwrap();

        let err = assemble_report(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]")
                && msg.contains("cannot parse RCTX")
                && msg.contains("context.rctx"),
            "malformed recorded RCTX must not become unavailable audit evidence: {msg}"
        );
    }

    #[test]
    fn test_assemble_report_rejects_malformed_audit_certificate_reference() {
        let tmp = TempDir::new().unwrap();
        let mut ctx = setup_plan_dir_with_all_required(&tmp, "vt_bad_certificate_audit");
        ctx.manifest.audit_certificate_path = Some("audit-certificate.json".to_string());
        std::fs::write(ctx.plan_dir.join("audit-certificate.json"), "{bad json").unwrap();

        let err = assemble_report(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]")
                && msg.contains("cannot parse audit certificate")
                && msg.contains("audit-certificate.json"),
            "malformed recorded audit certificate must not become unavailable audit evidence: {msg}"
        );
    }

    #[test]
    fn test_assemble_report_returns_err_on_missing_contiguity() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_missing_file(&tmp, "vt_no_cont", "contiguity.json");
        let result = assemble_report(&ctx);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("contiguity.json"),
            "error must mention contiguity.json"
        );
    }

    #[test]
    fn test_assemble_report_vra_unavailable_when_missing() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_vra_test");
        // Remove vra_analysis.json (optional) to ensure it returns "unavailable"
        let _ = std::fs::remove_file(ctx.plan_dir.join("analysis").join("vra_analysis.json"));
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(
            report.sections.vra_compliance["status"]
                .as_str()
                .unwrap_or(""),
            "unavailable",
            "vra_compliance must show 'unavailable' when file absent"
        );
    }

    #[test]
    fn test_assemble_report_comparison_unavailable_when_missing() {
        let tmp = TempDir::new().unwrap();
        let ctx = setup_plan_dir_with_all_required(&tmp, "vt_comp_test");
        let _ = std::fs::remove_file(ctx.plan_dir.join("analysis").join("comparison.json"));
        let report = assemble_report(&ctx).unwrap();
        assert_eq!(
            report.sections.comparison["status"].as_str().unwrap_or(""),
            "unavailable",
            "comparison must show 'unavailable' when file absent"
        );
    }
}
