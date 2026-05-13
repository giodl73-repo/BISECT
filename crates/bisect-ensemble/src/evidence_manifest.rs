use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const G_ENSEMBLE_EVIDENCE_MANIFEST_VERSION: &str = "g-ensemble-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidencePackageStatus {
    Active,
    MissingEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceFileRole {
    ExternalTrace,
    BisectPlan,
    RctxContext,
    ElectionInput,
    MetricOutput,
    Diagnostic,
    Manifest,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFile {
    pub path: String,
    pub sha256: String,
    pub role: EvidenceFileRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimReference {
    pub paper: String,
    pub claim: String,
    pub required_roles: Vec<EvidenceFileRole>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingEvidence {
    pub role: EvidenceFileRole,
    pub reason: String,
    pub next_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GEnsembleEvidenceManifest {
    pub schema_version: String,
    pub package_id: String,
    pub status: EvidencePackageStatus,
    pub papers: Vec<String>,
    pub claims: Vec<ClaimReference>,
    pub files: Vec<EvidenceFile>,
    pub verifier_path: String,
    pub verification_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_evidence: Vec<MissingEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvidenceManifestError {
    #[error("unsupported schema_version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("path is not package-relative and portable: {0}")]
    InvalidPath(String),
    #[error("sha256 must be 64 lowercase hex characters: {0}")]
    InvalidSha256(String),
    #[error("active evidence packages require at least one file")]
    ActivePackageMissingFiles,
    #[error("active evidence packages require at least one verification command")]
    ActivePackageMissingVerification,
    #[error("active evidence packages must not carry missing evidence")]
    ActivePackageHasMissingEvidence,
    #[error("missing-evidence packages must explain at least one missing artifact")]
    MissingPackageWithoutGap,
    #[error("claim {index} must require at least one evidence role")]
    ClaimWithoutRequiredRoles { index: usize },
    #[error("could not read referenced file {path}: {message}")]
    FileRead { path: String, message: String },
    #[error("hash mismatch for {path}: declared {declared}, computed {computed}")]
    FileHashMismatch {
        path: String,
        declared: String,
        computed: String,
    },
}

impl GEnsembleEvidenceManifest {
    pub fn validate(&self) -> Result<(), EvidenceManifestError> {
        if self.schema_version != G_ENSEMBLE_EVIDENCE_MANIFEST_VERSION {
            return Err(EvidenceManifestError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        require_non_empty("package_id", &self.package_id)?;
        require_non_empty_vec("papers", &self.papers)?;
        require_non_empty_vec("claims", &self.claims)?;
        validate_portable_path(&self.verifier_path)?;

        for (index, claim) in self.claims.iter().enumerate() {
            require_non_empty("claim.paper", &claim.paper)?;
            require_non_empty("claim.claim", &claim.claim)?;
            if claim.required_roles.is_empty() {
                return Err(EvidenceManifestError::ClaimWithoutRequiredRoles { index });
            }
        }

        for file in &self.files {
            validate_portable_path(&file.path)?;
            validate_sha256(&file.sha256)?;
        }

        for command in &self.verification_commands {
            require_non_empty("verification_commands[]", command)?;
        }

        for missing in &self.missing_evidence {
            require_non_empty("missing_evidence.reason", &missing.reason)?;
            require_non_empty("missing_evidence.next_step", &missing.next_step)?;
        }

        match self.status {
            EvidencePackageStatus::Active => {
                if self.files.is_empty() {
                    return Err(EvidenceManifestError::ActivePackageMissingFiles);
                }
                if self.verification_commands.is_empty() {
                    return Err(EvidenceManifestError::ActivePackageMissingVerification);
                }
                if !self.missing_evidence.is_empty() {
                    return Err(EvidenceManifestError::ActivePackageHasMissingEvidence);
                }
            }
            EvidencePackageStatus::MissingEvidence => {
                if self.missing_evidence.is_empty() {
                    return Err(EvidenceManifestError::MissingPackageWithoutGap);
                }
            }
        }

        Ok(())
    }

    pub fn validate_referenced_file_hashes(
        &self,
        package_root: impl AsRef<Path>,
    ) -> Result<(), EvidenceManifestError> {
        self.validate()?;
        let package_root = package_root.as_ref();
        for file in &self.files {
            let path = package_root.join(&file.path);
            let bytes = fs::read(&path).map_err(|error| EvidenceManifestError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            })?;
            let computed = format!("{:x}", Sha256::digest(&bytes));
            if computed != file.sha256 {
                return Err(EvidenceManifestError::FileHashMismatch {
                    path: file.path.clone(),
                    declared: file.sha256.clone(),
                    computed,
                });
            }
        }
        Ok(())
    }
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), EvidenceManifestError> {
    if value.trim().is_empty() {
        Err(EvidenceManifestError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), EvidenceManifestError> {
    if value.is_empty() {
        Err(EvidenceManifestError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), EvidenceManifestError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(EvidenceManifestError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), EvidenceManifestError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(EvidenceManifestError::InvalidPath(path.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash() -> String {
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    }

    fn claim() -> ClaimReference {
        ClaimReference {
            paper: "G.1".to_string(),
            claim: "compactness percentile is trace-backed".to_string(),
            required_roles: vec![
                EvidenceFileRole::ExternalTrace,
                EvidenceFileRole::MetricOutput,
                EvidenceFileRole::RctxContext,
            ],
        }
    }

    #[test]
    fn active_manifest_requires_hash_bound_files_and_verifier() {
        let manifest = GEnsembleEvidenceManifest {
            schema_version: G_ENSEMBLE_EVIDENCE_MANIFEST_VERSION.to_string(),
            package_id: "G.1+synthetic-smoke".to_string(),
            status: EvidencePackageStatus::Active,
            papers: vec!["G.1".to_string()],
            claims: vec![claim()],
            files: vec![EvidenceFile {
                path: "traces/synthetic-recom.json".to_string(),
                sha256: hash(),
                role: EvidenceFileRole::ExternalTrace,
                description: None,
            }],
            verifier_path: "crates/bisect-ensemble/src/evidence_manifest.rs".to_string(),
            verification_commands: vec!["cargo test -p bisect-ensemble".to_string()],
            missing_evidence: vec![],
        };

        assert_eq!(manifest.validate(), Ok(()));
    }

    #[test]
    fn missing_evidence_manifest_must_explain_gap() {
        let manifest = GEnsembleEvidenceManifest {
            schema_version: G_ENSEMBLE_EVIDENCE_MANIFEST_VERSION.to_string(),
            package_id: "G.1+external-traces-missing".to_string(),
            status: EvidencePackageStatus::MissingEvidence,
            papers: vec!["G.1".to_string()],
            claims: vec![claim()],
            files: vec![],
            verifier_path: "crates/bisect-ensemble/src/evidence_manifest.rs".to_string(),
            verification_commands: vec![],
            missing_evidence: vec![MissingEvidence {
                role: EvidenceFileRole::ExternalTrace,
                reason: "No archived GerryChain trace package was found by pulse 01 scout."
                    .to_string(),
                next_step: "Add trace manifest or rerun external sampler in a later pulse."
                    .to_string(),
            }],
        };

        assert_eq!(manifest.validate(), Ok(()));
    }

    #[test]
    fn rejects_bad_hash_shape() {
        let mut manifest = GEnsembleEvidenceManifest {
            schema_version: G_ENSEMBLE_EVIDENCE_MANIFEST_VERSION.to_string(),
            package_id: "G.1+bad-hash".to_string(),
            status: EvidencePackageStatus::Active,
            papers: vec!["G.1".to_string()],
            claims: vec![claim()],
            files: vec![EvidenceFile {
                path: "traces/synthetic-recom.json".to_string(),
                sha256: "ABC".to_string(),
                role: EvidenceFileRole::ExternalTrace,
                description: None,
            }],
            verifier_path: "crates/bisect-ensemble/src/evidence_manifest.rs".to_string(),
            verification_commands: vec!["cargo test -p bisect-ensemble".to_string()],
            missing_evidence: vec![],
        };

        assert_eq!(
            manifest.validate(),
            Err(EvidenceManifestError::InvalidSha256("ABC".to_string()))
        );

        manifest.files[0].sha256 = hash();
        manifest.files[0].path = r"C:\trace.json".to_string();
        assert_eq!(
            manifest.validate(),
            Err(EvidenceManifestError::InvalidPath(
                r"C:\trace.json".to_string()
            ))
        );
    }

    #[test]
    fn active_manifest_rejects_missing_verification_command() {
        let manifest = GEnsembleEvidenceManifest {
            schema_version: G_ENSEMBLE_EVIDENCE_MANIFEST_VERSION.to_string(),
            package_id: "G.1+no-verifier".to_string(),
            status: EvidencePackageStatus::Active,
            papers: vec!["G.1".to_string()],
            claims: vec![claim()],
            files: vec![EvidenceFile {
                path: "traces/synthetic-recom.json".to_string(),
                sha256: hash(),
                role: EvidenceFileRole::ExternalTrace,
                description: None,
            }],
            verifier_path: "crates/bisect-ensemble/src/evidence_manifest.rs".to_string(),
            verification_commands: vec![],
            missing_evidence: vec![],
        };

        assert_eq!(
            manifest.validate(),
            Err(EvidenceManifestError::ActivePackageMissingVerification)
        );
    }

    #[test]
    fn fixture_manifest_hashes_match_referenced_files() {
        let manifest: GEnsembleEvidenceManifest = serde_json::from_str(include_str!(
            "fixtures/g_ensemble/active-smoke/manifest.json"
        ))
        .expect("fixture must parse");
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/g_ensemble/active-smoke");

        assert_eq!(manifest.validate_referenced_file_hashes(root), Ok(()));
    }

    #[test]
    fn negative_fixture_rejects_hash_mismatch() {
        let manifest: GEnsembleEvidenceManifest =
            serde_json::from_str(include_str!("fixtures/g_ensemble/bad-hash/manifest.json"))
                .expect("fixture must parse");
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/g_ensemble/bad-hash");

        assert!(matches!(
            manifest.validate_referenced_file_hashes(root),
            Err(EvidenceManifestError::FileHashMismatch { .. })
        ));
    }

    #[test]
    fn missing_evidence_package_fixture_validates() {
        let manifest: GEnsembleEvidenceManifest = serde_json::from_str(include_str!(
            "../../../docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence/manifest.json"
        ))
        .expect("fixture must parse");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence");

        assert_eq!(manifest.validate_referenced_file_hashes(root), Ok(()));
    }
}
