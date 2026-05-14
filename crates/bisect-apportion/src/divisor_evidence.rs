use crate::divisor_methods::{apportionment_divisor, RoundingRule};
use crate::paradoxes::check_alabama_paradox;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const J_DIVISOR_EVIDENCE_MANIFEST_VERSION: &str = "j-divisor-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivisorEvidenceManifest {
    pub schema_version: String,
    pub package_id: String,
    pub papers: Vec<String>,
    pub fixture_file: EvidenceFile,
    pub verifier_path: String,
    pub verification_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceFile {
    pub path: String,
    pub sha256: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivisorFixtureSet {
    pub fixture_version: String,
    pub populations: Vec<PopulationRow>,
    pub house_size: u32,
    pub expected_allocations: Vec<ExpectedAllocation>,
    pub paradox_check: ParadoxCheck,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationRow {
    pub name: String,
    pub population: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedAllocation {
    pub method: String,
    pub seats: Vec<SeatRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatRow {
    pub name: String,
    pub seats: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParadoxCheck {
    pub house_sizes: Vec<u32>,
    pub methods: Vec<String>,
    pub expected_alabama_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DivisorEvidenceError {
    #[error("unsupported schema_version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("path is not package-relative and portable: {0}")]
    InvalidPath(String),
    #[error("sha256 must be 64 lowercase hex characters: {0}")]
    InvalidSha256(String),
    #[error("could not read referenced file {path}: {message}")]
    FileRead { path: String, message: String },
    #[error("hash mismatch for {path}: declared {declared}, computed {computed}")]
    FileHashMismatch {
        path: String,
        declared: String,
        computed: String,
    },
    #[error("unknown divisor method: {0}")]
    UnknownMethod(String),
    #[error("allocation mismatch for {method}/{state}: expected {expected}, computed {computed}")]
    AllocationMismatch {
        method: String,
        state: String,
        expected: u32,
        computed: u32,
    },
    #[error("unexpected Alabama paradox events for {method}: {events:?}")]
    UnexpectedParadoxEvents { method: String, events: Vec<String> },
}

impl DivisorEvidenceManifest {
    pub fn validate(&self) -> Result<(), DivisorEvidenceError> {
        if self.schema_version != J_DIVISOR_EVIDENCE_MANIFEST_VERSION {
            return Err(DivisorEvidenceError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        require_non_empty("package_id", &self.package_id)?;
        require_non_empty_vec("papers", &self.papers)?;
        validate_file(&self.fixture_file)?;
        validate_portable_path(&self.verifier_path)?;
        require_non_empty_vec("verification_commands", &self.verification_commands)?;
        for command in &self.verification_commands {
            require_non_empty("verification_commands[]", command)?;
        }
        Ok(())
    }

    pub fn validate_package(
        &self,
        package_root: impl AsRef<Path>,
    ) -> Result<(), DivisorEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            DivisorEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(DivisorEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl DivisorFixtureSet {
    pub fn verify(&self) -> Result<(), DivisorEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        require_non_empty_vec("populations", &self.populations)?;
        require_non_empty_vec("expected_allocations", &self.expected_allocations)?;
        require_non_empty_vec("paradox_check.house_sizes", &self.paradox_check.house_sizes)?;
        require_non_empty_vec("paradox_check.methods", &self.paradox_check.methods)?;

        let populations: Vec<(String, u64)> = self
            .populations
            .iter()
            .map(|row| (row.name.clone(), row.population))
            .collect();

        for expected in &self.expected_allocations {
            let rule = parse_rule(&expected.method)?;
            let computed: HashMap<String, u32> =
                apportionment_divisor(&populations, self.house_size, rule)
                    .into_iter()
                    .collect();
            for row in &expected.seats {
                let got = computed.get(&row.name).copied().unwrap_or(0);
                if got != row.seats {
                    return Err(DivisorEvidenceError::AllocationMismatch {
                        method: expected.method.clone(),
                        state: row.name.clone(),
                        expected: row.seats,
                        computed: got,
                    });
                }
            }
        }

        let expected_events: Vec<String> = self.paradox_check.expected_alabama_events.clone();
        for method in &self.paradox_check.methods {
            let rule = parse_rule(method)?;
            let events: Vec<String> =
                check_alabama_paradox(&populations, &self.paradox_check.house_sizes, rule)
                    .into_iter()
                    .map(|(state, after, before)| format!("{state}:{before}->{after}"))
                    .collect();
            if events != expected_events {
                return Err(DivisorEvidenceError::UnexpectedParadoxEvents {
                    method: method.clone(),
                    events,
                });
            }
        }

        Ok(())
    }
}

fn parse_rule(value: &str) -> Result<RoundingRule, DivisorEvidenceError> {
    match value {
        "huntington-hill" => Ok(RoundingRule::HuntingtonHill),
        "webster" => Ok(RoundingRule::Webster),
        "adams" => Ok(RoundingRule::Adams),
        "jefferson" => Ok(RoundingRule::Jefferson),
        other => Err(DivisorEvidenceError::UnknownMethod(other.to_string())),
    }
}

fn validate_file(file: &EvidenceFile) -> Result<(), DivisorEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), DivisorEvidenceError> {
    if value.trim().is_empty() {
        Err(DivisorEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), DivisorEvidenceError> {
    if value.is_empty() {
        Err(DivisorEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), DivisorEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(DivisorEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), DivisorEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(DivisorEvidenceError::InvalidPath(path.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("examples")
            .join("j-apportionment-evidence-packages")
            .join("divisor-method-smoke")
    }

    #[test]
    fn divisor_method_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: DivisorEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn divisor_method_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: DivisorFixtureSet = serde_json::from_slice(
            &fs::read(root.join("divisor-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_divisor_fixture_rejected() {
        let root = fixture_root();
        let mut fixtures: DivisorFixtureSet = serde_json::from_slice(
            &fs::read(root.join("divisor-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.expected_allocations[0].seats[0].seats += 1;
        assert!(matches!(
            fixtures.verify(),
            Err(DivisorEvidenceError::AllocationMismatch { .. })
        ));
    }
}
