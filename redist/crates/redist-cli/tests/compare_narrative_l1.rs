//! Plan Comparison plan Task 11 — `redist compare --format narrative` end-to-end.
//!
//! Builds two synthetic plan directories on disk, shells out to the production
//! `redist` binary with `--format narrative`, then asserts:
//! - exit code 0
//! - `narrative.md` is written under the comparisons dir, contains the
//!   `[DRAFT]` prefix when `--approved-by` is unset, drops the prefix when set
//! - `narrative_manifest.json` is written with the right schema_version,
//!   threshold/band, plan-a/plan-b labels and SHA-256s, civic-counter-proposal
//!   attribution when plan-b is tagged
//! - `--format both` ALSO prints the table to stdout
//!
//! Depends on the production binary; run with `cargo test -p redist-cli --test
//! compare_narrative_l1` (the binary fixture is auto-built via
//! `CARGO_BIN_EXE_redist`).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

const REDIST: &str = env!("CARGO_BIN_EXE_redist");

/// Minimal `manifest.json` for a synthetic plan. Mirrors the production
/// `PlanManifest` shape (see redist-report::manifest::PlanManifest); fields
/// the narrative dispatch doesn't read are still required so serde::Deserialize
/// succeeds upstream — but `load_plan_side_from_dir` parses with
/// `serde_json::Value` and only needs `label`, `num_districts`, and the
/// optional civic submission fields.
fn write_manifest(
    plan_dir: &Path,
    label: &str,
    num_districts: usize,
    submission_type: Option<&str>,
    submitted_by: Option<&str>,
    submitted_at: Option<&str>,
) {
    std::fs::create_dir_all(plan_dir).unwrap();
    let mut m = serde_json::json!({
        "label": label,
        "state_code": "VT",
        "year": "2020",
        "chamber": "congressional",
        "num_districts": num_districts,
        "population_source": "total",
        "partition_mode": "edge-weighted",
        "seed": null,
        "binary_version": "0.1.0",
        "binary_sha256": "",
        "binary_download_url": "",
        "adjacency_file": "",
        "adjacency_sha256": "",
        "adjacency_build_command": "",
        "adjacency_build_version": "0.1.0",
        "tiger_source_url": "",
        "tiger_sha256": null,
        "created_at": "2026-04-26T00:00:00Z",
        "balance_tolerance_pct": 0.5,
        "population_balance_valid": true,
        "seats_per_district": 1,
        "total_seats": num_districts,
        "electoral_system": "single_member",
        "gpmetis_version": "unknown"
    });
    if let Some(t) = submission_type {
        m["submission_type"] = serde_json::Value::String(t.to_string());
    }
    if let Some(by) = submitted_by {
        m["submitted_by"] = serde_json::Value::String(by.to_string());
    }
    if let Some(at) = submitted_at {
        m["submitted_at"] = serde_json::Value::String(at.to_string());
    }
    std::fs::write(
        plan_dir.join("manifest.json"),
        serde_json::to_string_pretty(&m).unwrap(),
    )
    .unwrap();
}

fn write_partisan(plan_dir: &Path, dem_shares: &[f64]) {
    let analysis = plan_dir.join("analysis");
    std::fs::create_dir_all(&analysis).unwrap();
    let v = serde_json::json!({ "per_district_dem_share": dem_shares });
    std::fs::write(
        analysis.join("partisan.json"),
        serde_json::to_string_pretty(&v).unwrap(),
    )
    .unwrap();
}

fn write_compactness(plan_dir: &Path, mean_pp: f64) {
    let analysis = plan_dir.join("analysis");
    std::fs::create_dir_all(&analysis).unwrap();
    let v = serde_json::json!({ "mean_polsby_popper": mean_pp });
    std::fs::write(
        analysis.join("compactness.json"),
        serde_json::to_string_pretty(&v).unwrap(),
    )
    .unwrap();
}

fn write_assignments(plan_dir: &Path, map: &[(&str, u64)]) {
    let data = plan_dir.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let mut obj = serde_json::Map::new();
    for (geoid, dist) in map {
        obj.insert(
            (*geoid).to_string(),
            serde_json::Value::Number((*dist).into()),
        );
    }
    std::fs::write(
        data.join("final_assignments.json"),
        serde_json::to_string_pretty(&serde_json::Value::Object(obj)).unwrap(),
    )
    .unwrap();
}

/// Build two synthetic plans under a temp output_base. Returns
/// (output_base, plan_a_label, plan_b_label).
fn build_two_plans(tmp: &TempDir) -> (PathBuf, &'static str, &'static str) {
    let output_base = tmp.path().to_path_buf();
    let plans_dir = output_base.join("v1").join("2020").join("plans");
    let plan_a = plans_dir.join("vt_state");
    let plan_b = plans_dir.join("vt_civic");
    write_manifest(&plan_a, "vt_state", 6, None, None, None);
    write_partisan(&plan_a, &[0.30, 0.45, 0.55, 0.60, 0.65, 0.70]);
    write_compactness(&plan_a, 0.42);
    write_assignments(
        &plan_a,
        &[
            ("50001000100", 1), ("50001000200", 2), ("50001000300", 3),
            ("50001000400", 4), ("50001000500", 5), ("50001000600", 6),
        ],
    );
    write_manifest(
        &plan_b,
        "vt_civic",
        6,
        Some("civic_counter_proposal"),
        Some("League of Women Voters Vermont"),
        Some("2026-04-15T12:00:00Z"),
    );
    write_partisan(&plan_b, &[0.25, 0.40, 0.547, 0.553, 0.62, 0.68]);
    write_compactness(&plan_b, 0.40);
    // Reassign two tracts to force diff
    write_assignments(
        &plan_b,
        &[
            ("50001000100", 1), ("50001000200", 3), ("50001000300", 2),
            ("50001000400", 4), ("50001000500", 5), ("50001000600", 6),
        ],
    );
    (output_base, "vt_state", "vt_civic")
}

#[test]
fn narrative_dispatch_writes_files_and_records_provenance() {
    let tmp = TempDir::new().unwrap();
    let (output_base, plan_a, plan_b) = build_two_plans(&tmp);

    let report_dir = tmp.path().join("out_narrative");

    let status = Command::new(REDIST)
        .args([
            "compare",
            "--plan-a", plan_a,
            "--plan-b", plan_b,
            "--year", "2020",
            "--version", "v1",
            "--output-base", output_base.to_str().unwrap(),
            "--report-dir", report_dir.to_str().unwrap(),
            "--format", "narrative",
        ])
        .env("SOURCE_DATE_EPOCH", "1700000000")
        .status()
        .expect("redist compare failed to launch");
    assert!(status.success(), "redist compare exit code = {:?}", status.code());

    let narrative_path = report_dir.join("narrative.md");
    let manifest_path = report_dir.join("narrative_manifest.json");
    assert!(narrative_path.is_file(), "narrative.md missing at {}", narrative_path.display());
    assert!(manifest_path.is_file(), "narrative_manifest.json missing at {}", manifest_path.display());

    // Narrative content asserts.
    let narrative = std::fs::read_to_string(&narrative_path).unwrap();
    assert!(
        narrative.contains("[DRAFT - review before publication]"),
        "narrative must carry DRAFT prefix when --approved-by is unset; got:\n{narrative}"
    );
    // ASCII-only per PP-34.
    assert!(
        narrative.is_ascii(),
        "narrative must be ASCII-only (PP-34); got non-ASCII content"
    );
    // Civic-counter-proposal framing fires (plan B is tagged).
    assert!(
        narrative.contains("League of Women Voters Vermont"),
        "civic-counter-proposal framing must mention submitter; got:\n{narrative}"
    );
    assert!(
        narrative.contains("not the state's official map"),
        "civic-counter-proposal framing must include the COMMONS disclaimer line; got:\n{narrative}"
    );

    // Manifest content asserts.
    let manifest_text = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: Value = serde_json::from_str(&manifest_text).expect("manifest is valid JSON");

    assert_eq!(
        manifest["schema_version"].as_str(),
        Some("narrative-manifest v1"),
        "schema_version mismatch"
    );
    assert_eq!(manifest["plan_a_label"].as_str(), Some(plan_a));
    assert_eq!(manifest["plan_b_label"].as_str(), Some(plan_b));
    assert_eq!(
        manifest["leaning_threshold"].as_f64(),
        Some(0.55),
        "leaning_threshold default = 0.55"
    );
    assert_eq!(
        manifest["close_call_band"].as_f64(),
        Some(0.02),
        "close_call_band default = 0.02"
    );

    // SHAs are 64-hex chars.
    let a_sha = manifest["plan_a_manifest_sha256"].as_str().unwrap();
    let b_sha = manifest["plan_b_manifest_sha256"].as_str().unwrap();
    let t_sha = manifest["template_sha256"].as_str().unwrap();
    assert_eq!(a_sha.len(), 64, "plan_a_manifest_sha256 must be 64 hex chars");
    assert_eq!(b_sha.len(), 64, "plan_b_manifest_sha256 must be 64 hex chars");
    assert_eq!(t_sha.len(), 64, "template_sha256 must be 64 hex chars");
    assert_ne!(a_sha, b_sha, "two distinct plans must have distinct manifest SHAs");

    // Per-plan analysis SHAs include partisan.json + compactness.json.
    let plan_a_analyses = manifest["analysis_sha256"]["plan_a"].as_object().unwrap();
    assert!(plan_a_analyses.contains_key("partisan.json"), "plan_a must record partisan.json SHA");
    assert!(plan_a_analyses.contains_key("compactness.json"), "plan_a must record compactness.json SHA");

    // Civic-counter-proposal attribution flows from plan B.
    let attribution = &manifest["civic_counter_proposal_attribution"];
    assert!(!attribution.is_null(), "civic_counter_proposal_attribution must be set when plan B is tagged");
    assert_eq!(
        attribution["plan_label"].as_str(),
        Some(plan_b),
        "attribution.plan_label must point to the civic-tagged plan"
    );
    assert_eq!(
        attribution["submitted_by"].as_str(),
        Some("League of Women Voters Vermont"),
    );

    // SOURCE_DATE_EPOCH propagated -> approved_at frozen.
    assert_eq!(
        manifest["approved_at"].as_str(),
        Some("2023-11-14T22:13:20Z"),
        "approved_at must reflect SOURCE_DATE_EPOCH=1700000000"
    );
    // approved_by stays null when --approved-by is unset.
    assert!(
        manifest["approved_by"].is_null(),
        "approved_by must be null in DRAFT mode; got {:?}",
        manifest["approved_by"]
    );
}

#[test]
fn narrative_dispatch_drops_draft_prefix_when_approved() {
    let tmp = TempDir::new().unwrap();
    let (output_base, plan_a, plan_b) = build_two_plans(&tmp);
    let report_dir = tmp.path().join("out_approved");

    let status = Command::new(REDIST)
        .args([
            "compare",
            "--plan-a", plan_a,
            "--plan-b", plan_b,
            "--year", "2020",
            "--version", "v1",
            "--output-base", output_base.to_str().unwrap(),
            "--report-dir", report_dir.to_str().unwrap(),
            "--format", "narrative",
            "--approved-by", "J. Doe (state demographer)",
        ])
        .env("SOURCE_DATE_EPOCH", "1700000000")
        .status()
        .expect("redist compare failed");
    assert!(status.success());

    let narrative = std::fs::read_to_string(report_dir.join("narrative.md")).unwrap();
    assert!(
        !narrative.contains("[DRAFT"),
        "approved narrative must NOT carry [DRAFT] prefix; got:\n{narrative}"
    );

    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(report_dir.join("narrative_manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        manifest["approved_by"].as_str(),
        Some("J. Doe (state demographer)"),
    );
}

#[test]
fn narrative_dispatch_format_both_prints_table_and_writes_files() {
    let tmp = TempDir::new().unwrap();
    let (output_base, plan_a, plan_b) = build_two_plans(&tmp);
    let report_dir = tmp.path().join("out_both");

    let out = Command::new(REDIST)
        .args([
            "compare",
            "--plan-a", plan_a,
            "--plan-b", plan_b,
            "--year", "2020",
            "--version", "v1",
            "--output-base", output_base.to_str().unwrap(),
            "--report-dir", report_dir.to_str().unwrap(),
            "--format", "both",
        ])
        .env("SOURCE_DATE_EPOCH", "1700000000")
        .output()
        .expect("redist compare failed");
    assert!(out.status.success(), "exit={:?} stderr={}", out.status.code(), String::from_utf8_lossy(&out.stderr));

    let stdout = String::from_utf8_lossy(&out.stdout);
    // The text table format is the legacy `format_comparison_table` output
    // which prints both plan labels as a column header row.
    assert!(
        stdout.contains(plan_a) && stdout.contains(plan_b),
        "--format both must print the legacy table to stdout containing both plan labels; got:\n{stdout}"
    );

    // Files still get written.
    assert!(report_dir.join("narrative.md").is_file(), "--format both must still write narrative.md");
    assert!(
        report_dir.join("narrative_manifest.json").is_file(),
        "--format both must still write narrative_manifest.json"
    );
}

#[test]
fn html_dispatch_writes_self_contained_html_with_civic_banner() {
    let tmp = TempDir::new().unwrap();
    let (output_base, plan_a, plan_b) = build_two_plans(&tmp);
    let report_dir = tmp.path().join("out_html");

    let status = Command::new(REDIST)
        .args([
            "compare",
            "--plan-a", plan_a,
            "--plan-b", plan_b,
            "--year", "2020",
            "--version", "v1",
            "--output-base", output_base.to_str().unwrap(),
            "--report-dir", report_dir.to_str().unwrap(),
            "--format", "html",
        ])
        .env("SOURCE_DATE_EPOCH", "1700000000")
        .status()
        .expect("redist compare --format html failed to launch");
    assert!(status.success(), "redist compare --format html exit = {:?}", status.code());

    let html_path = report_dir.join("comparison.html");
    assert!(html_path.is_file(), "comparison.html missing at {}", html_path.display());

    // Narrative + manifest also written (HTML embeds the narrative).
    assert!(report_dir.join("narrative.md").is_file());
    assert!(report_dir.join("narrative_manifest.json").is_file());

    let html = std::fs::read_to_string(&html_path).unwrap();

    // Self-contained: doctype + embedded CSS + no external src/href.
    assert!(html.starts_with("<!DOCTYPE html>"), "must be a complete HTML5 document");
    assert!(html.contains("<style>"), "CSS must be embedded inline");
    assert!(!html.contains("href=\"http"), "must not link external stylesheets");
    assert!(!html.contains("src=\"http"), "must not link external scripts/images");

    // Civic banner fires (plan B is tagged).
    assert!(
        html.contains("class=\"civic-banner\""),
        "civic-counter-proposal banner must render when plan B is tagged"
    );
    assert!(
        html.contains("League of Women Voters Vermont"),
        "submitter must appear in banner"
    );
    assert!(
        html.contains("not the state's official map"),
        "civic disclaimer must appear in banner"
    );

    // DRAFT badge (no --approved-by).
    assert!(html.contains("badge draft"), "DRAFT badge must render");

    // Side-by-side metrics table.
    assert!(html.contains("Democratic-leaning seats"));
    assert!(html.contains("Mean Polsby-Popper"));

    // Provenance footer with both plan SHAs.
    assert!(html.contains("class=\"provenance\""), "provenance footer must render");
}

#[test]
fn narrative_dispatch_errors_when_plan_b_missing() {
    let tmp = TempDir::new().unwrap();
    let (output_base, plan_a, _) = build_two_plans(&tmp);
    let report_dir = tmp.path().join("out_err");

    let out = Command::new(REDIST)
        .args([
            "compare",
            "--plan-a", plan_a,
            "--enacted",
            "--year", "2020",
            "--version", "v1",
            "--output-base", output_base.to_str().unwrap(),
            "--report-dir", report_dir.to_str().unwrap(),
            "--format", "narrative",
        ])
        .output()
        .expect("redist compare failed");
    assert!(!out.status.success(), "narrative dispatch must reject --enacted-only invocations");
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Either error path is acceptable: the older `--enacted` bail (which
    // fires before the narrative dispatch can run because plan_b is also
    // not loadable as an enacted source today) or the narrative-specific
    // [CONFIG] guard. Both are rejections — what we're verifying is that
    // the narrative path doesn't silently produce a half-rendered file.
    assert!(
        stderr.contains("[CONFIG]")
            || stderr.contains("requires --plan-b")
            || stderr.contains("requires an enacted plan file"),
        "error must reject the invocation with an actionable message; got:\n{stderr}"
    );
    // No narrative.md should have been written.
    assert!(
        !report_dir.join("narrative.md").exists(),
        "narrative.md must not be written when the dispatch errors out"
    );
}
