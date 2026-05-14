use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const SHORT_BURST_EVIDENCE_MANIFEST_VERSION: &str = "g-short-burst-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortBurstEvidenceManifest {
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
pub struct ShortBurstFixtureSet {
    pub fixture_version: String,
    pub standard: ShortBurstRunFixture,
    pub forest_recom: SeedStreamFixture,
    pub merge_split: SeedStreamFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortBurstRunFixture {
    pub paper: String,
    pub seed_prefix: String,
    pub base_seed: u64,
    pub burst_length: u32,
    pub n_bursts: usize,
    pub percentile: f64,
    pub expected_chain_seeds: Vec<u64>,
    pub bursts: Vec<BurstRecord>,
    pub expected_selected_burst_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurstRecord {
    pub burst_index: usize,
    pub start_plan_id: String,
    pub endpoint_plan_id: String,
    pub within_burst_min_plan_id: String,
    pub edge_cut: u32,
    pub accepted_steps: u32,
    pub acceptance_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedStreamFixture {
    pub paper: String,
    pub chain_seed_prefix: String,
    pub forward_seed_prefix: String,
    pub reverse_seed_prefix: String,
    pub base_seed: u64,
    pub burst_index: usize,
    pub step_index: u32,
    pub expected_chain_seed: u64,
    pub expected_forward_seed: u64,
    pub expected_reverse_seed: u64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ShortBurstEvidenceError {
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
    #[error("short-burst invariant failed: {0}")]
    Invariant(String),
}

impl ShortBurstEvidenceManifest {
    pub fn validate(&self) -> Result<(), ShortBurstEvidenceError> {
        if self.schema_version != SHORT_BURST_EVIDENCE_MANIFEST_VERSION {
            return Err(ShortBurstEvidenceError::UnsupportedSchemaVersion(
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
    ) -> Result<(), ShortBurstEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            ShortBurstEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(ShortBurstEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl ShortBurstFixtureSet {
    pub fn verify(&self) -> Result<(), ShortBurstEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        self.standard.verify()?;
        self.forest_recom.verify()?;
        self.merge_split.verify()
    }
}

impl ShortBurstRunFixture {
    pub fn verify(&self) -> Result<(), ShortBurstEvidenceError> {
        require_non_empty("standard.paper", &self.paper)?;
        require_non_empty("standard.seed_prefix", &self.seed_prefix)?;
        if self.burst_length == 0 {
            return Err(ShortBurstEvidenceError::Invariant(
                "burst_length must be positive".to_string(),
            ));
        }
        if self.n_bursts == 0
            || self.bursts.len() != self.n_bursts
            || self.expected_chain_seeds.len() != self.n_bursts
        {
            return Err(ShortBurstEvidenceError::Invariant(
                "n_bursts must match bursts and expected_chain_seeds".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.percentile) {
            return Err(ShortBurstEvidenceError::Invariant(
                "percentile must be in [0, 1]".to_string(),
            ));
        }

        for i in 0..self.n_bursts {
            let expected = short_burst_chain_seed(&self.seed_prefix, self.base_seed, i);
            if self.expected_chain_seeds[i] != expected {
                return Err(ShortBurstEvidenceError::NumericMismatch {
                    field: "expected_chain_seeds",
                    expected: expected as f64,
                    computed: self.expected_chain_seeds[i] as f64,
                });
            }
        }

        for (i, burst) in self.bursts.iter().enumerate() {
            if burst.burst_index != i {
                return Err(ShortBurstEvidenceError::NumericMismatch {
                    field: "burst_index",
                    expected: i as f64,
                    computed: burst.burst_index as f64,
                });
            }
            require_non_empty("burst.start_plan_id", &burst.start_plan_id)?;
            require_non_empty("burst.endpoint_plan_id", &burst.endpoint_plan_id)?;
            require_non_empty(
                "burst.within_burst_min_plan_id",
                &burst.within_burst_min_plan_id,
            )?;
            if i > 0 && burst.start_plan_id != self.bursts[i - 1].endpoint_plan_id {
                return Err(ShortBurstEvidenceError::Invariant(format!(
                    "burst {i} must restart from previous endpoint, not a selected within-burst minimum"
                )));
            }
            if burst.accepted_steps > self.burst_length {
                return Err(ShortBurstEvidenceError::NumericMismatch {
                    field: "accepted_steps",
                    expected: self.burst_length as f64,
                    computed: burst.accepted_steps as f64,
                });
            }
            approx(
                "acceptance_rate",
                burst.accepted_steps as f64 / self.burst_length as f64,
                burst.acceptance_rate,
                1e-12,
            )?;
        }

        let mut ranked: Vec<(u32, usize)> = self
            .bursts
            .iter()
            .map(|burst| (burst.edge_cut, burst.burst_index))
            .collect();
        ranked.sort_unstable();
        let selected_rank =
            ((self.percentile * self.n_bursts as f64).floor() as usize).min(self.n_bursts - 1);
        let selected_burst_index = ranked[selected_rank].1;
        if selected_burst_index != self.expected_selected_burst_index {
            return Err(ShortBurstEvidenceError::NumericMismatch {
                field: "expected_selected_burst_index",
                expected: selected_burst_index as f64,
                computed: self.expected_selected_burst_index as f64,
            });
        }

        Ok(())
    }
}

impl SeedStreamFixture {
    pub fn verify(&self) -> Result<(), ShortBurstEvidenceError> {
        require_non_empty("seed_stream.paper", &self.paper)?;
        require_non_empty("seed_stream.chain_seed_prefix", &self.chain_seed_prefix)?;
        require_non_empty("seed_stream.forward_seed_prefix", &self.forward_seed_prefix)?;
        require_non_empty("seed_stream.reverse_seed_prefix", &self.reverse_seed_prefix)?;

        let chain_seed =
            short_burst_chain_seed(&self.chain_seed_prefix, self.base_seed, self.burst_index);
        if chain_seed != self.expected_chain_seed {
            return Err(ShortBurstEvidenceError::NumericMismatch {
                field: "expected_chain_seed",
                expected: chain_seed as f64,
                computed: self.expected_chain_seed as f64,
            });
        }
        let forward_seed =
            short_burst_step_seed(&self.forward_seed_prefix, self.step_index, chain_seed);
        if forward_seed != self.expected_forward_seed {
            return Err(ShortBurstEvidenceError::NumericMismatch {
                field: "expected_forward_seed",
                expected: forward_seed as f64,
                computed: self.expected_forward_seed as f64,
            });
        }
        let reverse_seed =
            short_burst_step_seed(&self.reverse_seed_prefix, self.step_index, chain_seed);
        if reverse_seed != self.expected_reverse_seed {
            return Err(ShortBurstEvidenceError::NumericMismatch {
                field: "expected_reverse_seed",
                expected: reverse_seed as f64,
                computed: self.expected_reverse_seed as f64,
            });
        }
        Ok(())
    }
}

pub fn short_burst_chain_seed(prefix: &str, base_seed: u64, burst_index: usize) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update((burst_index as u64).to_le_bytes());
    hasher.update(b"_");
    hasher.update(base_seed.to_le_bytes());
    let digest = hasher.finalize();
    u64::from_le_bytes(digest[0..8].try_into().expect("sha256 has 32 bytes"))
}

pub fn short_burst_step_seed(prefix: &str, step_index: u32, chain_seed: u64) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(step_index.to_le_bytes());
    hasher.update(b"_");
    hasher.update(chain_seed.to_le_bytes());
    let digest = hasher.finalize();
    u64::from_le_bytes(digest[0..8].try_into().expect("sha256 has 32 bytes"))
}

fn validate_file(file: &EvidenceFile) -> Result<(), ShortBurstEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ShortBurstEvidenceError> {
    if value.trim().is_empty() {
        Err(ShortBurstEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(
    field: &'static str,
    value: &[T],
) -> Result<(), ShortBurstEvidenceError> {
    if value.is_empty() {
        Err(ShortBurstEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), ShortBurstEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ShortBurstEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), ShortBurstEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(ShortBurstEvidenceError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn approx(
    field: &'static str,
    expected: f64,
    computed: f64,
    tolerance: f64,
) -> Result<(), ShortBurstEvidenceError> {
    if (expected - computed).abs() <= tolerance {
        Ok(())
    } else {
        Err(ShortBurstEvidenceError::NumericMismatch {
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
            .join("g-ensemble-evidence-packages")
            .join("G.6-G.12+short-burst-smoke")
    }

    #[test]
    fn g_short_burst_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: ShortBurstEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn g_short_burst_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: ShortBurstFixtureSet = serde_json::from_slice(
            &fs::read(root.join("short-burst-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_short_burst_restart_is_rejected() {
        let root = fixture_root();
        let mut fixtures: ShortBurstFixtureSet = serde_json::from_slice(
            &fs::read(root.join("short-burst-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.standard.bursts[2].start_plan_id =
            fixtures.standard.bursts[1].within_burst_min_plan_id.clone();
        assert!(matches!(
            fixtures.verify(),
            Err(ShortBurstEvidenceError::Invariant(message))
                if message.contains("previous endpoint")
        ));
    }

    #[test]
    fn tampered_short_burst_seed_is_rejected() {
        let root = fixture_root();
        let mut fixtures: ShortBurstFixtureSet = serde_json::from_slice(
            &fs::read(root.join("short-burst-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.forest_recom.expected_forward_seed += 1;
        assert!(matches!(
            fixtures.verify(),
            Err(ShortBurstEvidenceError::NumericMismatch {
                field: "expected_forward_seed",
                ..
            })
        ));
    }
}
