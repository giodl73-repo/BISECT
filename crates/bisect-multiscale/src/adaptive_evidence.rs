use crate::chain::AdaptiveMultiScaleConfig;
use crate::seeds::step_seed;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const U5_ADAPTIVE_EVIDENCE_MANIFEST_VERSION: &str = "u5-adaptive-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveEvidenceManifest {
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
pub struct AdaptiveFixtureSet {
    pub fixture_version: String,
    pub robbins_monro: RobbinsMonroFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobbinsMonroFixture {
    pub paper: String,
    pub total_steps: usize,
    pub adapt_interval: usize,
    pub initial_alpha: f64,
    pub target_accept: f64,
    pub gamma_0: f64,
    pub gamma_exponent: f64,
    pub min_alpha: f64,
    pub max_alpha: f64,
    pub pop_tolerance: f64,
    pub expected_coarse_tol: f64,
    pub base_seed: u64,
    pub chain_idx: u32,
    pub seed_step: u64,
    pub expected_step_seed: u64,
    pub coarse_acceptance_windows: Vec<Vec<bool>>,
    pub expected_acceptance_rates: Vec<f64>,
    pub expected_gamma_trace: Vec<f64>,
    pub expected_alpha_trace: Vec<f64>,
    pub expected_final_alpha: f64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AdaptiveEvidenceError {
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
    #[error("adaptive invariant failed: {0}")]
    Invariant(String),
}

impl AdaptiveEvidenceManifest {
    pub fn validate(&self) -> Result<(), AdaptiveEvidenceError> {
        if self.schema_version != U5_ADAPTIVE_EVIDENCE_MANIFEST_VERSION {
            return Err(AdaptiveEvidenceError::UnsupportedSchemaVersion(
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
    ) -> Result<(), AdaptiveEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            AdaptiveEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(AdaptiveEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl AdaptiveFixtureSet {
    pub fn verify(&self) -> Result<(), AdaptiveEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        self.robbins_monro.verify()
    }
}

impl RobbinsMonroFixture {
    pub fn verify(&self) -> Result<(), AdaptiveEvidenceError> {
        require_non_empty("robbins_monro.paper", &self.paper)?;
        if self.adapt_interval == 0 {
            return Err(AdaptiveEvidenceError::Invariant(
                "adapt_interval must be positive".to_string(),
            ));
        }
        if self.coarse_acceptance_windows.len() != self.total_steps / self.adapt_interval {
            return Err(AdaptiveEvidenceError::Invariant(
                "window count must equal floor(total_steps / adapt_interval)".to_string(),
            ));
        }
        if self.expected_acceptance_rates.len() != self.coarse_acceptance_windows.len()
            || self.expected_gamma_trace.len() != self.coarse_acceptance_windows.len()
            || self.expected_alpha_trace.len() != self.coarse_acceptance_windows.len()
        {
            return Err(AdaptiveEvidenceError::Invariant(
                "expected traces must match adaptation-round count".to_string(),
            ));
        }

        let cfg = AdaptiveMultiScaleConfig {
            total_steps: self.total_steps,
            target_accept: self.target_accept,
            initial_alpha: self.initial_alpha,
            adapt_interval: self.adapt_interval,
            gamma_0: self.gamma_0,
            pop_tolerance: self.pop_tolerance,
            coarse_tol: 3.0 * self.pop_tolerance,
            p: 0.0,
            base_seed: self.base_seed,
            chain_idx: self.chain_idx,
        };
        approx(
            "expected_coarse_tol",
            cfg.coarse_tol,
            self.expected_coarse_tol,
            1e-12,
        )?;
        let computed_seed = step_seed(cfg.base_seed, self.seed_step, cfg.chain_idx);
        if computed_seed != self.expected_step_seed {
            return Err(AdaptiveEvidenceError::NumericMismatch {
                field: "expected_step_seed",
                expected: computed_seed as f64,
                computed: self.expected_step_seed as f64,
            });
        }

        let replay = replay_alpha_trace(
            self.initial_alpha,
            self.target_accept,
            self.gamma_0,
            self.gamma_exponent,
            self.min_alpha,
            self.max_alpha,
            &self.coarse_acceptance_windows,
        )?;
        for (i, round) in replay.iter().enumerate() {
            approx(
                "expected_acceptance_rates",
                round.acceptance_rate,
                self.expected_acceptance_rates[i],
                1e-12,
            )?;
            approx(
                "expected_gamma_trace",
                round.gamma,
                self.expected_gamma_trace[i],
                1e-12,
            )?;
            approx(
                "expected_alpha_trace",
                round.alpha,
                self.expected_alpha_trace[i],
                1e-12,
            )?;
        }
        let final_alpha = replay
            .last()
            .map(|round| round.alpha)
            .unwrap_or(self.initial_alpha);
        approx(
            "expected_final_alpha",
            final_alpha,
            self.expected_final_alpha,
            1e-12,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AdaptationRound {
    acceptance_rate: f64,
    gamma: f64,
    alpha: f64,
}

fn replay_alpha_trace(
    initial_alpha: f64,
    target_accept: f64,
    gamma_0: f64,
    gamma_exponent: f64,
    min_alpha: f64,
    max_alpha: f64,
    windows: &[Vec<bool>],
) -> Result<Vec<AdaptationRound>, AdaptiveEvidenceError> {
    if min_alpha > max_alpha {
        return Err(AdaptiveEvidenceError::Invariant(
            "min_alpha must be <= max_alpha".to_string(),
        ));
    }
    let mut alpha = initial_alpha;
    let mut trace = Vec::with_capacity(windows.len());
    for (idx, window) in windows.iter().enumerate() {
        if window.is_empty() {
            return Err(AdaptiveEvidenceError::Invariant(
                "coarse acceptance windows must not be empty".to_string(),
            ));
        }
        let acceptance_rate =
            window.iter().filter(|accepted| **accepted).count() as f64 / window.len() as f64;
        let round = (idx + 1) as f64;
        let gamma = gamma_0 / round.powf(gamma_exponent);
        alpha = (alpha + gamma * (acceptance_rate - target_accept)).clamp(min_alpha, max_alpha);
        trace.push(AdaptationRound {
            acceptance_rate,
            gamma,
            alpha,
        });
    }
    Ok(trace)
}

fn validate_file(file: &EvidenceFile) -> Result<(), AdaptiveEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), AdaptiveEvidenceError> {
    if value.trim().is_empty() {
        Err(AdaptiveEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), AdaptiveEvidenceError> {
    if value.is_empty() {
        Err(AdaptiveEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), AdaptiveEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(AdaptiveEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), AdaptiveEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(AdaptiveEvidenceError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn approx(
    field: &'static str,
    expected: f64,
    computed: f64,
    tolerance: f64,
) -> Result<(), AdaptiveEvidenceError> {
    if (expected - computed).abs() <= tolerance {
        Ok(())
    } else {
        Err(AdaptiveEvidenceError::NumericMismatch {
            field,
            expected,
            computed,
        })
    }
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
            .join("U.5+adaptive-multiscale-smoke")
    }

    #[test]
    fn u5_adaptive_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: AdaptiveEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn u5_adaptive_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: AdaptiveFixtureSet = serde_json::from_slice(
            &fs::read(root.join("adaptive-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_adaptive_alpha_trace_is_rejected() {
        let root = fixture_root();
        let mut fixtures: AdaptiveFixtureSet = serde_json::from_slice(
            &fs::read(root.join("adaptive-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.robbins_monro.expected_alpha_trace[1] += 0.01;
        assert!(matches!(
            fixtures.verify(),
            Err(AdaptiveEvidenceError::NumericMismatch {
                field: "expected_alpha_trace",
                ..
            })
        ));
    }

    #[test]
    fn tampered_adaptive_seed_is_rejected() {
        let root = fixture_root();
        let mut fixtures: AdaptiveFixtureSet = serde_json::from_slice(
            &fs::read(root.join("adaptive-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.robbins_monro.expected_step_seed += 1;
        assert!(matches!(
            fixtures.verify(),
            Err(AdaptiveEvidenceError::NumericMismatch {
                field: "expected_step_seed",
                ..
            })
        ));
    }
}
