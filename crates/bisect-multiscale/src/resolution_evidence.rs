use crate::hierarchy::HierarchyLevel;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const U11_RESOLUTION_EVIDENCE_MANIFEST_VERSION: &str = "u11-resolution-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionEvidenceManifest {
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
pub struct ResolutionFixtureSet {
    pub fixture_version: String,
    pub option_b: ResolutionOptionFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionOptionFixture {
    pub paper: String,
    pub plan_resolution: String,
    pub unit_type: String,
    pub n_units: usize,
    pub multiscale_fine: String,
    pub multiscale_coarse: String,
    pub fine_to_coarse_formula: String,
    pub geoid_prefix_len: usize,
    pub fine_geoids: Vec<String>,
    pub fine_populations: Vec<i64>,
    pub fine_adjacency: Vec<Vec<usize>>,
    pub expected_coarse_geoids: Vec<String>,
    pub expected_fine_to_coarse: Vec<usize>,
    pub expected_coarse_populations: Vec<i64>,
    pub expected_coarse_adjacency: Vec<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ResolutionEvidenceError {
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
    #[error("{field} mismatch: expected {expected}, computed {computed}")]
    NumericMismatch {
        field: &'static str,
        expected: String,
        computed: String,
    },
    #[error("resolution invariant failed: {0}")]
    Invariant(String),
}

impl ResolutionEvidenceManifest {
    pub fn validate(&self) -> Result<(), ResolutionEvidenceError> {
        if self.schema_version != U11_RESOLUTION_EVIDENCE_MANIFEST_VERSION {
            return Err(ResolutionEvidenceError::UnsupportedSchemaVersion(
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
    ) -> Result<(), ResolutionEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            ResolutionEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(ResolutionEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl ResolutionFixtureSet {
    pub fn verify(&self) -> Result<(), ResolutionEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        self.option_b.verify()
    }
}

impl ResolutionOptionFixture {
    pub fn verify(&self) -> Result<(), ResolutionEvidenceError> {
        require_non_empty("option_b.paper", &self.paper)?;
        require_non_empty("option_b.plan_resolution", &self.plan_resolution)?;
        require_non_empty("option_b.unit_type", &self.unit_type)?;
        require_non_empty("option_b.multiscale_fine", &self.multiscale_fine)?;
        require_non_empty("option_b.multiscale_coarse", &self.multiscale_coarse)?;
        require_non_empty(
            "option_b.fine_to_coarse_formula",
            &self.fine_to_coarse_formula,
        )?;

        if self.plan_resolution != self.multiscale_fine {
            return Err(ResolutionEvidenceError::Invariant(
                "plan_resolution must match multiscale_fine for fine-level outputs".to_string(),
            ));
        }
        if self.n_units != self.fine_geoids.len()
            || self.n_units != self.fine_populations.len()
            || self.n_units != self.fine_adjacency.len()
        {
            return Err(ResolutionEvidenceError::Invariant(
                "n_units must match fine GEOIDs, populations, and adjacency rows".to_string(),
            ));
        }
        if self.geoid_prefix_len == 0
            || !self
                .fine_to_coarse_formula
                .contains(&format!("[:{}]", self.geoid_prefix_len))
        {
            return Err(ResolutionEvidenceError::Invariant(
                "fine_to_coarse_formula must name the fixture prefix length".to_string(),
            ));
        }

        let coarse_geoids = derive_coarse_geoids(&self.fine_geoids, self.geoid_prefix_len)?;
        compare_vec(
            "expected_coarse_geoids",
            &self.expected_coarse_geoids,
            &coarse_geoids,
        )?;

        let coarse_index: HashMap<&str, usize> = coarse_geoids
            .iter()
            .enumerate()
            .map(|(idx, geoid)| (geoid.as_str(), idx))
            .collect();
        let mut fine_to_coarse = Vec::with_capacity(self.fine_geoids.len());
        for geoid in &self.fine_geoids {
            let prefix = geoid_prefix(geoid, self.geoid_prefix_len)?;
            let coarse = coarse_index.get(prefix).ok_or_else(|| {
                ResolutionEvidenceError::Invariant(format!(
                    "fine GEOID {geoid} has orphan prefix {prefix}"
                ))
            })?;
            fine_to_coarse.push(*coarse);
        }
        compare_vec(
            "expected_fine_to_coarse",
            &self.expected_fine_to_coarse,
            &fine_to_coarse,
        )?;

        let hierarchy = HierarchyLevel::from_fine(
            &self.fine_adjacency,
            &self.fine_populations,
            &fine_to_coarse,
            coarse_geoids.len(),
        );
        compare_vec(
            "expected_coarse_populations",
            &self.expected_coarse_populations,
            &hierarchy.pop,
        )?;
        compare_vec(
            "expected_coarse_adjacency",
            &self.expected_coarse_adjacency,
            &hierarchy.adj,
        )?;
        if hierarchy.fine_to_coarse != fine_to_coarse {
            return Err(ResolutionEvidenceError::Invariant(
                "HierarchyLevel fine_to_coarse does not match GEOID prefix derivation".to_string(),
            ));
        }
        Ok(())
    }
}

fn derive_coarse_geoids(
    fine_geoids: &[String],
    prefix_len: usize,
) -> Result<Vec<String>, ResolutionEvidenceError> {
    let mut coarse = Vec::new();
    for geoid in fine_geoids {
        let prefix = geoid_prefix(geoid, prefix_len)?.to_string();
        if !coarse.contains(&prefix) {
            coarse.push(prefix);
        }
    }
    coarse.sort();
    Ok(coarse)
}

fn geoid_prefix(geoid: &str, prefix_len: usize) -> Result<&str, ResolutionEvidenceError> {
    if geoid.len() < prefix_len {
        Err(ResolutionEvidenceError::Invariant(format!(
            "GEOID {geoid} is shorter than prefix length {prefix_len}"
        )))
    } else {
        Ok(&geoid[..prefix_len])
    }
}

fn compare_vec<T>(
    field: &'static str,
    expected: &[T],
    computed: &[T],
) -> Result<(), ResolutionEvidenceError>
where
    T: PartialEq + std::fmt::Debug,
{
    if expected == computed {
        Ok(())
    } else {
        Err(ResolutionEvidenceError::NumericMismatch {
            field,
            expected: format!("{expected:?}"),
            computed: format!("{computed:?}"),
        })
    }
}

fn validate_file(file: &EvidenceFile) -> Result<(), ResolutionEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ResolutionEvidenceError> {
    if value.trim().is_empty() {
        Err(ResolutionEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(
    field: &'static str,
    value: &[T],
) -> Result<(), ResolutionEvidenceError> {
    if value.is_empty() {
        Err(ResolutionEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), ResolutionEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ResolutionEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), ResolutionEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(ResolutionEvidenceError::InvalidPath(path.to_string()));
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
            .join("u-search-evidence-packages")
            .join("U.11+resolution-smoke")
    }

    #[test]
    fn u11_resolution_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: ResolutionEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn u11_resolution_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: ResolutionFixtureSet = serde_json::from_slice(
            &fs::read(root.join("resolution-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_resolution_prefix_is_rejected() {
        let root = fixture_root();
        let mut fixtures: ResolutionFixtureSet = serde_json::from_slice(
            &fs::read(root.join("resolution-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.option_b.geoid_prefix_len = 6;
        assert!(matches!(
            fixtures.verify(),
            Err(ResolutionEvidenceError::Invariant(message))
                if message.contains("prefix length")
        ));
    }

    #[test]
    fn tampered_resolution_population_is_rejected() {
        let root = fixture_root();
        let mut fixtures: ResolutionFixtureSet = serde_json::from_slice(
            &fs::read(root.join("resolution-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.option_b.expected_coarse_populations[1] += 1;
        assert!(matches!(
            fixtures.verify(),
            Err(ResolutionEvidenceError::NumericMismatch {
                field: "expected_coarse_populations",
                ..
            })
        ));
    }
}
