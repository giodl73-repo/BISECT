use std::process::Command;

fn audit_fixture_path(name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rplan-audit/fixtures")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn path_context() -> &'static str {
    include_str!("../../rplan-io/src/fixtures/path5.rctx")
}

fn plan(chamber: &str, assignment: &str) -> String {
    format!(
        r#"{{
  "rplan_version": "0.2",
  "plan": {{
    "schema_version": "district-plan-v1",
    "units": {{
      "unit_kind": "tract",
      "state": "WA",
      "year": 2020,
      "canonical_order": "explicit-unit-ids",
      "unit_ids": ["53001000100", "53001000200", "53001000300", "53001000400", "53001000500"],
      "unit_universe_hash": "sha256:path5-unit-universe"
    }},
    "assignment": {assignment},
    "k": 2,
    "display_labels": ["1", "2"],
    "allow_empty_districts": false
  }},
  "metadata": {{
    "label": "wa_path5",
    "jurisdiction": "WA",
    "chamber": "{chamber}",
    "created_at": "2026-05-10T00:00:00Z"
  }},
  "provenance": {{}},
  "geometry": null,
  "extensions": {{}}
}}"#
    )
}

#[test]
fn audit_valid_plan_exits_zero_with_allow_warnings() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    let ctx_path = tmp.path().join("context.rctx");
    std::fs::write(&plan_path, plan("congressional", "[0, 0, 0, 1, 1]")).unwrap();
    std::fs::write(&ctx_path, path_context()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--context",
            ctx_path.to_str().unwrap(),
            "--constraints",
            "plan-shape,contiguity",
            "--allow-warnings",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
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
    assert!(stdout.contains(r#""result":"pass-with-warnings""#));
}

#[test]
fn audit_disconnected_plan_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    let ctx_path = tmp.path().join("context.rctx");
    std::fs::write(&plan_path, plan("congressional", "[0, 1, 0, 1, 1]")).unwrap();
    std::fs::write(&ctx_path, path_context()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--context",
            ctx_path.to_str().unwrap(),
            "--constraints",
            "contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}

#[test]
fn state_house_without_profile_exits_two() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    std::fs::write(&plan_path, plan("state-house", "[0, 0, 0, 1, 1]")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--constraints",
            "plan-shape",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--legal-profile is required"));
}

#[test]
fn state_house_incomplete_profile_exits_two() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    let profile_path = tmp.path().join("profile.json");
    std::fs::write(&plan_path, plan("state-house", "[0, 0, 0, 1, 1]")).unwrap();
    std::fs::write(
        &profile_path,
        include_str!("../../rplan-audit/profiles/incomplete-state-house-profile.json"),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--legal-profile",
            profile_path.to_str().unwrap(),
            "--constraints",
            "plan-shape",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("state legislative audit requires an explicit legal profile"));
}

#[test]
fn grid3x3_valid_audit_matches_golden_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--constraints",
            "plan-shape,population,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        include_str!("../../rplan-audit/fixtures/grid3x3-valid-certificate.json").trim()
    );
}

#[test]
fn grid3x3_disconnected_audit_matches_golden_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-disconnected.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--constraints",
            "plan-shape,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        include_str!("../../rplan-audit/fixtures/grid3x3-disconnected-certificate.json").trim()
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}

#[test]
fn grid3x3_missing_contiguity_audit_matches_golden_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--constraints",
            "plan-shape,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        include_str!("../../rplan-audit/fixtures/grid3x3-missing-contiguity-certificate.json")
            .trim()
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}
