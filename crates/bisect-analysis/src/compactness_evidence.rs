use crate::compactness::{exact_reock, minimum_bounding_circle, reock};
use geo::Area;
use geo_types::{Coord, LineString, Polygon};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const K_REOCK_EVIDENCE_MANIFEST_VERSION: &str = "k-reock-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReockEvidenceManifest {
    pub schema_version: String,
    pub package_id: String,
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
pub struct ReockFixtureSet {
    pub fixture_version: String,
    pub shapes: Vec<ReockFixtureShape>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReockFixtureShape {
    pub id: String,
    pub description: String,
    pub vertices: Vec<[f64; 2]>,
    pub expected_area: f64,
    pub expected_mbc_center: [f64; 2],
    pub expected_mbc_radius: f64,
    pub expected_exact_reock: f64,
    pub expected_proxy_reock: f64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ReockEvidenceError {
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
    #[error("fixture {id} mismatch for {field}: expected {expected}, computed {computed}")]
    FixtureMismatch {
        id: String,
        field: &'static str,
        expected: f64,
        computed: f64,
    },
}

impl ReockEvidenceManifest {
    pub fn validate(&self) -> Result<(), ReockEvidenceError> {
        if self.schema_version != K_REOCK_EVIDENCE_MANIFEST_VERSION {
            return Err(ReockEvidenceError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        require_non_empty("package_id", &self.package_id)?;
        validate_file(&self.fixture_file)?;
        validate_portable_path(&self.verifier_path)?;
        if self.verification_commands.is_empty() {
            return Err(ReockEvidenceError::EmptyField {
                field: "verification_commands",
            });
        }
        for command in &self.verification_commands {
            require_non_empty("verification_commands[]", command)?;
        }
        Ok(())
    }

    pub fn validate_package(
        &self,
        package_root: impl AsRef<Path>,
    ) -> Result<(), ReockEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            ReockEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(ReockEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl ReockFixtureSet {
    pub fn verify(&self) -> Result<(), ReockEvidenceError> {
        for shape in &self.shapes {
            shape.verify()?;
        }
        Ok(())
    }
}

impl ReockFixtureShape {
    pub fn verify(&self) -> Result<(), ReockEvidenceError> {
        let polygon = self.polygon();
        approx(
            &self.id,
            "area",
            self.expected_area,
            polygon.unsigned_area(),
            1e-9,
        )?;
        let circle = minimum_bounding_circle(&polygon)
            .map_err(|_| ReockEvidenceError::EmptyField { field: "polygon" })?;
        approx(
            &self.id,
            "mbc_center_x",
            self.expected_mbc_center[0],
            circle.center_x,
            1e-9,
        )?;
        approx(
            &self.id,
            "mbc_center_y",
            self.expected_mbc_center[1],
            circle.center_y,
            1e-9,
        )?;
        approx(
            &self.id,
            "mbc_radius",
            self.expected_mbc_radius,
            circle.radius,
            1e-9,
        )?;
        approx(
            &self.id,
            "exact_reock",
            self.expected_exact_reock,
            exact_reock(&polygon)
                .map_err(|_| ReockEvidenceError::EmptyField { field: "polygon" })?,
            1e-12,
        )?;
        approx(
            &self.id,
            "proxy_reock",
            self.expected_proxy_reock,
            reock(&polygon).map_err(|_| ReockEvidenceError::EmptyField { field: "polygon" })?,
            1e-12,
        )?;
        Ok(())
    }

    fn polygon(&self) -> Polygon<f64> {
        let mut coords: Vec<Coord<f64>> = self
            .vertices
            .iter()
            .map(|point| Coord {
                x: point[0],
                y: point[1],
            })
            .collect();
        if coords.first() != coords.last() {
            coords.push(coords[0]);
        }
        Polygon::new(LineString::new(coords), vec![])
    }
}

fn validate_file(file: &EvidenceFile) -> Result<(), ReockEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ReockEvidenceError> {
    if value.trim().is_empty() {
        Err(ReockEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), ReockEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ReockEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), ReockEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        Err(ReockEvidenceError::InvalidPath(path.to_string()))
    } else {
        Ok(())
    }
}

fn approx(
    id: &str,
    field: &'static str,
    expected: f64,
    computed: f64,
    tolerance: f64,
) -> Result<(), ReockEvidenceError> {
    if (expected - computed).abs() <= tolerance {
        Ok(())
    } else {
        Err(ReockEvidenceError::FixtureMismatch {
            id: id.to_string(),
            field,
            expected,
            computed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ReockEvidenceManifest {
        serde_json::from_str(include_str!(
            "../../../docs/examples/k-reock-evidence-packages/exact-mbc-smoke/manifest.json"
        ))
        .expect("manifest fixture must parse")
    }

    fn fixture_set() -> ReockFixtureSet {
        serde_json::from_str(include_str!(
            "../../../docs/examples/k-reock-evidence-packages/exact-mbc-smoke/reock-fixtures.json"
        ))
        .expect("fixture set must parse")
    }

    #[test]
    fn reock_manifest_fixture_is_hash_bound() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/examples/k-reock-evidence-packages/exact-mbc-smoke");
        assert_eq!(manifest().validate_package(root), Ok(()));
    }

    #[test]
    fn exact_mbc_fixture_values_validate() {
        assert_eq!(fixture_set().verify(), Ok(()));
    }

    #[test]
    fn triangle_fixture_distinguishes_exact_from_proxy() {
        let fixtures = fixture_set();
        let triangle = fixtures
            .shapes
            .iter()
            .find(|shape| shape.id == "right-triangle-3-4-5")
            .expect("triangle fixture exists");
        assert!(triangle.expected_exact_reock > triangle.expected_proxy_reock);
    }
}
