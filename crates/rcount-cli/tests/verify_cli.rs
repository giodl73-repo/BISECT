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
