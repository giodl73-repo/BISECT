use std::process::Command;

fn docs_summary_basic_path() -> String {
    docs_package_path("summary-basic")
}

fn docs_canvass_correction_path() -> String {
    docs_package_path("canvass-correction")
}

fn docs_bad_selection_sum_path() -> String {
    docs_package_path("bad-selection-sum")
}

fn docs_mail_batch_added_path() -> String {
    docs_package_path("mail-batch-added")
}

fn docs_missing_batch_path() -> String {
    docs_package_path("missing-batch")
}

fn docs_precinct_split_lineage_path() -> String {
    docs_package_path("precinct-split-lineage")
}

fn docs_bad_lineage_path() -> String {
    docs_package_path("bad-lineage")
}

fn docs_privacy_inclusion_sketch_path() -> String {
    docs_package_path("privacy-inclusion-sketch")
}

fn docs_choice_bearing_proof_path() -> String {
    docs_package_path("choice-bearing-proof")
}

fn docs_cvr_summary_path() -> String {
    docs_package_path("cvr-summary")
}

fn docs_bad_cvr_summary_path() -> String {
    docs_package_path("bad-cvr-summary")
}

fn docs_rla_replay_path() -> String {
    docs_package_path("rla-replay")
}

fn docs_bad_rla_replay_path() -> String {
    docs_package_path("bad-rla-replay")
}

fn docs_rla_stopping_path() -> String {
    docs_package_path("rla-stopping")
}

fn docs_bad_rla_stopping_path() -> String {
    docs_package_path("bad-rla-stopping")
}

fn docs_rla_discrepancy_path() -> String {
    docs_package_path("rla-discrepancy")
}

fn docs_bad_rla_discrepancy_path() -> String {
    docs_package_path("bad-rla-discrepancy")
}

fn docs_rla_margin_path() -> String {
    docs_package_path("rla-margin")
}

fn docs_bad_rla_margin_path() -> String {
    docs_package_path("bad-rla-margin")
}

fn docs_district_aggregation_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join("district-aggregation-rplan")
}

fn docs_multi_election_cycle_path(cycle_id: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join("multi-election-harness")
        .join(cycle_id)
        .join("package")
        .to_string_lossy()
        .into_owned()
}

fn docs_multi_election_negative_path(case_name: &str, cycle_id: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join("multi-election-harness-negatives")
        .join(case_name)
        .join(cycle_id)
}

fn docs_package_path(package_name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join(package_name)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn verify_summary_basic_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_summary_basic_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""equation_id":"contest_selection_sum""#));
}

#[test]
fn verify_canvass_correction_exposes_event_correlation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_canvass_correction_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""equation_id":"canvass_correction_event""#));
}

#[test]
fn verify_bad_selection_sum_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_selection_sum_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"contest_selection_sum""#));
    assert!(stdout.contains("contest selection sum mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_mail_batch_added_exposes_batch_correlation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_mail_batch_added_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_summary_total""#));
    assert!(stdout.contains(r#""reporting_unit_id":"batch:P-001:late-mail""#));
}

#[test]
fn verify_missing_batch_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_missing_batch_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_summary_total""#));
    assert!(stdout.contains("references missing batch id"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_precinct_split_lineage_exposes_lineage_correlation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_precinct_split_lineage_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"lineage_conservation""#));
    assert!(stdout.contains(r#""reporting_unit_id":"lineage:P-004-split""#));
}

#[test]
fn verify_bad_lineage_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_lineage_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"lineage_conservation""#));
    assert!(stdout.contains("missing current reporting unit"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_privacy_inclusion_exposes_privacy_gate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_privacy_inclusion_sketch_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"proof_privacy_gate""#));
    assert!(stdout.contains(r#""reporting_unit_id":"proof:accepted-token-001""#));
}

#[test]
fn verify_choice_bearing_proof_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_choice_bearing_proof_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"proof_privacy_gate""#));
    assert!(stdout.contains("exposes candidate selections"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_cvr_summary_exposes_cvr_reconciliation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_cvr_summary_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"cvr_summary_total""#));
    assert!(stdout.contains(r#""reporting_unit_id":"syn:precinct:P-001""#));
}

#[test]
fn verify_bad_cvr_summary_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_cvr_summary_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"cvr_summary_total""#));
    assert!(stdout.contains("CVR summary mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_replay_exposes_sampler_replay() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_replay_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_sampler_replay""#));
    assert!(stdout.contains(r#""reporting_unit_id":"rla:syn-2024-mayor:round-1""#));
}

#[test]
fn verify_bad_rla_replay_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_rla_replay_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_sampler_replay""#));
    assert!(stdout.contains("RLA audit"));
    assert!(stdout.contains("sample mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_stopping_exposes_stopping_rule() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_stopping_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains(r#""equation_id":"rla_sampler_replay""#));
}

#[test]
fn verify_bad_rla_stopping_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_rla_stopping_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains("declares status Pass, computed Escalate"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_discrepancy_exposes_taxonomy() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_discrepancy_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains(r#""status":"pass""#));
}

#[test]
fn verify_bad_rla_discrepancy_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_bad_rla_discrepancy_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains("discrepancy mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_margin_exposes_margin_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_margin_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_margin_metadata""#));
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
}

#[test]
fn verify_bad_rla_margin_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_rla_margin_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_margin_metadata""#));
    assert!(stdout.contains("reported margin mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_tampered_manifest_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    let manifest_path = tmp.path().join("manifest.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("content_hash mismatch"));
}

#[test]
fn verify_tampered_source_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    std::fs::write(
        tmp.path()
            .join("sources")
            .join("synthetic-summary-export.json"),
        br#"{"tampered":true}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains(r#""equation_id":"source_hash_match""#));
}

#[test]
fn verify_missing_source_hash_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    std::fs::write(
        tmp.path().join("sources").join("source-index.json"),
        br#"{"sources":[]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"source_hash_match""#));
    assert!(stdout.contains("source index is empty"));
}

#[test]
fn verify_can_write_transcript_to_package() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    let transcript_path = tmp
        .path()
        .join("transcripts")
        .join("verify-transcript.json");
    std::fs::remove_file(&transcript_path).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            tmp.path().to_str().unwrap(),
            "--write-transcript",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let transcript = std::fs::read_to_string(transcript_path).unwrap();
    assert!(transcript.contains(r#""verifier": "rcount-audit""#));
}

#[test]
fn aggregate_districts_with_rplan_outputs_district_totals() {
    let dir = docs_district_aggregation_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "aggregate-districts",
            dir.join("package").to_str().unwrap(),
            "--plan",
            dir.join("plan.rplan.json").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"district_aggregation_total""#));
    assert!(stdout.contains(r#""district_label":"SYN-D1""#));
    assert!(stdout.contains(r#""counted_ballots":80"#));
    assert!(stdout.contains(r#""rplan_plan_hash":"sha256:"#));
}

#[test]
fn verify_multi_election_cycle_uses_package_contest_for_jurisdiction_total() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_multi_election_cycle_path("SYN-2028-general"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""contest_id":"syn-cycle-mayor""#));
    assert!(stdout.contains(r#""equation_id":"jurisdiction_contest_total""#));
}

#[test]
fn verify_bad_multi_election_lineage_exits_one() {
    let cycle_dir = docs_multi_election_negative_path("bad-lineage", "SYN-2028-general");
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            cycle_dir.join("package").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"lineage_conservation""#));
    assert!(stdout.contains("references missing current reporting unit"));
}

#[test]
fn aggregate_stale_multi_election_plan_exits_two() {
    let cycle_dir = docs_multi_election_negative_path("stale-plan", "SYN-2028-general");
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "aggregate-districts",
            cycle_dir.join("package").to_str().unwrap(),
            "--plan",
            cycle_dir.join("plan.rplan.json").to_str().unwrap(),
            "--contest-id",
            "syn-cycle-mayor",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("missing plan unit summary"));
    assert!(stderr.contains("syn:precinct:P-002"));
}

#[test]
fn verify_tampered_multi_election_source_exits_one() {
    let cycle_dir = docs_multi_election_negative_path("tampered-2028-source", "SYN-2028-general");
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            cycle_dir.join("package").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"source_hash_match""#));
    assert!(stdout.contains("source hash mismatch"));
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
