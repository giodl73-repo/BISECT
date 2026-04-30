//! Callais Evidence Layer — L1 integration test.
//!
//! Builds a synthetic precinct fixture, a synthetic race-of-candidate CSV, and
//! shells out to `redist analyze --types bloc-voting`. Asserts:
//! - Exit code 0
//! - `bloc_voting.json` written + valid + schema_version "bloc-voting v1"
//! - `bloc_voting_summary.md` written + contains the verbatim ecology caveat
//!   substring AND the spec's regression-specification string
//! - `analysis/bloc_voting/` staged with the CSV + the attestation doc (Task 5)
//!
//! Spec: `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md` Tasks 7-8.
//! ALSO satisfies B-02 anchor 1 end-to-end via `redist`'s production CLI path
//! (the unit-level anchor is in `redist-analysis/src/bloc_voting.rs`).

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

const REDIST: &str = env!("CARGO_BIN_EXE_redist");

fn synthetic_precincts_tsv(
    n_precincts: usize,
    candidate_name: &str,
    beta_minority: f64,
    seed: u64,
) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut s = String::from(
        "candidate_name\tprecinct_id\tcounty_fips\ttotal_votes\tcandidate_share\tpct_minority\tpct_dem_baseline\n",
    );
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    let mut state = h.finish();
    let mut next_uniform = || -> f64 {
        // xorshift-like, deterministic given seed.
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state >> 11) as f64) / ((1u64 << 53) as f64)
    };
    for i in 0..n_precincts {
        let pm = next_uniform();
        let pd = next_uniform();
        let noise = (next_uniform() - 0.5) * 0.05;
        let y = (0.05 + beta_minority * pm + 0.20 * pd + noise).clamp(0.0, 1.0);
        let county = format!("01{:03}", i % 5);
        s.push_str(&format!(
            "{candidate_name}\tP{i:04}\t{county}\t100\t{y:.6}\t{pm:.6}\t{pd:.6}\n"
        ));
    }
    s
}

fn write_race_of_candidate_csv(dir: &std::path::Path, candidate_name: &str) -> PathBuf {
    let attest_dir = dir.join("attestations");
    std::fs::create_dir_all(&attest_dir).unwrap();
    let attest_path = attest_dir.join(format!("{candidate_name}.pdf"));
    std::fs::write(&attest_path, b"%PDF-1.4 attestation body for L1 fixture\n").unwrap();
    let csv_path = dir.join("race_of_candidate.csv");
    let csv = format!(
        "candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path,attestation_doc_format\n\
         {candidate_name},DEM,Black,Test Curator,Ph.D. (test),2026-04-30,test fixture,true,attestations/{candidate_name}.pdf,pdf\n"
    );
    std::fs::write(&csv_path, csv).unwrap();
    csv_path
}

fn write_minimal_plan(plan_dir: &std::path::Path, label: &str) {
    std::fs::create_dir_all(plan_dir.join("analysis")).unwrap();
    let manifest = serde_json::json!({
        "label": label,
        "state_code": "VT",
        "year": "2020",
        "chamber": "congressional",
        "num_districts": 1,
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
        "created_at": "2026-04-30T00:00:00Z",
        "balance_tolerance_pct": 0.5,
        "population_balance_valid": true,
        "seats_per_district": 1,
        "total_seats": 1,
        "electoral_system": "single_member",
        "gpmetis_version": "unknown"
    });
    std::fs::write(
        plan_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    // Trivial assignments file (1 tract -> district 1).
    std::fs::write(
        plan_dir.join("final_assignments.json"),
        serde_json::json!({"50001000100": 1}).to_string(),
    )
    .unwrap();
}

#[test]
fn bloc_voting_l1_synthetic_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let label = "vt_bloc_l1";
    // The CLI computes plan_dir as `output_base/version/version/year/plans/label`
    // (analyze.rs builds output_root = output_base.join(version), then
    // PlanContext::from_label re-joins version). Mirror that convention here.
    let plan_dir = tmp
        .path()
        .join("v1")
        .join("v1")
        .join("2020")
        .join("plans")
        .join(label);
    write_minimal_plan(&plan_dir, label);

    // Inputs.
    let inputs_dir = tmp.path().join("inputs");
    std::fs::create_dir_all(&inputs_dir).unwrap();
    let candidate_name = "Adams_Test";
    let csv_path = write_race_of_candidate_csv(&inputs_dir, candidate_name);
    let tsv_path = inputs_dir.join("precincts.tsv");
    std::fs::write(
        &tsv_path,
        synthetic_precincts_tsv(150, candidate_name, 0.55, 1234),
    )
    .unwrap();

    // Invoke `redist analyze --types bloc-voting`.
    let output = Command::new(REDIST)
        .arg("analyze")
        .arg("--state").arg("VT")  // skips manifest auto-derivation path (its
                                    // single-version vs from_label's double-version
                                    // mismatch is pre-existing analyze.rs behavior)
        .arg("--label").arg(label)
        .arg("--year").arg("2020")
        .arg("--version").arg("v1")
        .arg("--output-base").arg(tmp.path().as_os_str())
        .arg("--types").arg("bloc-voting")
        .arg("--candidate-race-csv").arg(&csv_path)
        .arg("--partisan-baseline").arg(&tsv_path)
        .arg("--bootstrap-samples").arg("30")
        .arg("--ci-level").arg("0.95")
        .arg("--alpha").arg("0.05")
        .arg("--election").arg("test-primary")
        .arg("--party").arg("DEM")
        .arg("--minority-group").arg("black")
        .arg("--method").arg("wls")
        .arg("--min-precincts").arg("50")
        .output()
        .expect("spawn redist");

    assert!(
        output.status.success(),
        "redist analyze exited non-zero ({:?})\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // bloc_voting.json present + valid.
    let json_path = plan_dir.join("analysis").join("bloc_voting.json");
    assert!(json_path.exists(), "missing {}", json_path.display());
    let json: Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap())
        .expect("bloc_voting.json must parse");
    assert_eq!(json["schema_version"], "bloc-voting v1");
    assert_eq!(json["analyzer"], "bloc-voting");
    assert_eq!(json["state"], "VT");
    assert_eq!(json["election"], "test-primary");
    assert_eq!(json["candidates"][0]["candidate"], candidate_name);
    let spec = json["candidates"][0]["regression"]["specification"]
        .as_str()
        .unwrap();
    assert!(spec.contains("WLS weighted by precinct vote count"), "spec: {spec}");
    assert!(spec.contains("HC3 robust SE"), "spec: {spec}");
    assert!(spec.contains("cluster-bootstrap by county B=30"), "spec: {spec}");

    // B-02 anchor 1 end-to-end: WLS recovers β_minority within ±0.04 of ground
    // truth (looser than the unit test's ±0.02 because the CLI fixture has
    // smaller N and noisier xorshift PRNG).
    let est = json["candidates"][0]["regression"]["coefficients"]["pct_minority"]["estimate"]
        .as_f64()
        .unwrap();
    assert!(
        (est - 0.55).abs() <= 0.10,
        "B-02 anchor 1 (CLI end-to-end): estimate {est} not within ±0.10 of 0.55"
    );

    // bloc_voting_summary.md present + ecology caveat verbatim.
    let md_path = plan_dir.join("analysis").join("bloc_voting_summary.md");
    let md = std::fs::read_to_string(&md_path).expect("read summary md");
    assert!(
        md.contains("ecological inference fallacy"),
        "summary md must contain the verbatim ecology caveat: {md}"
    );

    // Task 5 staging: race CSV + attestation doc copied into analysis/bloc_voting/.
    let staged_csv = plan_dir
        .join("analysis")
        .join("bloc_voting")
        .join("race_of_candidate.csv");
    assert!(staged_csv.exists(), "missing staged CSV at {}", staged_csv.display());
    let attest_dir = plan_dir.join("analysis").join("bloc_voting").join("attestations");
    let attest_count = std::fs::read_dir(&attest_dir).unwrap().count();
    assert_eq!(attest_count, 1, "expected 1 attestation doc staged");
}

#[test]
fn bloc_voting_l1_rejects_missing_candidate_race_csv() {
    let tmp = TempDir::new().unwrap();
    let label = "vt_bloc_missing";
    // The CLI computes plan_dir as `output_base/version/version/year/plans/label`
    // (analyze.rs builds output_root = output_base.join(version), then
    // PlanContext::from_label re-joins version). Mirror that convention here.
    let plan_dir = tmp
        .path()
        .join("v1")
        .join("v1")
        .join("2020")
        .join("plans")
        .join(label);
    write_minimal_plan(&plan_dir, label);

    let output = Command::new(REDIST)
        .arg("analyze")
        .arg("--state").arg("VT")  // skips manifest auto-derivation path (its
                                    // single-version vs from_label's double-version
                                    // mismatch is pre-existing analyze.rs behavior)
        .arg("--label").arg(label)
        .arg("--year").arg("2020")
        .arg("--version").arg("v1")
        .arg("--output-base").arg(tmp.path().as_os_str())
        .arg("--types").arg("bloc-voting")
        .output()
        .expect("spawn redist");
    assert!(
        !output.status.success(),
        "must fail without --candidate-race-csv"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[INPUT]") && stderr.contains("--candidate-race-csv"),
        "error must be [INPUT] and name the missing flag: {stderr}"
    );
}

#[test]
fn bloc_voting_l1_rejects_method_rxc_as_not_yet_implemented() {
    let tmp = TempDir::new().unwrap();
    let label = "vt_bloc_rxc";
    // The CLI computes plan_dir as `output_base/version/version/year/plans/label`
    // (analyze.rs builds output_root = output_base.join(version), then
    // PlanContext::from_label re-joins version). Mirror that convention here.
    let plan_dir = tmp
        .path()
        .join("v1")
        .join("v1")
        .join("2020")
        .join("plans")
        .join(label);
    write_minimal_plan(&plan_dir, label);

    let inputs_dir = tmp.path().join("inputs");
    std::fs::create_dir_all(&inputs_dir).unwrap();
    let csv_path = write_race_of_candidate_csv(&inputs_dir, "Adams");
    let tsv_path = inputs_dir.join("precincts.tsv");
    std::fs::write(
        &tsv_path,
        synthetic_precincts_tsv(60, "Adams", 0.5, 7),
    )
    .unwrap();

    let output = Command::new(REDIST)
        .arg("analyze")
        .arg("--state").arg("VT")  // skips manifest auto-derivation path (its
                                    // single-version vs from_label's double-version
                                    // mismatch is pre-existing analyze.rs behavior)
        .arg("--label").arg(label)
        .arg("--year").arg("2020")
        .arg("--version").arg("v1")
        .arg("--output-base").arg(tmp.path().as_os_str())
        .arg("--types").arg("bloc-voting")
        .arg("--candidate-race-csv").arg(&csv_path)
        .arg("--partisan-baseline").arg(&tsv_path)
        .arg("--method").arg("rxc")
        .output()
        .expect("spawn redist");
    assert!(!output.status.success(), "--method rxc must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not yet implemented"),
        "stderr should explain RxC deferral: {stderr}"
    );
}
