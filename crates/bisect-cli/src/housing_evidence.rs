use crate::edge_weights::{EdgeMap, EdgeWeighter, HousingCharacterWeighter};
use crate::housing::{derive_housing_character, AcsHousingRaw};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const M3_HOUSING_EVIDENCE_MANIFEST_VERSION: &str = "m3-housing-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HousingEvidenceManifest {
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
pub struct HousingFixtureSet {
    pub fixture_version: String,
    pub rows: Vec<HousingFixtureRow>,
    pub edges: Vec<HousingEdgeFixture>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HousingFixtureRow {
    pub geoid: String,
    pub raw: RawHousingFixture,
    pub expected: ExpectedHousingFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawHousingFixture {
    pub total_units: f64,
    pub single_family_detached: f64,
    pub single_family_attached: f64,
    pub multifamily_10_19: f64,
    pub multifamily_20_49: f64,
    pub multifamily_50_plus: f64,
    pub occupied_total: f64,
    pub owner_occupied: f64,
    pub median_year_built: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedHousingFixture {
    pub pct_single_family: f64,
    pub pct_multifamily: f64,
    pub pct_owner: f64,
    pub housing_vintage: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HousingEdgeFixture {
    pub endpoints: [usize; 2],
    pub base_weight: f64,
    pub alpha: f64,
    pub expected_weight: f64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum HousingEvidenceError {
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
        "housing row {geoid} field {field} mismatch: expected {expected}, computed {computed}"
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

impl HousingEvidenceManifest {
    pub fn validate(&self) -> Result<(), HousingEvidenceError> {
        if self.schema_version != M3_HOUSING_EVIDENCE_MANIFEST_VERSION {
            return Err(HousingEvidenceError::UnsupportedSchemaVersion(
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
    ) -> Result<(), HousingEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            HousingEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(HousingEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl HousingFixtureSet {
    pub fn verify(&self) -> Result<(), HousingEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        require_non_empty_vec("rows", &self.rows)?;
        require_non_empty_vec("edges", &self.edges)?;

        let mut chars = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            require_non_empty("rows[].geoid", &row.geoid)?;
            let computed = derive_housing_character(row.raw.to_raw());
            compare(
                &row.geoid,
                "pct_single_family",
                row.expected.pct_single_family,
                computed.pct_single_family,
            )?;
            compare(
                &row.geoid,
                "pct_multifamily",
                row.expected.pct_multifamily,
                computed.pct_multifamily,
            )?;
            compare(
                &row.geoid,
                "pct_owner",
                row.expected.pct_owner,
                computed.pct_owner,
            )?;
            compare(
                &row.geoid,
                "housing_vintage",
                row.expected.housing_vintage,
                computed.housing_vintage,
            )?;
            chars.push(computed);
        }

        for edge in &self.edges {
            let key = (edge.endpoints[0], edge.endpoints[1]);
            let weights: EdgeMap = HashMap::from([(key, edge.base_weight)]);
            let computed = HousingCharacterWeighter::new(chars.clone(), edge.alpha).apply(weights);
            let got = computed.get(&key).copied().unwrap_or(f64::NAN);
            if !nearly_equal(got, edge.expected_weight) {
                return Err(HousingEvidenceError::EdgeMismatch {
                    edge: key,
                    expected: edge.expected_weight,
                    computed: got,
                });
            }
        }

        Ok(())
    }
}

impl RawHousingFixture {
    fn to_raw(&self) -> AcsHousingRaw {
        AcsHousingRaw {
            total_units: self.total_units,
            single_family_detached: self.single_family_detached,
            single_family_attached: self.single_family_attached,
            multifamily_10_19: self.multifamily_10_19,
            multifamily_20_49: self.multifamily_20_49,
            multifamily_50_plus: self.multifamily_50_plus,
            occupied_total: self.occupied_total,
            owner_occupied: self.owner_occupied,
            median_year_built: self.median_year_built,
        }
    }
}

fn compare(
    geoid: &str,
    field: &'static str,
    expected: f64,
    computed: f64,
) -> Result<(), HousingEvidenceError> {
    if nearly_equal(expected, computed) {
        Ok(())
    } else {
        Err(HousingEvidenceError::RowMismatch {
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

fn validate_file(file: &EvidenceFile) -> Result<(), HousingEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), HousingEvidenceError> {
    if value.trim().is_empty() {
        Err(HousingEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), HousingEvidenceError> {
    if value.is_empty() {
        Err(HousingEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), HousingEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(HousingEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), HousingEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(HousingEvidenceError::InvalidPath(path.to_string()));
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
            .join("M.3+acs-housing-smoke")
    }

    #[test]
    fn m3_housing_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: HousingEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn m3_housing_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: HousingFixtureSet = serde_json::from_slice(
            &fs::read(root.join("housing-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_m3_housing_fixture_rejected() {
        let root = fixture_root();
        let mut fixtures: HousingFixtureSet = serde_json::from_slice(
            &fs::read(root.join("housing-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.rows[0].expected.pct_single_family += 0.1;
        assert!(matches!(
            fixtures.verify(),
            Err(HousingEvidenceError::RowMismatch { .. })
        ));
    }
}
