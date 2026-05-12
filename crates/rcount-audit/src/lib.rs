use rcount_core::{
    package_content_hash, verify_canvass_correction_event, verify_jurisdiction_total,
    verify_package, EquationPass, RcountCoreError, RcountPackage, StatusEventType,
};
use rcount_io::{read_package_dir, verify_source_index, RcountIoError, RcountManifest};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const RCOUNT_AUDIT_TRANSCRIPT_VERSION: &str = "rcount-audit-transcript-v1";

#[derive(Debug, Error)]
pub enum RcountAuditError {
    #[error("io error: {0}")]
    Io(#[from] RcountIoError),
    #[error("core error: {0}")]
    Core(#[from] rcount_core::RcountCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("filesystem error: {0}")]
    Fs(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub equation_id: String,
    pub status: VerificationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_unit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationTranscript {
    pub transcript_version: String,
    pub verifier: String,
    pub status: VerificationStatus,
    pub package_content_hash: String,
    pub manifest_content_hash: String,
    pub checks: Vec<CheckResult>,
}

pub fn verify_package_dir(dir: &Path) -> VerificationTranscript {
    match read_package_dir(dir) {
        Ok((manifest, package)) => verify_loaded_package(dir, &manifest, &package),
        Err(err) => VerificationTranscript {
            transcript_version: RCOUNT_AUDIT_TRANSCRIPT_VERSION.to_string(),
            verifier: "rcount-audit".to_string(),
            status: VerificationStatus::Fail,
            package_content_hash: "<unavailable>".to_string(),
            manifest_content_hash: "<unavailable>".to_string(),
            checks: vec![CheckResult {
                equation_id: "package_read".to_string(),
                status: VerificationStatus::Fail,
                contest_id: None,
                reporting_unit_id: None,
                error: Some(err.to_string()),
            }],
        },
    }
}

pub fn write_verification_transcript(
    dir: &Path,
    transcript: &VerificationTranscript,
) -> Result<(), RcountAuditError> {
    let transcript_dir = dir.join("transcripts");
    fs::create_dir_all(&transcript_dir)?;
    let bytes = serde_json::to_vec_pretty(transcript)?;
    fs::write(transcript_dir.join("verify-transcript.json"), bytes)?;
    Ok(())
}

pub fn verify_and_write_transcript(dir: &Path) -> Result<VerificationTranscript, RcountAuditError> {
    let transcript = verify_package_dir(dir);
    write_verification_transcript(dir, &transcript)?;
    Ok(transcript)
}

fn verify_loaded_package(
    dir: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> VerificationTranscript {
    let package_hash = package_content_hash(package).unwrap_or_else(|err| format!("error:{err}"));
    let mut checks = Vec::new();

    match verify_package(package) {
        Ok(report) => {
            checks.extend(report.passed.into_iter().map(pass_result));
        }
        Err(err) => checks.push(CheckResult {
            equation_id: equation_id_for_core_error(&err).to_string(),
            status: VerificationStatus::Fail,
            contest_id: None,
            reporting_unit_id: None,
            error: Some(err.to_string()),
        }),
    }

    match verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries) {
        Ok(passes) => {
            checks.extend(passes.into_iter().map(pass_result));
        }
        Err(err) => checks.push(CheckResult {
            equation_id: "jurisdiction_contest_total".to_string(),
            status: VerificationStatus::Fail,
            contest_id: Some("syn-2024-mayor".to_string()),
            reporting_unit_id: Some("syn:jurisdiction:SYN".to_string()),
            error: Some(err.to_string()),
        }),
    }

    if package
        .status_events
        .iter()
        .any(|event| event.event_type == StatusEventType::Correction)
    {
        match verify_canvass_correction_event(package) {
            Ok(pass) => checks.push(pass_result(pass)),
            Err(err) => checks.push(CheckResult {
                equation_id: "canvass_correction_event".to_string(),
                status: VerificationStatus::Fail,
                contest_id: None,
                reporting_unit_id: None,
                error: Some(err.to_string()),
            }),
        }
    }

    match verify_source_index(dir) {
        Ok(source_checks) => {
            checks.extend(source_checks.into_iter().map(|source| CheckResult {
                equation_id: "source_hash_match".to_string(),
                status: VerificationStatus::Pass,
                contest_id: None,
                reporting_unit_id: Some(source.source_id),
                error: None,
            }));
        }
        Err(err) => checks.push(CheckResult {
            equation_id: "source_hash_match".to_string(),
            status: VerificationStatus::Fail,
            contest_id: None,
            reporting_unit_id: None,
            error: Some(err.to_string()),
        }),
    }

    let status = if checks
        .iter()
        .all(|check| check.status == VerificationStatus::Pass)
    {
        VerificationStatus::Pass
    } else {
        VerificationStatus::Fail
    };

    VerificationTranscript {
        transcript_version: RCOUNT_AUDIT_TRANSCRIPT_VERSION.to_string(),
        verifier: "rcount-audit".to_string(),
        status,
        package_content_hash: package_hash,
        manifest_content_hash: manifest.content_hash.clone(),
        checks,
    }
}

fn pass_result(pass: EquationPass) -> CheckResult {
    CheckResult {
        equation_id: pass.equation_id,
        status: VerificationStatus::Pass,
        contest_id: Some(pass.contest_id),
        reporting_unit_id: Some(pass.reporting_unit_id),
        error: None,
    }
}

fn equation_id_for_core_error(err: &RcountCoreError) -> &'static str {
    match err {
        RcountCoreError::MissingBatch { .. }
        | RcountCoreError::DuplicateBatchId { .. }
        | RcountCoreError::BatchSummaryTotalMismatch { .. } => "batch_summary_total",
        RcountCoreError::AcceptedBallotsMismatch { .. } => "accepted_ballots",
        RcountCoreError::DuplicateLineageId { .. }
        | RcountCoreError::MissingPriorLineageUnit { .. }
        | RcountCoreError::MissingCurrentLineageUnit { .. }
        | RcountCoreError::InvalidSplitLineage { .. }
        | RcountCoreError::InvalidMergeLineage { .. } => "lineage_conservation",
        RcountCoreError::DuplicateProofId { .. }
        | RcountCoreError::ChoiceBearingProof { .. }
        | RcountCoreError::LinkableVoterProof { .. }
        | RcountCoreError::InvalidProofTokenHash { .. } => "proof_privacy_gate",
        RcountCoreError::DuplicateStatusEventId { .. }
        | RcountCoreError::NoStatusTransition { .. }
        | RcountCoreError::IncompleteStatusEvent { .. } => "status_event_declared",
        RcountCoreError::MissingCanvassCorrectionEvent
        | RcountCoreError::MissingStatusSummaries { .. } => "canvass_correction_event",
        _ => "contest_selection_sum",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcount_core::{
        synthetic_bad_lineage_package, synthetic_canvass_correction_package,
        synthetic_choice_bearing_proof_package, synthetic_mail_batch_added_package,
        synthetic_missing_batch_package, synthetic_precinct_split_lineage_package,
        synthetic_privacy_inclusion_package, synthetic_summary_basic_package,
    };
    use rcount_io::{
        synthetic_canvass_correction_manifest, synthetic_summary_basic_manifest, write_package_dir,
    };

    #[test]
    fn valid_summary_basic_produces_pass_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(transcript.checks.len(), 5);
        assert_eq!(
            transcript.package_content_hash,
            transcript.manifest_content_hash
        );
    }

    #[test]
    fn tampered_manifest_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let manifest_path = tmp.path().join("manifest.json");
        let mut raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert_eq!(transcript.checks[0].equation_id, "package_read");
        assert!(transcript.checks[0]
            .error
            .as_ref()
            .unwrap()
            .contains("content_hash mismatch"));
    }

    #[test]
    fn bad_arithmetic_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let mut package = synthetic_summary_basic_package();
        package.summaries[0].counted_ballots += 1;
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "contest_selection_sum"
                && check.status == VerificationStatus::Fail));
    }

    #[test]
    fn tampered_source_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path()
                .join("sources")
                .join("synthetic-summary-export.json"),
            br#"{"tampered":true}"#,
        )
        .unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "source_hash_match"
                && check.status == VerificationStatus::Fail));
    }

    #[test]
    fn missing_source_hash_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path().join("sources").join("source-index.json"),
            br#"{"sources":[]}"#,
        )
        .unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "source_hash_match"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("source index is empty"))));
    }

    #[test]
    fn canvass_correction_produces_event_correlation_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_canvass_correction_package();
        let manifest = synthetic_canvass_correction_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "canvass_correction_event"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn mail_batch_added_produces_batch_correlation_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_mail_batch_added_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(
            transcript
                .checks
                .iter()
                .filter(|check| check.equation_id == "batch_summary_total"
                    && check.status == VerificationStatus::Pass)
                .count(),
            3
        );
    }

    #[test]
    fn missing_batch_produces_batch_correlation_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_missing_batch_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "batch_summary_total"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("references missing batch id"))));
    }

    #[test]
    fn precinct_split_lineage_produces_lineage_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_precinct_split_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(
            transcript
                .checks
                .iter()
                .filter(|check| check.equation_id == "lineage_conservation"
                    && check.status == VerificationStatus::Pass)
                .count(),
            2
        );
    }

    #[test]
    fn bad_lineage_produces_lineage_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "lineage_conservation"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("missing current reporting unit"))));
    }

    #[test]
    fn privacy_inclusion_produces_privacy_gate_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_privacy_inclusion_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "proof_privacy_gate"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn choice_bearing_proof_produces_privacy_gate_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_choice_bearing_proof_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "proof_privacy_gate"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("exposes candidate selections"))));
    }

    #[test]
    fn docs_summary_basic_transcript_verifies_when_present() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("examples")
            .join("rcount-golden-packages")
            .join("summary-basic");
        if dir.exists() {
            let transcript = verify_package_dir(&dir);
            assert_eq!(transcript.status, VerificationStatus::Pass);
        }
    }
}
