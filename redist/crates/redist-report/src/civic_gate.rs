//! Civic-input gating for court-mode reports (BD-R1).
//!
//! When a plan's analysis chain references civic inputs ingested under
//! `--validate {lenient,advisory}`, court-mode PDF generation refuses to embed
//! those inputs unless the operator passes `--allow-non-strict-civic`.
//!
//! Spec: Court Submission Reports plan Task 8.
//! Tracking: BD-R1.
//!
//! This module is independent of the Civic Bidirectional plan's *write* side
//! (which doesn't ship until that plan lands). The read side scans for any
//! `civic-coi v1` manifest under the plan dir's `civic_inputs/` subtree (or a
//! sibling directory referenced by the plan's manifest, when that wiring lands)
//! and inspects the `validate_mode` field.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Result of inspecting a plan's analysis chain for civic inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CivicGateOutcome {
    /// No civic inputs in the chain — no restriction applies.
    NoneFound,
    /// Every civic input was ingested under `--validate strict` — court-mode safe.
    AllStrict { paths: Vec<PathBuf> },
    /// At least one civic input was lenient/advisory and `--allow-non-strict-civic`
    /// was NOT supplied — court-mode render must REFUSE.
    NonStrictRefused { offending: Vec<NonStrictRow> },
    /// At least one civic input was lenient/advisory but the operator opted in
    /// via `--allow-non-strict-civic` — render proceeds; the override is
    /// recorded in the reproducibility package manifest.
    NonStrictAllowed { offending: Vec<NonStrictRow> },
}

/// One row of the override audit trail. The reproducibility package manifest
/// records this verbatim.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NonStrictRow {
    pub path: PathBuf,
    pub validate_mode: String,
    pub label: String,
}

/// Civic-coi manifest shape. Only the fields we read are listed; serde tolerates
/// the rest. Schema source of truth: `docs/file-formats/manifests.md` §3.8.
#[derive(Debug, Deserialize)]
struct CivicManifestRead {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    validate_mode: String,
}

/// Scan `plan_dir/civic_inputs/*/manifest.json` (and any `civic_inputs/*/manifest.json`
/// under the plan's `output_root`-sibling `civic_inputs/` if present) for civic-coi
/// manifests. Apply the gating policy.
///
/// `civic_inputs_root` is the directory expected to contain `<label>/manifest.json`
/// subdirectories. Pass `None` to skip filesystem scanning (useful for unit tests
/// that pre-build a list of manifests).
pub fn check_civic_inputs(
    civic_inputs_root: Option<&Path>,
    allow_non_strict_civic: bool,
) -> anyhow::Result<CivicGateOutcome> {
    let mut found: Vec<(PathBuf, CivicManifestRead)> = Vec::new();
    if let Some(root) = civic_inputs_root {
        if root.is_dir() {
            for entry in std::fs::read_dir(root)? {
                let entry = entry?;
                let manifest_path = entry.path().join("manifest.json");
                if !manifest_path.is_file() {
                    continue;
                }
                let bytes = std::fs::read(&manifest_path)?;
                match serde_json::from_slice::<CivicManifestRead>(&bytes) {
                    Ok(m) => {
                        if m.schema_version.starts_with("civic-coi v") {
                            found.push((manifest_path, m));
                        }
                        // Other schema versions silently ignored — not our concern.
                    }
                    Err(_) => {
                        // Malformed manifests aren't our concern here either; the
                        // civic-ingest validator would've caught them. Don't fail
                        // the whole report render on a stray bad JSON file.
                    }
                }
            }
        }
    }
    classify(found, allow_non_strict_civic)
}

/// Classify the gating outcome from a (already-loaded) list of manifests.
/// Test entry point.
pub fn classify(
    manifests: Vec<(PathBuf, CivicManifestRead)>,
    allow_non_strict_civic: bool,
) -> anyhow::Result<CivicGateOutcome> {
    if manifests.is_empty() {
        return Ok(CivicGateOutcome::NoneFound);
    }
    let mut strict_paths: Vec<PathBuf> = Vec::new();
    let mut offending: Vec<NonStrictRow> = Vec::new();
    for (path, m) in manifests {
        match m.validate_mode.as_str() {
            "strict" => strict_paths.push(path),
            "lenient" | "advisory" => offending.push(NonStrictRow {
                path,
                validate_mode: m.validate_mode,
                label: m.label,
            }),
            other => {
                // Unknown mode: be conservative — treat as non-strict.
                offending.push(NonStrictRow {
                    path,
                    validate_mode: format!("unknown:{other}"),
                    label: m.label,
                });
            }
        }
    }
    if offending.is_empty() {
        Ok(CivicGateOutcome::AllStrict {
            paths: strict_paths,
        })
    } else if allow_non_strict_civic {
        Ok(CivicGateOutcome::NonStrictAllowed { offending })
    } else {
        Ok(CivicGateOutcome::NonStrictRefused { offending })
    }
}

/// Format a refusal as the `[BOUNDARY]` actionable error string. Use this to
/// build the `anyhow::Error` returned by the report-render pipeline.
pub fn refusal_message(offending: &[NonStrictRow]) -> String {
    let mut s = String::from(
        "[BOUNDARY] court-mode render refused: civic input(s) ingested under non-strict validation:\n",
    );
    for row in offending {
        s.push_str(&format!(
            "  - {} ({}, validate_mode={})\n",
            row.path.display(),
            row.label,
            row.validate_mode
        ));
    }
    s.push_str(
        "Pass --allow-non-strict-civic to embed them anyway; the override will be \
         recorded in the reproducibility package manifest. See \
         docs/file-formats/manifests.md section 3.8 (civic-coi v1) and \
         docs/superpowers/plans/2026-04-30-court-submission-reports.md Task 8.",
    );
    // ASCII-only per PP-34 (Windows CP1252 console policy). The error string
    // flows through anyhow::Error -> stderr, which the redist-cli envelope
    // writes via eprintln! — must not contain non-ASCII bytes.
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_civic_manifest(dir: &Path, label: &str, validate_mode: &str) -> PathBuf {
        let label_dir = dir.join(label);
        fs::create_dir_all(&label_dir).unwrap();
        let path = label_dir.join("manifest.json");
        let body = serde_json::json!({
            "schema_version": "civic-coi v1",
            "label": label,
            "validate_mode": validate_mode,
        });
        fs::write(&path, body.to_string()).unwrap();
        path
    }

    #[test]
    fn test_none_found_when_civic_inputs_dir_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let outcome = check_civic_inputs(Some(&tmp.path().join("nope")), false).unwrap();
        assert_eq!(outcome, CivicGateOutcome::NoneFound);
    }

    #[test]
    fn test_none_found_when_civic_inputs_dir_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("civic_inputs")).unwrap();
        let outcome = check_civic_inputs(Some(&tmp.path().join("civic_inputs")), false).unwrap();
        assert_eq!(outcome, CivicGateOutcome::NoneFound);
    }

    #[test]
    fn test_all_strict_passes() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        write_civic_manifest(&root, "lwv_comments", "strict");
        write_civic_manifest(&root, "naacp_coi", "strict");
        let outcome = check_civic_inputs(Some(&root), false).unwrap();
        match outcome {
            CivicGateOutcome::AllStrict { paths } => assert_eq!(paths.len(), 2),
            other => panic!("expected AllStrict, got {:?}", other),
        }
    }

    #[test]
    fn test_one_lenient_refused_without_allow_flag() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        write_civic_manifest(&root, "lwv_strict", "strict");
        write_civic_manifest(&root, "lwv_lenient", "lenient");
        let outcome = check_civic_inputs(Some(&root), false).unwrap();
        match outcome {
            CivicGateOutcome::NonStrictRefused { offending } => {
                assert_eq!(offending.len(), 1);
                assert_eq!(offending[0].validate_mode, "lenient");
                assert_eq!(offending[0].label, "lwv_lenient");
            }
            other => panic!("expected NonStrictRefused, got {:?}", other),
        }
    }

    #[test]
    fn test_one_advisory_refused_without_allow_flag() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        write_civic_manifest(&root, "lwv_advisory", "advisory");
        let outcome = check_civic_inputs(Some(&root), false).unwrap();
        match outcome {
            CivicGateOutcome::NonStrictRefused { offending } => {
                assert_eq!(offending.len(), 1);
                assert_eq!(offending[0].validate_mode, "advisory");
            }
            other => panic!("expected NonStrictRefused, got {:?}", other),
        }
    }

    #[test]
    fn test_lenient_allowed_with_flag() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        write_civic_manifest(&root, "lwv_lenient", "lenient");
        let outcome = check_civic_inputs(Some(&root), true).unwrap();
        match outcome {
            CivicGateOutcome::NonStrictAllowed { offending } => {
                assert_eq!(offending.len(), 1);
                assert_eq!(offending[0].validate_mode, "lenient");
            }
            other => panic!("expected NonStrictAllowed, got {:?}", other),
        }
    }

    #[test]
    fn test_unknown_validate_mode_treated_as_non_strict() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        write_civic_manifest(&root, "weird", "future_mode");
        let outcome = check_civic_inputs(Some(&root), false).unwrap();
        match outcome {
            CivicGateOutcome::NonStrictRefused { offending } => {
                assert!(offending[0].validate_mode.starts_with("unknown:"));
            }
            other => panic!("expected NonStrictRefused, got {:?}", other),
        }
    }

    #[test]
    fn test_non_civic_schema_silently_ignored() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        let label_dir = root.join("not_civic");
        fs::create_dir_all(&label_dir).unwrap();
        fs::write(
            label_dir.join("manifest.json"),
            serde_json::json!({"schema_version": "something-else v1"}).to_string(),
        )
        .unwrap();
        let outcome = check_civic_inputs(Some(&root), false).unwrap();
        // Non-civic schema is not our concern — falls through to NoneFound.
        assert_eq!(outcome, CivicGateOutcome::NoneFound);
    }

    #[test]
    fn test_malformed_manifest_does_not_panic() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("civic_inputs");
        let label_dir = root.join("bad");
        fs::create_dir_all(&label_dir).unwrap();
        fs::write(label_dir.join("manifest.json"), "{ not valid json").unwrap();
        // Don't fail the whole render on a stray bad JSON file in civic_inputs.
        let outcome = check_civic_inputs(Some(&root), false).unwrap();
        assert_eq!(outcome, CivicGateOutcome::NoneFound);
    }

    #[test]
    fn test_refusal_message_names_offending_paths_and_actionable_hint() {
        let offending = vec![NonStrictRow {
            path: PathBuf::from("outputs/v1/civic_inputs/lwv_lenient/manifest.json"),
            validate_mode: "lenient".into(),
            label: "lwv_lenient".into(),
        }];
        let msg = refusal_message(&offending);
        assert!(msg.starts_with("[BOUNDARY]"));
        assert!(msg.contains("lwv_lenient"));
        assert!(msg.contains("--allow-non-strict-civic"));
        assert!(msg.contains("manifests.md"));
    }
}
