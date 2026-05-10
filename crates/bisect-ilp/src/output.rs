//! Stable JSON output envelope for exact/ILP workflows.

use crate::formulation::IlpFormulation;
use crate::result::{IlpResult, SolverStatus};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub const ILP_SOLVE_REPORT_SCHEMA_VERSION: &str = "ilp-solve-report-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IlpSolveReport {
    pub schema_version: String,
    pub audit_summary: IlpSolveAuditSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_artifact: Option<IlpModelArtifact>,
    pub formulation: IlpFormulation,
    pub result: IlpResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IlpSolveAuditSummary {
    pub outcome: String,
    pub proof_status: String,
    pub has_plan: bool,
    pub has_model_artifact: bool,
    pub has_branch_and_cut_certificate: bool,
    pub fallback_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_search_strategy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IlpModelArtifact {
    pub format: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedModelArtifact {
    pub format: String,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ModelArtifactVerificationError {
    #[error("read solve report {path}: {source}")]
    ReadReport {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parse solve report {path}: {source}")]
    ParseReport {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("solve report has no model_artifact block: {path}")]
    MissingModelArtifact { path: PathBuf },
    #[error("model artifact path must be relative: {path}")]
    AbsoluteModelPath { path: String },
    #[error("model artifact path may not contain parent components: {path}")]
    ParentModelPath { path: String },
    #[error("read model artifact {path}: {source}")]
    ReadModel {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("model artifact hash mismatch for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
}

impl IlpSolveReport {
    pub fn new(formulation: IlpFormulation, result: IlpResult) -> Self {
        let audit_summary = IlpSolveAuditSummary::from_result(&result, false);
        Self {
            schema_version: ILP_SOLVE_REPORT_SCHEMA_VERSION.to_string(),
            audit_summary,
            model_artifact: None,
            formulation,
            result,
        }
    }

    pub fn with_model_artifact(
        formulation: IlpFormulation,
        result: IlpResult,
        model_artifact: IlpModelArtifact,
    ) -> Self {
        let audit_summary = IlpSolveAuditSummary::from_result(&result, true);
        Self {
            schema_version: ILP_SOLVE_REPORT_SCHEMA_VERSION.to_string(),
            audit_summary,
            model_artifact: Some(model_artifact),
            formulation,
            result,
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl IlpSolveAuditSummary {
    pub fn from_result(result: &IlpResult, has_model_artifact: bool) -> Self {
        let has_plan = result.plan.is_some();
        let has_branch_and_cut_certificate = result.branch_and_cut.is_some();
        let exact_search_strategy = result
            .branch_and_cut
            .as_ref()
            .and_then(|cert| cert.exact_search.as_ref())
            .map(|stats| stats.search_strategy.clone());

        let (outcome, proof_status) = match &result.status {
            SolverStatus::Optimal if exact_search_strategy.is_some() => {
                ("exact-plan", "proved-optimal")
            }
            SolverStatus::Optimal => ("solver-plan", "proved-optimal"),
            SolverStatus::TimeoutWithSolution => ("incumbent-plan", "timed-out"),
            SolverStatus::GapReached { .. } => ("incumbent-plan", "bounded-gap"),
            SolverStatus::Infeasible => ("infeasible", "proved-infeasible"),
            SolverStatus::FallbackMetis => ("fallback", "not-ilp-solved"),
            SolverStatus::FormulationOnly => ("formulation-only", "not-solved"),
            SolverStatus::SubprocessNotImplemented => ("not-implemented", "not-solved"),
            SolverStatus::BranchAndCutNotImplemented => ("not-implemented", "not-solved"),
        };

        Self {
            outcome: outcome.to_string(),
            proof_status: proof_status.to_string(),
            has_plan,
            has_model_artifact,
            has_branch_and_cut_certificate,
            fallback_required: !has_plan
                && !matches!(
                    result.status,
                    SolverStatus::Infeasible | SolverStatus::FallbackMetis
                ),
            exact_search_strategy,
        }
    }
}

pub fn solve_report_json(
    formulation: IlpFormulation,
    result: IlpResult,
) -> Result<String, serde_json::Error> {
    IlpSolveReport::new(formulation, result).to_json_string()
}

pub fn solve_report_json_with_model_artifact(
    formulation: IlpFormulation,
    result: IlpResult,
    model_artifact: IlpModelArtifact,
) -> Result<String, serde_json::Error> {
    IlpSolveReport::with_model_artifact(formulation, result, model_artifact).to_json_string()
}

pub fn verify_model_artifact_for_report<P: AsRef<Path>>(
    report_path: P,
) -> Result<VerifiedModelArtifact, ModelArtifactVerificationError> {
    let report_path = report_path.as_ref();
    let bytes = std::fs::read(report_path).map_err(|source| {
        ModelArtifactVerificationError::ReadReport {
            path: report_path.to_path_buf(),
            source,
        }
    })?;
    let report: IlpSolveReport = serde_json::from_slice(&bytes).map_err(|source| {
        ModelArtifactVerificationError::ParseReport {
            path: report_path.to_path_buf(),
            source,
        }
    })?;
    let artifact = report.model_artifact.ok_or_else(|| {
        ModelArtifactVerificationError::MissingModelArtifact {
            path: report_path.to_path_buf(),
        }
    })?;
    let model_path = resolve_relative_model_path(report_path, &artifact.path)?;
    let actual =
        sha256_file(&model_path).map_err(|source| ModelArtifactVerificationError::ReadModel {
            path: model_path.clone(),
            source,
        })?;
    if actual != artifact.sha256 {
        return Err(ModelArtifactVerificationError::HashMismatch {
            path: model_path,
            expected: artifact.sha256,
            actual,
        });
    }
    Ok(VerifiedModelArtifact {
        format: artifact.format,
        path: model_path,
        sha256: actual,
    })
}

fn resolve_relative_model_path(
    report_path: &Path,
    artifact_path: &str,
) -> Result<PathBuf, ModelArtifactVerificationError> {
    let path = Path::new(artifact_path);
    if path.is_absolute() {
        return Err(ModelArtifactVerificationError::AbsoluteModelPath {
            path: artifact_path.to_string(),
        });
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(ModelArtifactVerificationError::ParentModelPath {
            path: artifact_path.to_string(),
        });
    }
    let parent = report_path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parent.join(path))
}

fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificates::BranchAndCutMode;
    use crate::formulation::build_formulation;
    use crate::solver::{solve, IlpSolver};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn path_5() -> Vec<Vec<usize>> {
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]]
    }

    fn temp_test_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("bisect-ilp-{name}-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn formulation_result() -> (IlpFormulation, IlpResult) {
        let adjacency = path_5();
        let pop = vec![100, 100, 100, 150, 150];
        let formulation = build_formulation(&adjacency, &pop, 2, 0.005);
        let result = solve(
            &formulation,
            &adjacency,
            &pop,
            2,
            0.005,
            IlpSolver::FormulationOnly,
            0.01,
        );
        (formulation, result)
    }

    #[test]
    fn solve_report_json_round_trips() {
        let adjacency = path_5();
        let pop = vec![100, 100, 100, 150, 150];
        let formulation = build_formulation(&adjacency, &pop, 2, 0.005);
        let result = solve(
            &formulation,
            &adjacency,
            &pop,
            2,
            0.005,
            IlpSolver::BranchAndCut {
                mode: BranchAndCutMode::IterativeSeparation,
                incumbent_assignment: Some(vec![0, 1, 0, 1, 1]),
                solver_name: Some("highs".to_string()),
            },
            0.01,
        );
        let report = IlpSolveReport::new(formulation, result);
        let json = report.to_json_string().unwrap();
        assert!(json.contains(ILP_SOLVE_REPORT_SCHEMA_VERSION));
        assert!(json.contains(r#""status": "optimal""#));
        assert!(json.contains(r#""outcome": "exact-plan""#));
        assert!(json.contains(r#""proof_status": "proved-optimal""#));
        let decoded: IlpSolveReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);
        assert_eq!(decoded.audit_summary.outcome, "exact-plan");
        assert_eq!(decoded.audit_summary.proof_status, "proved-optimal");
        assert_eq!(
            decoded.audit_summary.exact_search_strategy.as_deref(),
            Some("k2-branch-and-bound")
        );
        assert!(!decoded.audit_summary.fallback_required);
    }

    #[test]
    fn solve_report_model_artifact_round_trips() {
        let (formulation, result) = formulation_result();
        let artifact = IlpModelArtifact {
            format: "cplex-lp".to_string(),
            path: "node_root.lp".to_string(),
            sha256: "a".repeat(64),
        };
        let report = IlpSolveReport::with_model_artifact(formulation, result, artifact);
        let json = report.to_json_string().unwrap();
        assert!(json.contains(r#""format": "cplex-lp""#));
        let decoded: IlpSolveReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);
        assert_eq!(decoded.audit_summary.outcome, "formulation-only");
        assert_eq!(decoded.audit_summary.proof_status, "not-solved");
        assert!(decoded.audit_summary.has_model_artifact);
        assert!(decoded.audit_summary.fallback_required);
    }

    #[test]
    fn verify_model_artifact_for_report_accepts_matching_hash() {
        let dir = temp_test_dir("verify-ok");
        let model_path = dir.join("node_root.lp");
        std::fs::write(&model_path, "Minimize\n obj: 0\nEnd\n").unwrap();
        let expected_hash = sha256_file(&model_path).unwrap();
        let (formulation, result) = formulation_result();
        let report = IlpSolveReport::with_model_artifact(
            formulation,
            result,
            IlpModelArtifact {
                format: "cplex-lp".to_string(),
                path: "node_root.lp".to_string(),
                sha256: expected_hash.clone(),
            },
        );
        let report_path = dir.join("node_root.json");
        std::fs::write(&report_path, report.to_json_string().unwrap()).unwrap();

        let verified = verify_model_artifact_for_report(&report_path).unwrap();
        assert_eq!(verified.format, "cplex-lp");
        assert_eq!(verified.path, model_path);
        assert_eq!(verified.sha256, expected_hash);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn verify_model_artifact_for_report_rejects_hash_mismatch() {
        let dir = temp_test_dir("verify-mismatch");
        let model_path = dir.join("node_root.lp");
        std::fs::write(&model_path, "Minimize\n obj: 0\nEnd\n").unwrap();
        let (formulation, result) = formulation_result();
        let report = IlpSolveReport::with_model_artifact(
            formulation,
            result,
            IlpModelArtifact {
                format: "cplex-lp".to_string(),
                path: "node_root.lp".to_string(),
                sha256: "0".repeat(64),
            },
        );
        let report_path = dir.join("node_root.json");
        std::fs::write(&report_path, report.to_json_string().unwrap()).unwrap();

        let err = verify_model_artifact_for_report(&report_path).unwrap_err();
        assert!(matches!(
            err,
            ModelArtifactVerificationError::HashMismatch { .. }
        ));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn verify_model_artifact_for_report_rejects_parent_path() {
        let dir = temp_test_dir("verify-parent");
        let (formulation, result) = formulation_result();
        let report = IlpSolveReport::with_model_artifact(
            formulation,
            result,
            IlpModelArtifact {
                format: "cplex-lp".to_string(),
                path: "../node_root.lp".to_string(),
                sha256: "0".repeat(64),
            },
        );
        let report_path = dir.join("node_root.json");
        std::fs::write(&report_path, report.to_json_string().unwrap()).unwrap();

        let err = verify_model_artifact_for_report(&report_path).unwrap_err();
        assert!(matches!(
            err,
            ModelArtifactVerificationError::ParentModelPath { .. }
        ));
        let _ = std::fs::remove_dir_all(dir);
    }
}
