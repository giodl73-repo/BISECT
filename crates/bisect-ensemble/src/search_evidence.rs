use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const U_SEARCH_EVIDENCE_MANIFEST_VERSION: &str = "u-search-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct USearchEvidenceManifest {
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
pub struct SearchFixtureSet {
    pub fixture_version: String,
    pub parameter_sweep: ParameterSweepFixture,
    pub parallel_tempering: ParallelTemperingFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSweepFixture {
    pub paper: String,
    pub design: String,
    pub baseline_run_id: String,
    pub parameters: Vec<ParameterSweepParameter>,
    pub runs: Vec<ParameterSweepRun>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSweepParameter {
    pub name: String,
    pub default: f64,
    pub grid: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSweepRun {
    pub run_id: String,
    pub parameter: String,
    pub value: f64,
    pub output_sha256: String,
    pub metrics: SweepMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweepMetrics {
    pub mean_polsby_popper: f64,
    pub county_splits: u32,
    pub dem_seats: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParallelTemperingFixture {
    pub paper: String,
    pub base_seed: u64,
    pub steps: u64,
    pub n_replicas: usize,
    pub swap_interval: usize,
    pub cold_tolerance: f64,
    pub hot_tolerance: f64,
    pub tolerance_ladder: Vec<f64>,
    pub swap_attempts: u64,
    pub swap_acceptances: u64,
    pub selected_rank: usize,
    pub cold_chain_records: u64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SearchEvidenceError {
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
        expected: f64,
        computed: f64,
    },
    #[error("parameter sweep is missing baseline run {0}")]
    MissingBaseline(String),
    #[error("parameter {0} grid does not include its default")]
    DefaultMissingFromGrid(String),
    #[error("parallel tempering selected_rank is outside cold_chain_records")]
    SelectedRankOutOfRange,
}

impl USearchEvidenceManifest {
    pub fn validate(&self) -> Result<(), SearchEvidenceError> {
        if self.schema_version != U_SEARCH_EVIDENCE_MANIFEST_VERSION {
            return Err(SearchEvidenceError::UnsupportedSchemaVersion(
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
    ) -> Result<(), SearchEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            SearchEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(SearchEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl SearchFixtureSet {
    pub fn verify(&self) -> Result<(), SearchEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        self.parameter_sweep.verify()?;
        self.parallel_tempering.verify()
    }
}

impl ParameterSweepFixture {
    pub fn verify(&self) -> Result<(), SearchEvidenceError> {
        require_non_empty("parameter_sweep.paper", &self.paper)?;
        require_non_empty("parameter_sweep.design", &self.design)?;
        require_non_empty("parameter_sweep.baseline_run_id", &self.baseline_run_id)?;
        require_non_empty_vec("parameter_sweep.parameters", &self.parameters)?;
        require_non_empty_vec("parameter_sweep.runs", &self.runs)?;
        if !self
            .runs
            .iter()
            .any(|run| run.run_id == self.baseline_run_id)
        {
            return Err(SearchEvidenceError::MissingBaseline(
                self.baseline_run_id.clone(),
            ));
        }
        for parameter in &self.parameters {
            require_non_empty("parameter.name", &parameter.name)?;
            require_non_empty_vec("parameter.grid", &parameter.grid)?;
            if !parameter
                .grid
                .iter()
                .any(|value| approx_eq(*value, parameter.default, 1e-12))
            {
                return Err(SearchEvidenceError::DefaultMissingFromGrid(
                    parameter.name.clone(),
                ));
            }
        }
        for run in &self.runs {
            require_non_empty("run.run_id", &run.run_id)?;
            require_non_empty("run.parameter", &run.parameter)?;
            validate_sha256(&run.output_sha256)?;
            if !(0.0..=1.0).contains(&run.metrics.mean_polsby_popper) {
                return Err(SearchEvidenceError::NumericMismatch {
                    field: "mean_polsby_popper",
                    expected: 0.5,
                    computed: run.metrics.mean_polsby_popper,
                });
            }
        }
        Ok(())
    }
}

impl ParallelTemperingFixture {
    pub fn verify(&self) -> Result<(), SearchEvidenceError> {
        require_non_empty("parallel_tempering.paper", &self.paper)?;
        if self.tolerance_ladder.len() != self.n_replicas {
            return Err(SearchEvidenceError::NumericMismatch {
                field: "tolerance_ladder.len",
                expected: self.n_replicas as f64,
                computed: self.tolerance_ladder.len() as f64,
            });
        }
        for i in 0..self.n_replicas {
            let expected = if self.n_replicas == 1 {
                self.cold_tolerance
            } else {
                self.cold_tolerance
                    * (self.hot_tolerance / self.cold_tolerance)
                        .powf(i as f64 / (self.n_replicas - 1) as f64)
            };
            approx(
                "tolerance_ladder",
                expected,
                self.tolerance_ladder[i],
                1e-12,
            )?;
        }
        let expected_attempts =
            (self.steps / self.swap_interval as u64) * self.n_replicas.saturating_sub(1) as u64;
        if self.swap_attempts != expected_attempts {
            return Err(SearchEvidenceError::NumericMismatch {
                field: "swap_attempts",
                expected: expected_attempts as f64,
                computed: self.swap_attempts as f64,
            });
        }
        if self.swap_acceptances > self.swap_attempts {
            return Err(SearchEvidenceError::NumericMismatch {
                field: "swap_acceptances",
                expected: self.swap_attempts as f64,
                computed: self.swap_acceptances as f64,
            });
        }
        if self.cold_chain_records != self.steps + 1 {
            return Err(SearchEvidenceError::NumericMismatch {
                field: "cold_chain_records",
                expected: (self.steps + 1) as f64,
                computed: self.cold_chain_records as f64,
            });
        }
        if self.selected_rank as u64 >= self.cold_chain_records {
            return Err(SearchEvidenceError::SelectedRankOutOfRange);
        }
        Ok(())
    }
}

fn validate_file(file: &EvidenceFile) -> Result<(), SearchEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), SearchEvidenceError> {
    if value.trim().is_empty() {
        Err(SearchEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), SearchEvidenceError> {
    if value.is_empty() {
        Err(SearchEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), SearchEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(SearchEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), SearchEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(SearchEvidenceError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn approx(
    field: &'static str,
    expected: f64,
    computed: f64,
    tolerance: f64,
) -> Result<(), SearchEvidenceError> {
    if (expected - computed).abs() <= tolerance {
        Ok(())
    } else {
        Err(SearchEvidenceError::NumericMismatch {
            field,
            expected,
            computed,
        })
    }
}

fn approx_eq(left: f64, right: f64, tolerance: f64) -> bool {
    (left - right).abs() <= tolerance
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
            .join("synthetic-sweep-and-pt")
    }

    #[test]
    fn synthetic_u_search_package_hashes_validate() {
        let root = fixture_root();
        let manifest: USearchEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn synthetic_u_search_fixtures_verify() {
        let root = fixture_root();
        let fixtures: SearchFixtureSet = serde_json::from_slice(
            &fs::read(root.join("search-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_parameter_default_is_rejected() {
        let root = fixture_root();
        let mut fixtures: SearchFixtureSet = serde_json::from_slice(
            &fs::read(root.join("search-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.parameter_sweep.parameters[0].default = 0.12345;
        assert!(matches!(
            fixtures.verify(),
            Err(SearchEvidenceError::DefaultMissingFromGrid(_))
        ));
    }

    #[test]
    fn tampered_parallel_tempering_swap_count_is_rejected() {
        let root = fixture_root();
        let mut fixtures: SearchFixtureSet = serde_json::from_slice(
            &fs::read(root.join("search-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.parallel_tempering.swap_attempts += 1;
        assert!(matches!(
            fixtures.verify(),
            Err(SearchEvidenceError::NumericMismatch {
                field: "swap_attempts",
                ..
            })
        ));
    }
}
