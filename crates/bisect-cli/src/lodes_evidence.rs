use crate::edge_weights::{EconomicCharacterWeighter, EdgeMap, EdgeWeighter};
use crate::lodes::{derive_economic_character, LodesWacRaw};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const M1_ECONOMIC_EVIDENCE_MANIFEST_VERSION: &str = "m1-economic-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicEvidenceManifest {
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
pub struct EconomicFixtureSet {
    pub fixture_version: String,
    pub rows: Vec<EconomicFixtureRow>,
    pub edges: Vec<EconomicEdgeFixture>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicFixtureRow {
    pub geoid: String,
    pub raw: RawEconomicFixture,
    pub expected: ExpectedEconomicFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawEconomicFixture {
    pub c000: f64,
    pub cns01: f64,
    pub cns02: f64,
    pub cns05: f64,
    pub cns07: f64,
    pub cns08: f64,
    pub cns09: f64,
    pub cns10: f64,
    pub cns11: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedEconomicFixture {
    pub commercial_intensity: f64,
    pub industrial_fraction: f64,
    pub jobs_per_resident: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicEdgeFixture {
    pub endpoints: [usize; 2],
    pub base_weight: f64,
    pub alpha: f64,
    pub expected_weight: f64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EconomicEvidenceError {
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
    #[error(
        "economic row {geoid} field {field} mismatch: expected {expected}, computed {computed}"
    )]
    RowMismatch {
        geoid: String,
        field: &'static str,
        expected: f64,
        computed: f64,
    },
    #[error("edge {edge:?} mismatch: expected {expected}, computed {computed}")]
    EdgeMismatch {
        edge: (usize, usize),
        expected: f64,
        computed: f64,
    },
}

impl EconomicEvidenceManifest {
    pub fn validate(&self) -> Result<(), EconomicEvidenceError> {
        if self.schema_version != M1_ECONOMIC_EVIDENCE_MANIFEST_VERSION {
            return Err(EconomicEvidenceError::UnsupportedSchemaVersion(
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
    ) -> Result<(), EconomicEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            EconomicEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(EconomicEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl EconomicFixtureSet {
    pub fn verify(&self) -> Result<(), EconomicEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        require_non_empty_vec("rows", &self.rows)?;
        require_non_empty_vec("edges", &self.edges)?;

        let mut chars = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            require_non_empty("rows[].geoid", &row.geoid)?;
            let computed = derive_economic_character(row.raw.to_raw());
            compare(
                &row.geoid,
                "commercial_intensity",
                row.expected.commercial_intensity,
                computed.commercial_intensity,
            )?;
            compare(
                &row.geoid,
                "industrial_fraction",
                row.expected.industrial_fraction,
                computed.industrial_fraction,
            )?;
            compare(
                &row.geoid,
                "jobs_per_resident",
                row.expected.jobs_per_resident,
                computed.jobs_per_resident,
            )?;
            chars.push(computed);
        }

        for edge in &self.edges {
            let key = (edge.endpoints[0], edge.endpoints[1]);
            let weights: EdgeMap = HashMap::from([(key, edge.base_weight)]);
            let computed = EconomicCharacterWeighter::new(chars.clone(), edge.alpha).apply(weights);
            let got = computed.get(&key).copied().unwrap_or(f64::NAN);
            if !nearly_equal(got, edge.expected_weight) {
                return Err(EconomicEvidenceError::EdgeMismatch {
                    edge: key,
                    expected: edge.expected_weight,
                    computed: got,
                });
            }
        }

        Ok(())
    }
}

impl RawEconomicFixture {
    fn to_raw(&self) -> LodesWacRaw {
        LodesWacRaw {
            c000: self.c000,
            cns01: self.cns01,
            cns02: self.cns02,
            cns05: self.cns05,
            cns07: self.cns07,
            cns08: self.cns08,
            cns09: self.cns09,
            cns10: self.cns10,
            cns11: self.cns11,
        }
    }
}

fn compare(
    geoid: &str,
    field: &'static str,
    expected: f64,
    computed: f64,
) -> Result<(), EconomicEvidenceError> {
    if nearly_equal(expected, computed) {
        Ok(())
    } else {
        Err(EconomicEvidenceError::RowMismatch {
            geoid: geoid.to_string(),
            field,
            expected,
            computed,
        })
    }
}

fn nearly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6
}

fn validate_file(file: &EvidenceFile) -> Result<(), EconomicEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), EconomicEvidenceError> {
    if value.trim().is_empty() {
        Err(EconomicEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), EconomicEvidenceError> {
    if value.is_empty() {
        Err(EconomicEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), EconomicEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(EconomicEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), EconomicEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(EconomicEvidenceError::InvalidPath(path.to_string()));
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
            .join("m-community-character-evidence-packages")
            .join("M.1+economic-character-smoke")
    }

    #[test]
    fn m1_economic_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: EconomicEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn m1_economic_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: EconomicFixtureSet = serde_json::from_slice(
            &fs::read(root.join("economic-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_m1_economic_fixture_rejected() {
        let root = fixture_root();
        let mut fixtures: EconomicFixtureSet = serde_json::from_slice(
            &fs::read(root.join("economic-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.rows[0].expected.commercial_intensity += 0.1;
        assert!(matches!(
            fixtures.verify(),
            Err(EconomicEvidenceError::RowMismatch { .. })
        ));
    }
}
