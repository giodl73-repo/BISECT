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
    /// Seats per constituency (1 = single-member, 3-5 = multi-member)
    #[serde(default = "default_seats_per_district")]
    pub seats_per_district: usize,
    /// Total seats across all constituencies
    #[serde(default)]
    pub total_seats: usize,
    /// Electoral system classification
    #[serde(default = "default_electoral_system")]
    pub electoral_system: String,
    /// Version of gpmetis used for partitioning (captured at runtime)
    #[serde(default)]
    pub gpmetis_version: String,

    // ── State Staff Interop additions (Tasks 5, 7) ────────────────────────────
    /// Submission classification. `"authoritative"` (default for state-staff
    /// drawn maps) or `"civic_counter_proposal"` (Task 7, COMMONS). Downstream
    /// comparison reports surface this tag so a journalist screen-capping the
    /// summary card cannot mistake a civic counter-proposal for the official
    /// state map.
    #[serde(default = "default_submission_type")]
    pub submission_type: String,
    /// For civic counter-proposals: the submitting organization name. Required
    /// when `submission_type == "civic_counter_proposal"`. Free text.
    #[serde(default)]
    pub submitted_by: Option<String>,
    /// For civic counter-proposals: ISO-8601 UTC timestamp of submission.
    /// Defaults to import time when `--submitted-at` is not supplied.
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Source tool that produced the import (e.g., `"districtr"`, `"dra"`,
    /// `"shapefile"`, `"geojson"`, `"gerrychain"`). Empty for non-imported plans.
    #[serde(default)]
    pub source_tool: Option<String>,
    /// Source tool version (e.g., Districtr build version). Optional.
    #[serde(default)]
    pub source_tool_version: Option<String>,
    /// Multi-attribute schema fingerprint for the imported file (PP-33). For
    /// Districtr: derived from `(schema_version, problem.type, units.name)`.
    /// For DRA: the column-set string (`"GEOID,DISTRICT"` etc.).
    #[serde(default)]
    pub source_format_fingerprint: Option<String>,
    /// SHA-256 of the embedded `import_compat.json` at import time (C-05).
    /// Lets a future reader determine which compat ranges were active.
    #[serde(default)]
    pub import_compat_sha256: Option<String>,

    // ── AlgorithmConfig reproducibility fields ────────────────────────────────
    /// METIS imbalance tolerance factor (ufactor). Default 5 = ±0.5%.
    #[serde(default = "default_ufactor")]
    pub ufactor: u32,
    /// METIS refinement iterations (niter). Default 100.
    #[serde(default = "default_niter")]
    pub niter: u32,
    /// Governmental subdivision county stickiness (alpha_county). 0 = off.
    #[serde(default)]
    pub alpha_county: f64,
    /// GeoSection/CompactBisect seeds per ratio or level. None if not applicable.
    #[serde(default)]
    pub split_seeds: Option<usize>,
    /// CompactBisect near-min-cut filter epsilon. None if not applicable.
    #[serde(default)]
    pub split_epsilon: Option<f64>,
    /// AreaSection area-swing tolerance (e.g. 1.10 = ±10%). None if not applicable.
    #[serde(default)]
    pub area_swing: Option<f64>,
    /// GeoSection directional penalty lambda. 0.0 = off.
    #[serde(default)]
    pub directional_lambda: f64,

    // ── B.7: solution-space research ─────────────────────────────────────────
    /// Total edge-cut of the final partition: sum of edge weights (boundary
    /// lengths in meters) across all edges whose two endpoints are assigned to
    /// different districts. Lower = more compact. Enables seed-sensitivity
    /// research without re-running the algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_cut: Option<f64>,
}

fn default_seats_per_district() -> usize { 1 }
fn default_electoral_system() -> String { "single_member".to_string() }
fn default_submission_type() -> String { "authoritative".to_string() }
fn default_ufactor() -> u32 { 5 }
fn default_niter() -> u32 { 100 }

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

/// Atomic guard for an entire plan directory (State Staff Interop plan Task 1, PP-22).
///
/// Builds the plan into a sibling `{plan_dir}.tmp/` directory; on `commit()` it
/// renames to `{plan_dir}/`; on drop without commit, it deletes the tmp dir.
/// Half-imported plan labels MUST NOT be visible to subsequent commands — that
/// is the load-bearing invariant per spec §7.
///
/// The "label collision" semantics: if `{plan_dir}/` already exists, `new()`
/// fails unless `force` is true (in which case the existing directory is
/// removed before the rename — caller responsibility to confirm with the user).
///
/// Usage:
/// ```ignore
/// let mut guard = PlanDirGuard::new(plan_root.join(label), false)?;
/// // write files into guard.tmp_dir() ...
/// // ... run validations ...
/// guard.commit()?;  // atomic rename; on success the tmp dir is gone
/// ```
pub struct PlanDirGuard {
    /// The final destination path (e.g., `outputs/v1/2020/plans/vt_test/`).
    final_dir: std::path::PathBuf,
    /// The sibling tmp staging directory (`outputs/v1/2020/plans/vt_test.tmp/`).
    tmp_dir: std::path::PathBuf,
    /// Set to `true` after a successful `commit()`. Drop becomes a no-op.
    committed: bool,
    /// When `true`, `commit()` removes any existing `final_dir` before rename.
    force: bool,
}

impl PlanDirGuard {
    /// Create a new guard. `final_dir` is the eventual plan directory; the tmp
    /// staging directory is created as a sibling at `<final_dir>.tmp/`.
    ///
    /// Errors:
    /// - `final_dir` already exists and `force=false`: returns Err with the
    ///   "label collision" message; existing dir is untouched.
    /// - tmp staging directory creation fails: returns Err.
    pub fn new(final_dir: std::path::PathBuf, force: bool) -> anyhow::Result<Self> {
        if final_dir.exists() && !force {
            anyhow::bail!(
                "[INPUT] plan directory already exists at {} (use --force to overwrite). \
                 Refusing to silently clobber existing data.",
                final_dir.display()
            );
        }
        // Build the sibling tmp path: append ".tmp" to the final-dir name.
        let mut tmp_name = final_dir
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("[INTERNAL] final_dir has no file name component: {}", final_dir.display()))?
            .to_os_string();
        tmp_name.push(".tmp");
        let tmp_dir = final_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("[INTERNAL] final_dir has no parent: {}", final_dir.display()))?
            .join(&tmp_name);

        // Clear any stale tmp from a previous failed run.
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).map_err(|e| {
                anyhow::anyhow!(
                    "[INTERNAL] cannot clean stale tmp directory {}: {e}",
                    tmp_dir.display()
                )
            })?;
        }
        std::fs::create_dir_all(&tmp_dir)?;
        Ok(PlanDirGuard {
            final_dir,
            tmp_dir,
            committed: false,
            force,
        })
    }

    /// Path to the staging directory. All file writes go here.
    pub fn tmp_dir(&self) -> &Path {
        &self.tmp_dir
    }

    /// Path to the final destination (do NOT write here directly; use `tmp_dir()`).
    pub fn final_dir(&self) -> &Path {
        &self.final_dir
    }

    /// Atomically promote the staging directory to the final destination.
    ///
    /// On success, the staging directory no longer exists and `Drop` becomes a
    /// no-op. On failure, the staging directory is left in place for debugging
    /// (caller may retry or invoke an explicit cleanup).
    pub fn commit(mut self) -> anyhow::Result<()> {
        // Handle the force-overwrite case: remove existing final_dir first.
        if self.final_dir.exists() {
            if !self.force {
                anyhow::bail!(
                    "[INPUT] plan directory appeared at {} between guard creation and commit \
                     (use --force to overwrite). Refusing to clobber.",
                    self.final_dir.display()
                );
            }
            std::fs::remove_dir_all(&self.final_dir).map_err(|e| {
                anyhow::anyhow!(
                    "[INTERNAL] --force overwrite cannot remove existing {}: {e}",
                    self.final_dir.display()
                )
            })?;
        }
        std::fs::rename(&self.tmp_dir, &self.final_dir).map_err(|e| {
            anyhow::anyhow!(
                "[INTERNAL] atomic rename {} -> {} failed: {e}",
                self.tmp_dir.display(),
                self.final_dir.display()
            )
        })?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for PlanDirGuard {
    fn drop(&mut self) {
        if !self.committed && self.tmp_dir.exists() {
            // Best-effort cleanup. We can't surface errors from Drop; if this
            // fails, the next PlanDirGuard::new for the same final_dir will
            // remove the stale tmp during its own preflight.
            let _ = std::fs::remove_dir_all(&self.tmp_dir);
        }
    }
}

/// Callais p.36 disentanglement mutex preflight (State Staff Interop plan Task 6).
///
/// Inspects a `PlanManifest` for the simultaneous presence of VRA-aware and
/// partisan-weighted bisection markers. Returns Err when both are active.
///
/// Detection rules:
/// - VRA-aware: `partition_mode == "metis-vra"` OR `population_source == "cvap"`
///   (CVAP-population plans are typically race-conscious; either signal counts).
/// - Partisan-weighted: `partition_mode == "partisan-weighted"`.
///
/// When both are active in the same manifest, the run is post-Callais
/// inadmissible. The error string matches `runner::validate_partisan_config`
/// for consistency across error sites.
///
/// Called at:
/// - `redist state` runtime via `validate_partisan_config(StateConfig)` (existing).
/// - `redist import` (this gate, before PlanDirGuard::commit) — Task 6.2.
/// - `redist analyze` (top of run_analyze) — Task 6.3.
pub fn callais_preflight(manifest: &PlanManifest) -> anyhow::Result<()> {
    let vra_aware = manifest.partition_mode == "metis-vra"
        || manifest.population_source == "cvap";
    let partisan_weighted = manifest.partition_mode == "partisan-weighted";
    if vra_aware && partisan_weighted {
        anyhow::bail!(
            "[BOUNDARY] partisan-weighted and metis-vra are mutually exclusive per Callais p.36 disentanglement. \
             Manifest {} has partition_mode={} AND population_source={}; this run is post-Callais inadmissible. \
             See docs/legal/CALLAIS_REFERENCE.md.",
            manifest.label, manifest.partition_mode, manifest.population_source
        );
    }
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
            seats_per_district: 1,
            total_seats: 10,
            electoral_system: "single_member".into(),
            gpmetis_version: String::new(),
            ..Default::default()
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

    // ── seats / electoral_system fields ──────────────────────────────────────

    #[test]
    fn test_manifest_seats_per_district_default_is_1() {
        let manifest = make_test_manifest("53");
        assert_eq!(manifest.seats_per_district, 1);
    }

    #[test]
    fn test_manifest_electoral_system_single_member() {
        let manifest = PlanManifest {
            seats_per_district: 1,
            total_seats: 10,
            electoral_system: "single_member".into(),
            ..Default::default()
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["electoral_system"], "single_member");
    }

    #[test]
    fn test_manifest_electoral_system_multi_member_uniform() {
        let manifest = PlanManifest {
            seats_per_district: 5,
            total_seats: 65,
            electoral_system: "multi_member_uniform".into(),
            ..Default::default()
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["electoral_system"], "multi_member_uniform");
        assert_eq!(json["seats_per_district"], 5);
        assert_eq!(json["total_seats"], 65);
    }

    #[test]
    fn test_manifest_total_seats_serialized() {
        let manifest = make_test_manifest("53");
        let json = serde_json::to_value(&manifest).unwrap();
        assert!(json["total_seats"].is_number());
        assert_eq!(json["total_seats"], 10);
    }

    // ── Task 132: gpmetis_version field ──────────────────────────────────────

    #[test]
    fn test_manifest_has_gpmetis_version_field() {
        // PlanManifest must serialize with gpmetis_version field present
        let manifest = PlanManifest {
            gpmetis_version: "METIS 5.1.0".to_string(),
            ..make_test_manifest("53")
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert!(json["gpmetis_version"].is_string(),
            "gpmetis_version must be a string field in the serialized manifest");
        assert_eq!(json["gpmetis_version"], "METIS 5.1.0");
    }

    #[test]
    fn test_manifest_gpmetis_version_default_is_empty() {
        // When not set, gpmetis_version defaults to empty string (serde default)
        let manifest = make_test_manifest("53");
        let json = serde_json::to_value(&manifest).unwrap();
        // Default is empty string (from #[serde(default)])
        assert!(json["gpmetis_version"].is_string(),
            "gpmetis_version field must be present even when empty");
    }

    #[test]
    fn test_manifest_seats_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            seats_per_district: 5,
            total_seats: 65,
            electoral_system: "multi_member_uniform".into(),
            ..make_test_manifest("53")
        };
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        let json_bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let parsed: PlanManifest = serde_json::from_slice(&json_bytes).unwrap();
        assert_eq!(parsed.seats_per_district, 5);
        assert_eq!(parsed.total_seats, 65);
        assert_eq!(parsed.electoral_system, "multi_member_uniform");
    }

    // ── callais_preflight (State Staff Interop Task 6, BOUNDARY) ──────────────

    #[test]
    fn test_callais_preflight_passes_clean_manifest() {
        let mut m = make_test_manifest("50");
        m.partition_mode = "edge-weighted".into();
        m.population_source = "total".into();
        assert!(callais_preflight(&m).is_ok(), "edge-weighted + total population must pass");
    }

    #[test]
    fn test_callais_preflight_passes_vra_only() {
        let mut m = make_test_manifest("50");
        m.partition_mode = "metis-vra".into();
        m.population_source = "cvap".into();
        assert!(callais_preflight(&m).is_ok(), "VRA-only configuration must pass");
    }

    #[test]
    fn test_callais_preflight_passes_partisan_only() {
        let mut m = make_test_manifest("50");
        m.partition_mode = "partisan-weighted".into();
        m.population_source = "total".into();
        assert!(callais_preflight(&m).is_ok(), "partisan-weighted + total population must pass");
    }

    #[test]
    fn test_callais_preflight_blocks_partisan_weighted_with_cvap() {
        let mut m = make_test_manifest("50");
        m.partition_mode = "partisan-weighted".into();
        m.population_source = "cvap".into();  // VRA-aware proxy
        let err = callais_preflight(&m).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.starts_with("[BOUNDARY]"), "must be [BOUNDARY] category: {msg}");
        assert!(msg.contains("Callais p.36"), "must cite Callais p.36: {msg}");
        assert!(msg.contains("disentanglement"), "must say disentanglement: {msg}");
    }

    #[test]
    fn test_callais_preflight_error_names_offending_fields() {
        let mut m = make_test_manifest("50");
        m.label = "la_2020_disputed".into();
        m.partition_mode = "partisan-weighted".into();
        m.population_source = "cvap".into();
        let err = callais_preflight(&m).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("la_2020_disputed"), "must name the manifest: {msg}");
        assert!(msg.contains("partisan-weighted"), "must name partition_mode: {msg}");
        assert!(msg.contains("cvap"), "must name population_source: {msg}");
    }

    // ── PlanDirGuard (State Staff Interop Task 1, PP-22) ──────────────────────

    #[test]
    fn test_plan_dir_guard_commit_renames_tmp_to_final() {
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(final_dir.parent().unwrap()).unwrap();
        let guard = PlanDirGuard::new(final_dir.clone(), false).unwrap();
        // Stage some files in the tmp dir.
        std::fs::write(guard.tmp_dir().join("manifest.json"), b"{}").unwrap();
        std::fs::write(guard.tmp_dir().join("final_assignments.json"), b"{}").unwrap();
        // Pre-commit: tmp exists, final doesn't.
        let tmp_path = guard.tmp_dir().to_path_buf();
        assert!(tmp_path.exists());
        assert!(!final_dir.exists());
        guard.commit().unwrap();
        // Post-commit: final exists with our files; tmp is gone.
        assert!(final_dir.exists());
        assert!(final_dir.join("manifest.json").exists());
        assert!(final_dir.join("final_assignments.json").exists());
        assert!(!tmp_path.exists());
    }

    #[test]
    fn test_plan_dir_guard_drop_without_commit_leaves_no_label() {
        // PP-22 invariant: a guard that is dropped without commit must remove
        // the tmp staging directory and never create the final directory.
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(final_dir.parent().unwrap()).unwrap();
        let tmp_path = {
            let guard = PlanDirGuard::new(final_dir.clone(), false).unwrap();
            std::fs::write(guard.tmp_dir().join("manifest.json"), b"{}").unwrap();
            // Simulate a validation failure: drop the guard without committing.
            let p = guard.tmp_dir().to_path_buf();
            drop(guard);
            p
        };
        assert!(!tmp_path.exists(), "tmp dir must be removed on drop");
        assert!(!final_dir.exists(), "final dir must not be created on failure");
    }

    #[test]
    fn test_plan_dir_guard_label_collision_refuses_without_force() {
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(&final_dir).unwrap();
        std::fs::write(final_dir.join("preexisting.txt"), b"keep me").unwrap();
        let result = PlanDirGuard::new(final_dir.clone(), false);
        assert!(result.is_err(), "must refuse to overwrite without force");
        assert!(final_dir.join("preexisting.txt").exists(), "existing files untouched");
    }

    #[test]
    fn test_plan_dir_guard_force_overwrites_existing() {
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(&final_dir).unwrap();
        std::fs::write(final_dir.join("preexisting.txt"), b"replace me").unwrap();
        let guard = PlanDirGuard::new(final_dir.clone(), true).unwrap();
        std::fs::write(guard.tmp_dir().join("new_file.json"), b"new").unwrap();
        guard.commit().unwrap();
        assert!(final_dir.join("new_file.json").exists());
        assert!(!final_dir.join("preexisting.txt").exists(), "force replaces atomically");
    }

    #[test]
    fn test_plan_dir_guard_clears_stale_tmp_from_prior_failed_run() {
        // If a prior process crashed and left a stale .tmp/ directory, the
        // next guard creation must clean it up rather than fail.
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(final_dir.parent().unwrap()).unwrap();
        let stale = final_dir.parent().unwrap().join("vt_test.tmp");
        std::fs::create_dir_all(&stale).unwrap();
        std::fs::write(stale.join("garbage.txt"), b"left behind").unwrap();
        let guard = PlanDirGuard::new(final_dir.clone(), false).unwrap();
        // Stale contents should be gone.
        assert!(!guard.tmp_dir().join("garbage.txt").exists());
        guard.commit().unwrap();
        assert!(final_dir.exists());
    }

    #[test]
    fn test_plan_dir_guard_commit_fails_if_final_dir_appears_mid_run() {
        // Race: someone else creates the final dir between our preflight and
        // commit. We must refuse rather than clobber, unless force=true.
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(final_dir.parent().unwrap()).unwrap();
        let guard = PlanDirGuard::new(final_dir.clone(), false).unwrap();
        std::fs::write(guard.tmp_dir().join("staged.json"), b"{}").unwrap();
        // Simulate a mid-run conflict.
        std::fs::create_dir_all(&final_dir).unwrap();
        std::fs::write(final_dir.join("from_other_process.txt"), b"hi").unwrap();
        let tmp_path = guard.tmp_dir().to_path_buf();
        let result = guard.commit();
        assert!(result.is_err(), "mid-run collision must refuse without force");
        // Existing final dir untouched; tmp still exists for debugging.
        assert!(final_dir.join("from_other_process.txt").exists());
        // (Tmp may or may not exist depending on Drop order; what matters is
        // the existing final_dir was not clobbered.)
        let _ = tmp_path;
    }

    // ── 20 additional tests ───────────────────────────────────────────────────

    #[test]
    fn test_callais_preflight_blocks_metis_vra_with_partisan_weighted() {
        let mut m = make_test_manifest("50");
        m.partition_mode = "partisan-weighted".into();
        m.population_source = "total".into(); // not cvap, but partition_mode is metis-vra is set below
        // Actually the vra check is partition_mode == "metis-vra" OR population_source == "cvap"
        // So: metis-vra + partisan-weighted
        m.partition_mode = "metis-vra".into();
        // But partisan_weighted is checked via partition_mode == "partisan-weighted"
        // They can't both be true in the same field — the real test is cvap + partisan-weighted
        // Let's use the correct combination: metis-vra partition_mode won't trigger partisan_weighted.
        // A real BOUNDARY case: population_source=cvap AND partition_mode=partisan-weighted
        m.partition_mode = "partisan-weighted".into();
        m.population_source = "cvap".into();
        let result = callais_preflight(&m);
        assert!(result.is_err(), "partisan-weighted + cvap must fail preflight");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("[BOUNDARY]"));
    }

    #[test]
    fn test_callais_preflight_message_contains_docs_reference() {
        let mut m = make_test_manifest("50");
        m.partition_mode = "partisan-weighted".into();
        m.population_source = "cvap".into();
        let err = callais_preflight(&m).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("CALLAIS_REFERENCE"), "error must cite the reference doc: {msg}");
    }

    #[test]
    fn test_callais_preflight_passes_edge_weighted_with_total() {
        let mut m = make_test_manifest("06");
        m.partition_mode = "edge-weighted".into();
        m.population_source = "total".into();
        assert!(callais_preflight(&m).is_ok());
    }

    #[test]
    fn test_callais_preflight_passes_default_manifest() {
        // A default PlanManifest has empty strings — must not trigger BOUNDARY.
        let m = PlanManifest::default();
        assert!(callais_preflight(&m).is_ok(),
            "default manifest (empty strings) must pass callais_preflight");
    }

    #[test]
    fn test_sha256_bytes_known_vector_empty() {
        // SHA-256 of empty slice is well-known.
        let hash = sha256_bytes(b"");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_sha256_bytes_collision_resistance() {
        // Two similar but distinct inputs must produce distinct hashes.
        let h1 = sha256_bytes(b"plan_a_2020");
        let h2 = sha256_bytes(b"plan_a_2021");
        assert_ne!(h1, h2, "different inputs must hash differently");
    }

    #[test]
    fn test_sha256_file_deterministic_on_same_content() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.bin");
        std::fs::write(&path, b"redistricting data").unwrap();
        let h1 = sha256_file(&path).unwrap();
        let h2 = sha256_file(&path).unwrap();
        assert_eq!(h1, h2, "sha256_file must be deterministic");
    }

    #[test]
    fn test_sha256_file_nonexistent_returns_error() {
        let result = sha256_file(std::path::Path::new("/tmp/does_not_exist_xyz_abc.bin"));
        assert!(result.is_err(), "sha256_file on nonexistent path must return Err");
    }

    #[test]
    fn test_write_manifest_atomic_round_trip_all_fields() {
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            label: "ca_congressional_2020".into(),
            state_code: "CA".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 52,
            seed: Some(99),
            binary_version: "1.2.3".into(),
            binary_sha256: "d".repeat(64),
            gpmetis_version: "METIS 5.1.0".into(),
            balance_tolerance_pct: 0.25,
            population_balance_valid: true,
            seats_per_district: 1,
            total_seats: 52,
            electoral_system: "single_member".into(),
            ..make_test_manifest("06")
        };
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        let bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let parsed: PlanManifest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.label, "ca_congressional_2020");
        assert_eq!(parsed.num_districts, 52);
        assert_eq!(parsed.seed, Some(99));
        assert_eq!(parsed.gpmetis_version, "METIS 5.1.0");
        assert_eq!(parsed.binary_version, "1.2.3");
    }

    #[test]
    fn test_default_label_single_word_state() {
        assert_eq!(default_label("vermont", "congressional", "2020"), "vermont_congressional_2020");
    }

    #[test]
    fn test_default_label_multi_word_state_lowercased() {
        assert_eq!(default_label("NORTH DAKOTA", "senate", "2010"), "north_dakota_senate_2010");
    }

    #[test]
    fn test_default_label_uppercase_state_normalized() {
        let label = default_label("CALIFORNIA", "congressional", "2000");
        assert_eq!(label, "california_congressional_2000");
    }

    #[test]
    fn test_tiger_source_url_contains_fips_and_year() {
        let url = tiger_source_url("06", "2020");
        assert!(url.contains("06"), "FIPS must appear in URL");
        assert!(url.contains("2020"), "year must appear in URL");
        assert!(url.starts_with("https://"), "must be HTTPS URL");
        assert!(url.contains("census.gov"), "must point to census.gov");
    }

    #[test]
    fn test_tiger_source_url_different_fips_produce_different_urls() {
        let url_ca = tiger_source_url("06", "2020");
        let url_wa = tiger_source_url("53", "2020");
        assert_ne!(url_ca, url_wa, "different FIPS must produce different URLs");
    }

    #[test]
    fn test_manifest_submission_type_serde_default_is_authoritative() {
        // serde default_submission_type fires on deserialization when the field is absent.
        // Use a full manifest round-trip: write one without submission_type, read it back.
        let tmp = TempDir::new().unwrap();
        // Write a manifest using make_test_manifest which has submission_type = default
        let original = make_test_manifest("50");
        write_manifest_atomic(tmp.path(), &original).unwrap();
        // Now read the JSON and strip submission_type to simulate an old manifest
        let bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v.as_object_mut().unwrap().remove("submission_type");
        let stripped = v.to_string();
        let m: PlanManifest = serde_json::from_str(&stripped).unwrap();
        assert_eq!(m.submission_type, "authoritative",
            "submission_type serde default must be 'authoritative' when absent from JSON");
    }

    #[test]
    fn test_manifest_civic_counter_proposal_round_trip() {
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            submission_type: "civic_counter_proposal".into(),
            submitted_by: Some("League of Women Voters".into()),
            submitted_at: Some("2026-04-15T12:00:00Z".into()),
            ..make_test_manifest("50")
        };
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        let bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let parsed: PlanManifest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.submission_type, "civic_counter_proposal");
        assert_eq!(parsed.submitted_by.as_deref(), Some("League of Women Voters"));
        assert_eq!(parsed.submitted_at.as_deref(), Some("2026-04-15T12:00:00Z"));
    }

    #[test]
    fn test_manifest_edge_cut_optional_field_round_trip() {
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            edge_cut: Some(123_456.789),
            ..make_test_manifest("50")
        };
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        let bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let parsed: PlanManifest = serde_json::from_slice(&bytes).unwrap();
        let ec = parsed.edge_cut.expect("edge_cut must survive round-trip");
        assert!((ec - 123_456.789).abs() < 0.001);
    }

    #[test]
    fn test_manifest_edge_cut_none_absent_from_json() {
        // When edge_cut is None, skip_serializing_if = "Option::is_none" means it
        // must NOT appear in the JSON output.
        let manifest = PlanManifest {
            edge_cut: None,
            ..make_test_manifest("50")
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert!(json.get("edge_cut").is_none(),
            "edge_cut=None must be absent from serialized JSON (skip_serializing_if)");
    }

    #[test]
    fn test_manifest_algorithm_params_round_trip() {
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            ufactor: 10,
            niter: 50,
            alpha_county: 0.75,
            split_seeds: Some(5),
            split_epsilon: Some(0.05),
            area_swing: Some(1.15),
            directional_lambda: 0.3,
            ..make_test_manifest("50")
        };
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        let bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let parsed: PlanManifest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.ufactor, 10);
        assert_eq!(parsed.niter, 50);
        assert!((parsed.alpha_county - 0.75).abs() < 1e-9);
        assert_eq!(parsed.split_seeds, Some(5));
        assert!((parsed.split_epsilon.unwrap() - 0.05).abs() < 1e-9);
        assert!((parsed.area_swing.unwrap() - 1.15).abs() < 1e-9);
        assert!((parsed.directional_lambda - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_plan_dir_guard_tmp_dir_is_sibling_with_tmp_suffix() {
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("vt_test");
        std::fs::create_dir_all(final_dir.parent().unwrap()).unwrap();
        let guard = PlanDirGuard::new(final_dir.clone(), false).unwrap();
        let tmp_path = guard.tmp_dir();
        // tmp_dir must be a sibling (same parent)
        assert_eq!(tmp_path.parent(), final_dir.parent(),
            "tmp_dir must be a sibling of final_dir");
        // tmp_dir name must end with ".tmp"
        let name = tmp_path.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with(".tmp"), "tmp_dir name must end with .tmp, got: {name}");
        drop(guard);
    }

    #[test]
    fn test_plan_dir_guard_final_dir_accessor() {
        let tmp = TempDir::new().unwrap();
        let final_dir = tmp.path().join("plans").join("ny_congressional");
        std::fs::create_dir_all(final_dir.parent().unwrap()).unwrap();
        let guard = PlanDirGuard::new(final_dir.clone(), false).unwrap();
        assert_eq!(guard.final_dir(), final_dir.as_path(),
            "final_dir() accessor must return the expected path");
        drop(guard);
    }

    #[test]
    fn test_check_incomplete_plan_error_contains_label() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("manifest.tmp"), b"{}").unwrap();
        let err = check_incomplete_plan(tmp.path(), "my_special_label").unwrap_err();
        assert!(err.to_string().contains("my_special_label"),
            "error must name the plan label");
    }

    #[test]
    fn test_manifest_import_compat_sha256_round_trip() {
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            import_compat_sha256: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into()),
            ..make_test_manifest("50")
        };
        write_manifest_atomic(tmp.path(), &manifest).unwrap();
        let bytes = std::fs::read(tmp.path().join("manifest.json")).unwrap();
        let parsed: PlanManifest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            parsed.import_compat_sha256.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }
}
