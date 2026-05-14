use crate::bisection_runner::{derive_sa_seed, split_subgraph_sa};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const U3_SA_EVIDENCE_MANIFEST_VERSION: &str = "u3-sa-evidence-manifest v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaEvidenceManifest {
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
pub struct SaFixtureSet {
    pub fixture_version: String,
    pub seed_fixture: SeedFixture,
    pub grid_fixture: GridFixture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedFixture {
    pub base_seed: u64,
    pub node_path: String,
    pub expected_sa_seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridFixture {
    pub rows: usize,
    pub cols: usize,
    pub balance_tolerance: f64,
    pub steps_per_tract: usize,
    pub t0_factor: f64,
    pub t_final: f64,
    pub base_seed: u64,
    pub node_path: String,
    pub expected_total_tracts: usize,
    pub max_edge_cut: usize,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SaEvidenceError {
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
    #[error("derived seed mismatch: expected {expected}, computed {computed}")]
    SeedMismatch { expected: u64, computed: u64 },
    #[error("invalid grid fixture: {0}")]
    InvalidGrid(String),
    #[error("SA partition failed: {0}")]
    SaFailed(String),
    #[error("SA partition invariant failed: {0}")]
    PartitionInvariant(String),
}

impl SaEvidenceManifest {
    pub fn validate(&self) -> Result<(), SaEvidenceError> {
        if self.schema_version != U3_SA_EVIDENCE_MANIFEST_VERSION {
            return Err(SaEvidenceError::UnsupportedSchemaVersion(
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

    pub fn validate_package(&self, package_root: impl AsRef<Path>) -> Result<(), SaEvidenceError> {
        self.validate()?;
        let file = &self.fixture_file;
        let bytes = fs::read(package_root.as_ref().join(&file.path)).map_err(|error| {
            SaEvidenceError::FileRead {
                path: file.path.clone(),
                message: error.to_string(),
            }
        })?;
        let computed = format!("{:x}", Sha256::digest(&bytes));
        if computed != file.sha256 {
            return Err(SaEvidenceError::FileHashMismatch {
                path: file.path.clone(),
                declared: file.sha256.clone(),
                computed,
            });
        }
        Ok(())
    }
}

impl SaFixtureSet {
    pub fn verify(&self) -> Result<(), SaEvidenceError> {
        require_non_empty("fixture_version", &self.fixture_version)?;
        require_non_empty("seed_fixture.node_path", &self.seed_fixture.node_path)?;
        require_non_empty("grid_fixture.node_path", &self.grid_fixture.node_path)?;

        let computed_seed =
            derive_sa_seed(self.seed_fixture.base_seed, &self.seed_fixture.node_path);
        if computed_seed != self.seed_fixture.expected_sa_seed {
            return Err(SaEvidenceError::SeedMismatch {
                expected: self.seed_fixture.expected_sa_seed,
                computed: computed_seed,
            });
        }

        let grid = &self.grid_fixture;
        if grid.rows == 0 || grid.cols == 0 || grid.expected_total_tracts != grid.rows * grid.cols {
            return Err(SaEvidenceError::InvalidGrid(
                "rows*cols must equal expected_total_tracts and be non-zero".to_string(),
            ));
        }
        let (adjacency, vertex_weights) = grid_graph(grid.rows, grid.cols);
        let tracts: HashSet<usize> = (0..grid.expected_total_tracts).collect();
        let sa_seed = derive_sa_seed(grid.base_seed, &grid.node_path);
        let (left_a, right_a) = split_subgraph_sa(
            &adjacency,
            &vertex_weights,
            &HashMap::new(),
            &tracts,
            grid.balance_tolerance,
            grid.steps_per_tract,
            grid.t0_factor,
            grid.t_final,
            sa_seed,
        )
        .map_err(SaEvidenceError::SaFailed)?;
        let (left_b, right_b) = split_subgraph_sa(
            &adjacency,
            &vertex_weights,
            &HashMap::new(),
            &tracts,
            grid.balance_tolerance,
            grid.steps_per_tract,
            grid.t0_factor,
            grid.t_final,
            sa_seed,
        )
        .map_err(SaEvidenceError::SaFailed)?;

        if left_a != left_b || right_a != right_b {
            return Err(SaEvidenceError::PartitionInvariant(
                "same seed did not reproduce the same partition".to_string(),
            ));
        }
        validate_partition(&adjacency, &left_a, &right_a, grid.expected_total_tracts)?;
        let cut = edge_cut(&adjacency, &left_a);
        if cut > grid.max_edge_cut {
            return Err(SaEvidenceError::PartitionInvariant(format!(
                "edge cut {cut} exceeded max_edge_cut {}",
                grid.max_edge_cut
            )));
        }

        Ok(())
    }
}

fn grid_graph(rows: usize, cols: usize) -> (Vec<Vec<usize>>, Vec<i64>) {
    let n = rows * cols;
    let mut adj = vec![Vec::new(); n];
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            if r > 0 {
                adj[i].push((r - 1) * cols + c);
            }
            if r + 1 < rows {
                adj[i].push((r + 1) * cols + c);
            }
            if c > 0 {
                adj[i].push(r * cols + c - 1);
            }
            if c + 1 < cols {
                adj[i].push(r * cols + c + 1);
            }
        }
    }
    (adj, vec![1; n])
}

fn validate_partition(
    adjacency: &[Vec<usize>],
    left: &HashSet<usize>,
    right: &HashSet<usize>,
    n: usize,
) -> Result<(), SaEvidenceError> {
    if left.is_empty() || right.is_empty() {
        return Err(SaEvidenceError::PartitionInvariant(
            "both sides must be non-empty".to_string(),
        ));
    }
    if left.len() + right.len() != n || !left.is_disjoint(right) {
        return Err(SaEvidenceError::PartitionInvariant(
            "partition must cover every tract exactly once".to_string(),
        ));
    }
    if !is_connected(adjacency, left) || !is_connected(adjacency, right) {
        return Err(SaEvidenceError::PartitionInvariant(
            "both partition sides must be connected".to_string(),
        ));
    }
    Ok(())
}

fn is_connected(adjacency: &[Vec<usize>], side: &HashSet<usize>) -> bool {
    let Some(&start) = side.iter().next() else {
        return false;
    };
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(v) = queue.pop_front() {
        for &nb in &adjacency[v] {
            if side.contains(&nb) && seen.insert(nb) {
                queue.push_back(nb);
            }
        }
    }
    seen.len() == side.len()
}

fn edge_cut(adjacency: &[Vec<usize>], left: &HashSet<usize>) -> usize {
    rgraph_core::undirected_edge_cut_by(adjacency, |node| left.contains(&node))
        .expect("validated SA evidence adjacency")
}

fn validate_file(file: &EvidenceFile) -> Result<(), SaEvidenceError> {
    validate_portable_path(&file.path)?;
    validate_sha256(&file.sha256)?;
    require_non_empty("fixture_file.role", &file.role)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), SaEvidenceError> {
    if value.trim().is_empty() {
        Err(SaEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_non_empty_vec<T>(field: &'static str, value: &[T]) -> Result<(), SaEvidenceError> {
    if value.is_empty() {
        Err(SaEvidenceError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), SaEvidenceError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(SaEvidenceError::InvalidSha256(value.to_string()))
    }
}

fn validate_portable_path(path: &str) -> Result<(), SaEvidenceError> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(':')
    {
        return Err(SaEvidenceError::InvalidPath(path.to_string()));
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
            .join("U.3+sa-smoke")
    }

    #[test]
    fn u3_sa_smoke_package_hashes_validate() {
        let root = fixture_root();
        let manifest: SaEvidenceManifest =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("read manifest"))
                .expect("parse manifest");
        manifest.validate_package(&root).expect("valid package");
    }

    #[test]
    fn u3_sa_smoke_fixture_replays() {
        let root = fixture_root();
        let fixtures: SaFixtureSet = serde_json::from_slice(
            &fs::read(root.join("sa-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.verify().expect("valid fixtures");
    }

    #[test]
    fn tampered_u3_sa_fixture_rejected() {
        let root = fixture_root();
        let mut fixtures: SaFixtureSet = serde_json::from_slice(
            &fs::read(root.join("sa-fixtures.json")).expect("read fixtures"),
        )
        .expect("parse fixtures");
        fixtures.seed_fixture.expected_sa_seed += 1;
        assert!(matches!(
            fixtures.verify(),
            Err(SaEvidenceError::SeedMismatch { .. })
        ));
    }
}
