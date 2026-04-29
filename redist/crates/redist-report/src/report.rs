/// report.rs — 10-section report assembly.
///
/// Spec 6 / Board amendments:
/// - assemble_report() returns Err (not panic) when analysis files are missing
/// - check_required_analysis_files() lists which files are missing before assembly
/// - External analyzer disclaimer required in HTML when external_analyzers present
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::manifest::PlanManifest;
use crate::audit::{generate_verification_command_from_manifest, generate_verification_script};

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
        Self { plan_dir, manifest, map_dir }
    }
}

/// Names of required analysis files (relative to plan_dir/analysis/).
pub const REQUIRED_ANALYSIS_FILES: &[&str] = &[
    "summary.json",
    "contiguity.json",
    "compactness.json",
];

/// Optional analysis files (not required — report section will note "unavailable").
pub const OPTIONAL_ANALYSIS_FILES: &[&str] = &[
    "partisan.json",
    "splits.json",
    "vra_analysis.json",
    "comparison.json",
    "nesting.json",
];

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

/// Check which optional analysis files are present.
fn present_optional_files(ctx: &ReportContext) -> std::collections::HashSet<&'static str> {
    let analysis_dir = ctx.plan_dir.join("analysis");
    OPTIONAL_ANALYSIS_FILES
        .iter()
        .filter(|&&name| analysis_dir.join(name).exists())
        .copied()
        .collect()
}

/// Read an analysis JSON file if present.
/// Injects `"status": "ok"` into the result so templates can always
/// check `.status == "unavailable"` regardless of file structure.
fn read_analysis_json(analysis_dir: &Path, name: &str) -> Option<Value> {
    let path = analysis_dir.join(name);
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .map(|mut v| {
                if let Some(obj) = v.as_object_mut() {
                    obj.entry("status").or_insert_with(|| serde_json::json!("ok"));
                }
                v
            })
    } else {
        None
    }
}

/// Fully assembled 10-section redistricting report.
#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub report_version: String,
    pub label: String,
    pub state: String,
    pub year: String,
    pub generated_at: String,
    pub sections: ReportSections,
    /// Whether any external analyzers were used (triggers disclaimer in HTML)
    pub has_external_analyzers: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSections {
    /// Section 1: Executive summary
    pub executive_summary: Value,
    /// Section 2: Population equality
    pub population_equality: Value,
    /// Section 3: Geographic constraints (contiguity + splits)
    pub geographic_constraints: Value,
    /// Section 4: Partisan fairness
    pub partisan_fairness: Value,
    /// Section 5: VRA compliance
    pub vra_compliance: Value,
    /// Section 6: Compactness
    pub compactness: Value,
    /// Section 7: Comparison vs enacted plan
    pub comparison: Value,
    /// Section 8: Nesting compliance
    pub nesting: Value,
    /// Section 9: Audit trail
    pub audit: Value,
    /// Section 10: Maps
    pub maps: Value,
}

/// Assemble a full 10-section report from a plan context.
///
/// Board amendment: returns Err if required analysis files are missing.
/// (Does NOT panic, does NOT produce a partial report.)
pub fn assemble_report(ctx: &ReportContext) -> anyhow::Result<Report> {
    // R3 amendment: check for required files first
    let missing = check_required_analysis_files(ctx);
    if !missing.is_empty() {
        anyhow::bail!(
            "Cannot assemble report: required analysis files missing: {}. \
             Run 'redist analyze' first.",
            missing.join(", ")
        );
    }

    let analysis_dir = ctx.plan_dir.join("analysis");
    let m = &ctx.manifest;

    // Section 1: Executive summary
    let exec_summary = serde_json::json!({
        "label": m.label,
        "state": m.state_code,
        "year": m.year,
        "chamber": m.chamber,
        "num_districts": m.num_districts,
        "binary_version": m.binary_version,
        "created_at": m.created_at,
        "population_balance_valid": m.population_balance_valid,
        "balance_tolerance_pct": m.balance_tolerance_pct,
    });

    // Section 2: Population equality (written by SummaryAnalyzer as summary.json)
    let population_json = read_analysis_json(&analysis_dir, "summary.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 3: Geographic constraints
    let contiguity_json = read_analysis_json(&analysis_dir, "contiguity.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));
    let splits_json = read_analysis_json(&analysis_dir, "splits.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));
    let geographic = serde_json::json!({
        "contiguity": contiguity_json,
        "splits": splits_json,
    });

    // Section 4: Partisan fairness (optional)
    let partisan = read_analysis_json(&analysis_dir, "partisan.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 5: VRA compliance (optional)
    let vra = read_analysis_json(&analysis_dir, "vra_analysis.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 6: Compactness
    let compactness = read_analysis_json(&analysis_dir, "compactness.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 7: Comparison (optional)
    let comparison = read_analysis_json(&analysis_dir, "comparison.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 8: Nesting (optional)
    let nesting = read_analysis_json(&analysis_dir, "nesting.json")
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 9: Audit trail
    let verification_command = generate_verification_command_from_manifest(m);
    let verification_script = generate_verification_script(m);
    let audit = serde_json::json!({
        "verification_command": verification_command,
        "verification_script": verification_script,
        "binary_version": m.binary_version,
        "binary_download_url": m.binary_download_url,
        "binary_sha256": m.binary_sha256,
        "adjacency_file": m.adjacency_file,
        "adjacency_sha256": if m.adjacency_sha256.is_empty() { "(not computed — run: sha256sum adjacency_file)".to_string() } else { m.adjacency_sha256.clone() },
        "tiger_source_url": m.tiger_source_url,
        "tiger_sha256": m.tiger_sha256.clone().unwrap_or_else(|| format!("(not recorded — download from {} and compute manually)", m.tiger_source_url)),
        "created_at": m.created_at,
        "seed": m.seed,
    });

    // Section 10: Maps (embed base64 PNG if available)
    let maps = build_maps_section(ctx);

    Ok(Report {
        report_version: "1.0".into(),
        label: m.label.clone(),
        state: m.state_code.clone(),
        year: m.year.clone(),
        generated_at: crate::paths::now_iso8601(),
        has_external_analyzers: false,
        sections: ReportSections {
            executive_summary: exec_summary,
            population_equality: population_json,
            geographic_constraints: geographic,
            partisan_fairness: partisan,
            vra_compliance: vra,
            compactness,
            comparison,
            nesting,
            audit,
            maps,
        },
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

    fn setup_plan_dir_missing_file(
        tmp: &TempDir,
        label: &str,
        skip_file: &str,
    ) -> ReportContext {
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
        assert!(sections["population_equality"].is_object(), "section 2 missing");
        assert!(sections["geographic_constraints"].is_object(), "section 3 missing");
        assert!(sections["partisan_fairness"].is_object(), "section 4 missing");
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
        let cmd = report.sections.audit["verification_command"].as_str().unwrap_or("");
        assert!(!cmd.is_empty(), "audit section must contain verification_command");
        assert!(
            cmd.contains("redist state"),
            "verification command must contain 'redist state'"
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
}
