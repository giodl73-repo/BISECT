use crate::huntington_hill::huntington_hill;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const J_APPORTIONMENT_EVIDENCE_MANIFEST_VERSION: &str = "j-apportionment-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApportionmentEvidenceManifest {
    pub schema_version: String,
    pub package_id: String,
    pub census_table: String,
    pub source_url: String,
    pub source_sha256: String,
    pub extracted_files: Vec<EvidenceFile>,
    pub verifier_path: String,
    pub verification_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFile {
    pub path: String,
    pub sha256: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedApportionmentTable {
    pub dataset: String,
    pub source_url: String,
    pub source_sha256: String,
    pub house_size: u32,
    pub rows: Vec<ApportionmentRow>,
    pub total_apportionment_population: u64,
    pub total_representatives: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApportionmentRow {
    pub state: String,
    pub apportionment_population: u64,
    pub representatives: u32,
    pub change_from_2010: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApportionmentEvidenceError {
    #[error("unsupported schema_version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("sha256 must be 64 lowercase hex characters: {0}")]
    InvalidSha256(String),
    #[error("path is not package-relative and portable: {0}")]
    InvalidPath(String),
    #[error("could not read referenced file {path}: {message}")]
    FileRead { path: String, message: String },
    #[error("hash mismatch for {path}: declared {declared}, computed {computed}")]
    FileHashMismatch {
        path: String,
        declared: String,
        computed: String,
    },
    #[error("source hash mismatch between manifest and extracted table")]
    SourceHashMismatch,
    #[error("expected 50 states, found {0}")]
    WrongStateCount(usize),
    #[error("official representative total {0} does not equal house size {1}")]
    SeatTotalMismatch(u32, u32),
    #[error("official population total {declared} does not match row sum {computed}")]
    PopulationTotalMismatch { declared: u64, computed: u64 },
    #[error("Huntington-Hill mismatch for {state}: expected {expected}, computed {computed}")]
    HuntingtonHillMismatch {
        state: String,
        expected: u32,
        computed: u32,
    },
}

impl ApportionmentEvidenceManifest {
    pub fn validate(&self) -> Result<(), ApportionmentEvidenceError> {
        if self.schema_version != J_APPORTIONMENT_EVIDENCE_MANIFEST_VERSION {
            return Err(ApportionmentEvidenceError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        require_non_empty("package_id", &self.package_id)?;
        require_non_empty("census_table", &self.census_table)?;
        require_non_empty("source_url", &self.source_url)?;
        validate_sha256(&self.source_sha256)?;
        validate_portable_path(&self.verifier_path)?;
        if self.extracted_files.is_empty() {
            return Err(ApportionmentEvidenceError::EmptyField {
                field: "extracted_files",
            });
        }
        if self.verification_commands.is_empty() {
            return Err(ApportionmentEvidenceError::EmptyField {
                field: "verification_commands",
            });
        }
        for file in &self.extracted_files {
            validate_portable_path(&file.path)?;
            validate_sha256(&file.sha256)?;
            require_non_empty("extracted_files[].role", &file.role)?;
        }
        for command in &self.verification_commands {
            require_non_empty("verification_commands[]", command)?;
        }
        Ok(())
    }

    pub fn validate_package(
        &self,
        package_root: impl AsRef<Path>,
    ) -> Result<(), ApportionmentEvidenceError> {
        self.validate()?;
        let package_root = package_root.as_ref();
        for file in &self.extracted_files {
            let path = package_root.join(&file.path);
            let bytes = fs::read(&path).map_err(|error| ApportionmentEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            })?;
            let computed = format!("{:x}", Sha256::digest(&bytes));
            if computed != file.sha256 {
                return Err(ApportionmentEvidenceError::FileHashMismatch {
                    path: file.path.clone(),
                    declared: file.sha256.clone(),
                    computed,
                });
            }
        }
        Ok(())
    }
}

impl ExtractedApportionmentTable {
    pub fn verify_huntington_hill(
        &self,
        manifest: &ApportionmentEvidenceManifest,
    ) -> Result<(), ApportionmentEvidenceError> {
        if self.source_sha256 != manifest.source_sha256 {
            return Err(ApportionmentEvidenceError::SourceHashMismatch);
        }
        if self.rows.len() != 50 {
            return Err(ApportionmentEvidenceError::WrongStateCount(self.rows.len()));
        }
        let official_total: u32 = self.rows.iter().map(|row| row.representatives).sum();
        if official_total != self.house_size {
            return Err(ApportionmentEvidenceError::SeatTotalMismatch(
                official_total,
                self.house_size,
            ));
        }
        let population_total: u64 = self
            .rows
            .iter()
            .map(|row| row.apportionment_population)
            .sum();
        if population_total != self.total_apportionment_population {
            return Err(ApportionmentEvidenceError::PopulationTotalMismatch {
                declared: self.total_apportionment_population,
                computed: population_total,
            });
        }

        let populations: HashMap<String, u64> = self
            .rows
            .iter()
            .map(|row| (row.state.clone(), row.apportionment_population))
            .collect();
        let computed = huntington_hill(&populations, self.house_size);
        let official: BTreeMap<String, u32> = self
            .rows
            .iter()
            .map(|row| (row.state.clone(), row.representatives))
            .collect();

        for (state, expected) in official {
            let actual = computed[&state];
            if actual != expected {
                return Err(ApportionmentEvidenceError::HuntingtonHillMismatch {
                    state,
                    expected,
                    computed: actual,
                });
            }
        }
        Ok(())
    }
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ApportionmentEvidenceError> {
    if value.trim().is_empty() {
        Err(ApportionmentEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), ApportionmentEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ApportionmentEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), ApportionmentEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        Err(ApportionmentEvidenceError::InvalidPath(path.to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ApportionmentEvidenceManifest {
        serde_json::from_str(include_str!(
            "../../../docs/examples/j-apportionment-evidence-packages/2020-census-table01/manifest.json"
        ))
        .expect("manifest fixture must parse")
    }

    fn table() -> ExtractedApportionmentTable {
        serde_json::from_str(include_str!(
            "../../../docs/examples/j-apportionment-evidence-packages/2020-census-table01/apportionment-table01.json"
        ))
        .expect("table fixture must parse")
    }

    #[test]
    fn manifest_fixture_is_hash_bound() {
        let manifest = manifest();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/examples/j-apportionment-evidence-packages/2020-census-table01");

        assert_eq!(manifest.validate_package(root), Ok(()));
    }

    #[test]
    fn census_2020_table01_replays_huntington_hill() {
        let manifest = manifest();
        let table = table();

        assert_eq!(table.verify_huntington_hill(&manifest), Ok(()));
    }

    #[test]
    fn detects_tampered_official_seat() {
        let manifest = manifest();
        let mut table = table();
        table.rows[0].representatives += 1;

        assert!(matches!(
            table.verify_huntington_hill(&manifest),
            Err(ApportionmentEvidenceError::SeatTotalMismatch(_, _))
                | Err(ApportionmentEvidenceError::HuntingtonHillMismatch { .. })
        ));
    }
}
