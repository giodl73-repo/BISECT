/// PlanManifest — full audit chain for redistricting plans.
///
/// Records all inputs needed to independently reproduce and verify a plan:
/// - Binary version + SHA-256 + download URL (GitHub release)
/// - Adjacency file SHA-256 (filename only, not local path)
/// - TIGER source URL (Census.gov — no local paths)
/// - Balance tolerance, population source, chamber, seed
///
/// Board amendments:
/// - sha256_file() streams in 64KB chunks (no full-file Vec<u8>)
/// - manifest.json written via manifest.tmp + rename() (atomic)
/// - adjacency_file is filename only, never a full path
use std::io::Read;
use std::path::Path;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

/// Full audit chain for a redistricting plan run.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanManifest {
    pub label: String,
    pub state_code: String,
    pub year: String,
    pub chamber: String,
    pub num_districts: usize,
    pub population_source: String,
    pub partition_mode: String,
    pub seed: Option<i64>,
    pub binary_version: String,
    pub binary_sha256: String,
    /// GitHub release URL — no local paths ever
    pub binary_download_url: String,
    /// Filename only (not full path) for platform-independent verification
    pub adjacency_file: String,
    pub adjacency_sha256: String,
    pub adjacency_build_command: String,
    pub adjacency_build_version: String,
    /// Census.gov URL for independent verification (no local paths)
    pub tiger_source_url: String,
    /// SHA-256 of upstream TIGER shapefile (None if not hashed)
    pub tiger_sha256: Option<String>,
    pub created_at: String,
    pub balance_tolerance_pct: f64,
    pub population_balance_valid: bool,
}

/// Compute SHA-256 of a byte slice. Returns 64-character lowercase hex string.
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 of a file by streaming in 64KB chunks.
///
/// Board amendment: do NOT read entire file into Vec<u8>.
/// This is safe for large files (CA TIGER .shp ~50MB) and does not
/// block a Rayon thread for longer than needed per chunk.
pub fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024]; // 64KB chunks
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Generate the default plan label: `{state_name}_{chamber}_{year}`.
/// Normalizes spaces to underscores and lowercases everything.
pub fn default_label(state_name: &str, chamber: &str, year: &str) -> String {
    let normalized_state = state_name.to_lowercase().replace(' ', "_");
    format!("{}_{}_{}", normalized_state, chamber, year)
}

/// Write a manifest atomically: write to `manifest.tmp` then rename to `manifest.json`.
///
/// Board amendment: atomic write prevents partial manifests.
/// On startup, `manifest.tmp` presence signals an incomplete plan.
pub fn write_manifest_atomic(dir: &Path, manifest: &PlanManifest) -> anyhow::Result<()> {
    let tmp_path = dir.join("manifest.tmp");
    let final_path = dir.join("manifest.json");
    let json = serde_json::to_string_pretty(manifest)?;
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}

/// Check for an incomplete plan (manifest.tmp present without manifest.json).
/// Returns Err with a clear message if an incomplete plan is detected.
pub fn check_incomplete_plan(dir: &Path, label: &str) -> anyhow::Result<()> {
    let tmp_path = dir.join("manifest.tmp");
    if tmp_path.exists() {
        anyhow::bail!(
            "ERROR: Incomplete plan '{}' detected (manifest.tmp exists). \
             Delete the plan directory and re-run.",
            label
        );
    }
    Ok(())
}

/// Build a TIGER source URL for a given state FIPS code and year.
/// Always points to Census.gov — never local paths.
pub fn tiger_source_url(state_fips: &str, year: &str) -> String {
    format!(
        "https://www2.census.gov/geo/tiger/TIGER{year}/TRACT/tl_{year}_{state_fips}_tract.zip",
        year = year,
        state_fips = state_fips,
    )
}

// ---------------------------------------------------------------------------
// Tests — Task 3
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    pub(crate) fn make_test_manifest(state_fips: &str) -> PlanManifest {
        PlanManifest {
            label: format!("{}_congressional_2020", state_fips),
            state_code: state_fips.to_uppercase(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 10,
            population_source: "total".into(),
            partition_mode: "edge-weighted".into(),
            seed: Some(42),
            binary_version: "0.1.0".into(),
            binary_sha256: "a".repeat(64),
            binary_download_url: "https://github.com/owner/redist/releases/download/v0.1.0/redist".into(),
            adjacency_file: format!("{}_adjacency_2020.adj.bin", state_fips.to_lowercase()),
            adjacency_sha256: "b".repeat(64),
            adjacency_build_command: "python scripts/data/generate_adj_bin.py".into(),
            adjacency_build_version: "0.1.0".into(),
            tiger_source_url: tiger_source_url(state_fips, "2020"),
            tiger_sha256: Some("c".repeat(64)),
            created_at: "2026-04-26T00:00:00Z".into(),
            balance_tolerance_pct: 0.5,
            population_balance_valid: true,
        }
    }

    #[test]
    fn test_manifest_sha256_is_deterministic() {
        let content = b"test shapefile data";
        let hash1 = sha256_bytes(content);
        let hash2 = sha256_bytes(content);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // hex-encoded SHA-256
    }

    #[test]
    fn test_manifest_sha256_file_hello_world() {
        // Board amendment: L0 test that calls sha256_file() on real temp file
        // Known SHA-256 of b"hello world"
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.bin");
        std::fs::write(&path, b"hello world").unwrap();
        let hash = sha256_file(&path).unwrap();
        // SHA-256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576efe49e5d5ee24a9a
        // Note: this is the canonical test vector
        assert_eq!(hash.len(), 64, "SHA-256 hex must be 64 chars");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "sha256_file() must return known SHA-256 of 'hello world'"
        );
    }

    #[test]
    fn test_manifest_has_tiger_sha256_field() {
        let manifest = PlanManifest {
            tiger_sha256: Some("a".repeat(64)),
            ..Default::default()
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert!(json["tiger_sha256"].is_string());
    }

    #[test]
    fn test_manifest_has_tiger_source_url() {
        let manifest = make_test_manifest("53");
        assert!(manifest.tiger_source_url.contains("census.gov"));
        assert!(manifest.tiger_source_url.contains("53"));
    }

    #[test]
    fn test_manifest_adjacency_file_is_filename_not_path() {
        // adjacency_file must be filename only, not a full local path
        let manifest = make_test_manifest("53");
        assert!(!manifest.adjacency_file.contains('/'));
        assert!(!manifest.adjacency_file.contains('\\'));
    }

    #[test]
    fn test_manifest_binary_download_url_contains_github() {
        let manifest = make_test_manifest("53");
        assert!(manifest.binary_download_url.contains("github.com"));
    }

    #[test]
    fn test_label_default_generation_wa_house_2020() {
        let label = default_label("washington", "house", "2020");
        assert_eq!(label, "washington_house_2020");
    }

    #[test]
    fn test_label_default_generation_normalizes_spaces() {
        // "New York" → "new_york_congressional_2020"
        let label = default_label("new york", "congressional", "2020");
        assert_eq!(label, "new_york_congressional_2020");
    }

    #[test]
    fn test_write_manifest_atomic_renames_tmp() {
        let tmp = TempDir::new().unwrap();
        let manifest = make_test_manifest("53");
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        // manifest.json must exist, manifest.tmp must not
        assert!(tmp.path().join("manifest.json").exists());
        assert!(!tmp.path().join("manifest.tmp").exists());
    }

    #[test]
    fn test_check_incomplete_plan_detects_tmp() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("manifest.tmp"), b"{}").unwrap();
        let result = check_incomplete_plan(tmp.path(), "test_label");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Incomplete plan"));
        assert!(msg.contains("manifest.tmp"));
    }

    #[test]
    fn test_check_incomplete_plan_ok_when_no_tmp() {
        let tmp = TempDir::new().unwrap();
        assert!(check_incomplete_plan(tmp.path(), "test_label").is_ok());
    }
}
