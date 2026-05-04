//! Comparison report data structures (Plan Comparison plan Task 2).
//!
//! Defines the `ComparisonReport` struct that the narrative renderer consumes.
//! This module is the data layer; rendering lives in `narrative.rs`.
//!
//! Minimal scope for this commit: structures + a `from_pre_loaded` constructor.
//! The full assembler that reads analysis JSONs from disk + computes manifest
//! SHAs is the next session's pickup (it requires reaching into the
//! existing `redist-cli` plan-context module which is a cross-crate dance).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use sha2::{Digest, Sha256};

/// One side of the comparison: the data we need from a single plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSide {
    pub label: String,
    pub manifest_sha256: String,
    /// Number of seats won by the named threshold-defining party (typically Dem).
    pub leaning_seats: usize,
    /// Total district count.
    pub n_districts: usize,
    /// Per-district Dem-share point estimates (length n_districts). Used by
    /// the close-call detector + the threshold-classification narrative.
    pub per_district_dem_share: Vec<f64>,
    /// Number of majority-minority districts (BVAP > 50%).
    pub mm_count: usize,
    /// Mean Polsby-Popper compactness across districts.
    pub mean_pp: f64,
    /// Total population.
    pub total_population: u64,
    /// Optional civic-counter-proposal tag passthrough from the plan's
    /// PlanManifest.submission_type. Drives the framing label in the narrative.
    #[serde(default)]
    pub submission_type: Option<String>,
    /// Submitter (when civic counter-proposal).
    #[serde(default)]
    pub submitted_by: Option<String>,
    /// Submission timestamp (when civic counter-proposal).
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Per-analysis-file SHA-256s (e.g., {"partisan.json": "..."}).
    /// Used by the narrative_manifest writer.
    pub analysis_sha256: BTreeMap<String, String>,
}

/// Diff between two plans: which tracts moved, how many people, which districts changed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    pub tracts_changed: usize,
    pub population_changed: u64,
    /// Sorted list of district numbers (in plan A's labeling) where any tract
    /// was reassigned.
    pub districts_with_changes: Vec<usize>,
}

/// Top-level comparison report consumed by the narrative renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub plan_a: PlanSide,
    pub plan_b: PlanSide,
    pub baseline: Option<PlanSide>,
    pub diff: DiffSummary,
}

impl ComparisonReport {
    /// Construct from already-loaded plan sides. Used by tests and by the
    /// CLI dispatcher once it's wired (Task 11).
    pub fn from_loaded(
        plan_a: PlanSide,
        plan_b: PlanSide,
        baseline: Option<PlanSide>,
        diff: DiffSummary,
    ) -> Self {
        ComparisonReport {
            plan_a,
            plan_b,
            baseline,
            diff,
        }
    }

    /// True iff either plan carries the civic-counter-proposal tag.
    pub fn has_civic_counter_proposal(&self) -> bool {
        is_civic(&self.plan_a) || is_civic(&self.plan_b)
    }
}

fn is_civic(p: &PlanSide) -> bool {
    p.submission_type.as_deref() == Some("civic_counter_proposal")
}

// ---------------------------------------------------------------------------
// From-disk loader (Plan Comparison plan Task 11 / 2.2)
// ---------------------------------------------------------------------------

/// Errors from the from-disk assembler.
#[derive(Debug, thiserror::Error)]
pub enum AssembleError {
    #[error("[INPUT] plan directory not found: {0}")]
    PlanDirNotFound(String),
    #[error("[INPUT] plan manifest not found at {0}")]
    ManifestNotFound(String),
    #[error("[INPUT] plan manifest at {path}: {reason}")]
    ManifestInvalid { path: String, reason: String },
    #[error("[INPUT] cannot read {0}: {1}")]
    Io(String, String),
    #[error("[INPUT] cannot parse {0}: {1}")]
    JsonParse(String, String),
}

/// Build a `PlanSide` by reading `manifest.json` + selected analysis JSONs
/// from a plan directory. The leaning-seat count is derived by counting
/// districts whose `dem_share >= leaning_threshold` in `analysis/partisan.json`;
/// missing analysis JSONs leave the corresponding fields at their defaults
/// (NaN / 0) so the renderer can still produce output (the manifest will
/// record an empty SHA for the missing file).
///
/// `plan_dir` is the directory containing `manifest.json` (e.g.,
/// `outputs/v1/2020/plans/vt_2020/`).
pub fn load_plan_side_from_dir(
    plan_dir: &Path,
    leaning_threshold: f64,
) -> Result<PlanSide, AssembleError> {
    if !plan_dir.is_dir() {
        return Err(AssembleError::PlanDirNotFound(
            plan_dir.display().to_string(),
        ));
    }

    // Load manifest.json + compute its SHA-256.
    let manifest_path = plan_dir.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(AssembleError::ManifestNotFound(
            manifest_path.display().to_string(),
        ));
    }
    let manifest_bytes = std::fs::read(&manifest_path).map_err(|e| {
        AssembleError::Io(manifest_path.display().to_string(), e.to_string())
    })?;
    let manifest_sha = sha256_hex(&manifest_bytes);
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes).map_err(|e| {
        AssembleError::JsonParse(manifest_path.display().to_string(), e.to_string())
    })?;

    let label = manifest
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            plan_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        })
        .to_string();
    let n_districts = manifest
        .get("num_districts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let total_population = 0u64; // manifest doesn't carry it; would need analysis/summary.json
    let submission_type = manifest
        .get("submission_type")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let submitted_by = manifest
        .get("submitted_by")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let submitted_at = manifest
        .get("submitted_at")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    // Load analysis JSONs (best-effort; missing files leave fields at defaults).
    let mut analysis_sha: BTreeMap<String, String> = BTreeMap::new();
    let analysis_dir = plan_dir.join("analysis");

    let (leaning_seats, per_district_dem_share) =
        load_partisan_seats(&analysis_dir, leaning_threshold, &mut analysis_sha);
    let mm_count = load_mm_count(&analysis_dir, &mut analysis_sha);
    let mean_pp = load_mean_pp(&analysis_dir, &mut analysis_sha);

    Ok(PlanSide {
        label,
        manifest_sha256: manifest_sha,
        leaning_seats,
        n_districts,
        per_district_dem_share,
        mm_count,
        mean_pp,
        total_population,
        submission_type,
        submitted_by,
        submitted_at,
        analysis_sha256: analysis_sha,
    })
}

/// Compute SHA-256 of `path` if it exists; record under `key` in `sha_map`
/// and return the file bytes parsed as JSON. Returns None if the file is
/// absent (caller treats as "field unknown, use default").
fn load_analysis_json(
    analysis_dir: &Path,
    filename: &str,
    sha_map: &mut BTreeMap<String, String>,
) -> Option<serde_json::Value> {
    let path = analysis_dir.join(filename);
    if !path.is_file() {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    sha_map.insert(filename.to_string(), sha256_hex(&bytes));
    serde_json::from_slice(&bytes).ok()
}

fn load_partisan_seats(
    analysis_dir: &Path,
    leaning_threshold: f64,
    sha_map: &mut BTreeMap<String, String>,
) -> (usize, Vec<f64>) {
    let Some(partisan) = load_analysis_json(analysis_dir, "partisan.json", sha_map) else {
        return (0, Vec::new());
    };
    // Try several common shapes for the per-district Dem-share array.
    // Shape 1: { "districts": [{"dem_share": 0.55}, ...] }
    // Shape 2: { "per_district_dem_share": [0.55, 0.42, ...] }
    // Shape 3: top-level array of {dem_share}
    let dem_shares: Vec<f64> = if let Some(arr) = partisan.get("districts").and_then(|v| v.as_array()) {
        arr.iter()
            .filter_map(|d| d.get("dem_share").and_then(|x| x.as_f64()))
            .collect()
    } else if let Some(arr) = partisan
        .get("per_district_dem_share")
        .and_then(|v| v.as_array())
    {
        arr.iter().filter_map(|x| x.as_f64()).collect()
    } else if let Some(arr) = partisan.as_array() {
        arr.iter()
            .filter_map(|d| d.get("dem_share").and_then(|x| x.as_f64()))
            .collect()
    } else {
        Vec::new()
    };
    let leaning_seats = dem_shares.iter().filter(|&&s| s >= leaning_threshold).count();
    (leaning_seats, dem_shares)
}

fn load_mm_count(analysis_dir: &Path, sha_map: &mut BTreeMap<String, String>) -> usize {
    let Some(vra) = load_analysis_json(analysis_dir, "vra_analysis.json", sha_map) else {
        return 0;
    };
    vra.get("mm_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize
}

fn load_mean_pp(analysis_dir: &Path, sha_map: &mut BTreeMap<String, String>) -> f64 {
    let Some(comp) = load_analysis_json(analysis_dir, "compactness.json", sha_map) else {
        return f64::NAN;
    };
    // Try shapes:
    // 1. { "mean_polsby_popper": 0.42 }
    // 2. { "districts": [{"polsby_popper": 0.4}, ...] } -> compute mean
    if let Some(v) = comp.get("mean_polsby_popper").and_then(|v| v.as_f64()) {
        return v;
    }
    if let Some(arr) = comp.get("districts").and_then(|v| v.as_array()) {
        let pps: Vec<f64> = arr
            .iter()
            .filter_map(|d| d.get("polsby_popper").and_then(|x| x.as_f64()))
            .collect();
        if !pps.is_empty() {
            return pps.iter().sum::<f64>() / (pps.len() as f64);
        }
    }
    f64::NAN
}

/// Compute a basic `DiffSummary` between two plans by reading their
/// `final_assignments.json` files. Returns zeros when either file is missing.
pub fn diff_from_assignments(
    plan_a_dir: &Path,
    plan_b_dir: &Path,
) -> Result<DiffSummary, AssembleError> {
    let a = load_assignments(plan_a_dir)?;
    let b = load_assignments(plan_b_dir)?;
    if a.is_empty() || b.is_empty() {
        return Ok(DiffSummary::default());
    }
    let mut tracts_changed = 0usize;
    let mut districts_with_changes = std::collections::BTreeSet::<usize>::new();
    for (geoid, dist_a) in &a {
        if let Some(dist_b) = b.get(geoid) {
            if dist_a != dist_b {
                tracts_changed += 1;
                districts_with_changes.insert(*dist_a);
                districts_with_changes.insert(*dist_b);
            }
        }
    }
    Ok(DiffSummary {
        tracts_changed,
        population_changed: 0, // would need per-tract population data
        districts_with_changes: districts_with_changes.into_iter().collect(),
    })
}

fn load_assignments(plan_dir: &Path) -> Result<BTreeMap<String, usize>, AssembleError> {
    // Production layout: plan_dir/data/final_assignments.json (matches PlanContext::assignments_path).
    // Fallback: plan_dir/final_assignments.json (legacy/flat tests).
    let primary = plan_dir.join("data").join("final_assignments.json");
    let fallback = plan_dir.join("final_assignments.json");
    let path = if primary.is_file() {
        primary
    } else if fallback.is_file() {
        fallback
    } else {
        return Ok(BTreeMap::new());
    };
    let bytes = std::fs::read(&path)
        .map_err(|e| AssembleError::Io(path.display().to_string(), e.to_string()))?;
    let v: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| AssembleError::JsonParse(path.display().to_string(), e.to_string()))?;
    let mut out = BTreeMap::new();
    if let Some(obj) = v.as_object() {
        for (geoid, district) in obj {
            if let Some(d) = district.as_u64() {
                out.insert(geoid.clone(), d as usize);
            }
        }
    }
    Ok(out)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let mut s = String::with_capacity(64);
    for b in h.finalize() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_plan_a() -> PlanSide {
        PlanSide {
            label: "vt_state_proposal".into(),
            manifest_sha256: "a".repeat(64),
            leaning_seats: 5,
            n_districts: 6,
            per_district_dem_share: vec![0.30, 0.45, 0.55, 0.60, 0.65, 0.70],
            mm_count: 1,
            mean_pp: 0.42,
            total_population: 600_000,
            submission_type: None,
            submitted_by: None,
            submitted_at: None,
            analysis_sha256: BTreeMap::new(),
        }
    }
    fn fixture_plan_b() -> PlanSide {
        PlanSide {
            label: "vt_civic_alt".into(),
            manifest_sha256: "b".repeat(64),
            leaning_seats: 4,
            n_districts: 6,
            per_district_dem_share: vec![0.25, 0.40, 0.547, 0.553, 0.62, 0.68],
            mm_count: 1,
            mean_pp: 0.40,
            total_population: 600_000,
            submission_type: Some("civic_counter_proposal".into()),
            submitted_by: Some("League of Women Voters Vermont".into()),
            submitted_at: Some("2026-04-15T12:00:00Z".into()),
            analysis_sha256: BTreeMap::new(),
        }
    }

    #[test]
    fn test_has_civic_counter_proposal_detects_b() {
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(),
            fixture_plan_b(),
            None,
            DiffSummary::default(),
        );
        assert!(report.has_civic_counter_proposal());
    }

    #[test]
    fn test_has_civic_counter_proposal_false_when_neither() {
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(),
            fixture_plan_a(),
            None,
            DiffSummary::default(),
        );
        assert!(!report.has_civic_counter_proposal());
    }

    #[test]
    fn test_diff_summary_default_is_empty() {
        let d = DiffSummary::default();
        assert_eq!(d.tracts_changed, 0);
        assert_eq!(d.population_changed, 0);
        assert!(d.districts_with_changes.is_empty());
    }

    // ── ComparisonReport: from_loaded / structural tests ─────────────────────

    #[test]
    fn test_from_loaded_stores_plan_a() {
        let a = fixture_plan_a();
        let b = fixture_plan_b();
        let report = ComparisonReport::from_loaded(a.clone(), b, None, DiffSummary::default());
        assert_eq!(report.plan_a.label, a.label);
        assert_eq!(report.plan_a.leaning_seats, a.leaning_seats);
    }

    #[test]
    fn test_from_loaded_stores_plan_b() {
        let a = fixture_plan_a();
        let b = fixture_plan_b();
        let report = ComparisonReport::from_loaded(a, b.clone(), None, DiffSummary::default());
        assert_eq!(report.plan_b.label, b.label);
        assert_eq!(report.plan_b.submission_type, b.submission_type);
    }

    #[test]
    fn test_from_loaded_baseline_none() {
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(), fixture_plan_b(), None, DiffSummary::default()
        );
        assert!(report.baseline.is_none());
    }

    #[test]
    fn test_from_loaded_baseline_some() {
        let baseline = fixture_plan_a();
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(), fixture_plan_b(), Some(baseline.clone()), DiffSummary::default()
        );
        assert!(report.baseline.is_some());
        assert_eq!(report.baseline.as_ref().unwrap().label, baseline.label);
    }

    #[test]
    fn test_has_civic_detects_plan_a_civic() {
        let mut a = fixture_plan_a();
        a.submission_type = Some("civic_counter_proposal".into());
        let report = ComparisonReport::from_loaded(a, fixture_plan_b(), None, DiffSummary::default());
        // plan_b is also civic in fixture, but let's use a non-civic plan_b to isolate.
        assert!(report.has_civic_counter_proposal(), "civic plan_a must be detected");
    }

    #[test]
    fn test_has_civic_false_neither_civic() {
        let mut a = fixture_plan_a();
        let mut b = fixture_plan_b();
        a.submission_type = None;
        b.submission_type = None;
        let report = ComparisonReport::from_loaded(a, b, None, DiffSummary::default());
        assert!(!report.has_civic_counter_proposal());
    }

    // ── PlanSide fields ──────────────────────────────────────────────────────

    #[test]
    fn test_plan_side_per_district_dem_share_length() {
        let plan = fixture_plan_a();
        assert_eq!(plan.per_district_dem_share.len(), plan.n_districts,
            "per_district_dem_share length must equal n_districts");
    }

    #[test]
    fn test_plan_side_manifest_sha256_length() {
        let plan = fixture_plan_a();
        assert_eq!(plan.manifest_sha256.len(), 64, "SHA-256 hex must be 64 chars");
    }

    #[test]
    fn test_diff_summary_population_changed_round_trip() {
        let d = DiffSummary {
            tracts_changed: 10,
            population_changed: 25_000,
            districts_with_changes: vec![1, 3, 5],
        };
        assert_eq!(d.tracts_changed, 10);
        assert_eq!(d.population_changed, 25_000);
        assert_eq!(d.districts_with_changes, vec![1, 3, 5]);
    }

    #[test]
    fn test_sha256_hex_length() {
        // The sha256_hex helper must produce a 64-char lowercase hex string.
        let hash = sha256_hex(b"hello world");
        assert_eq!(hash.len(), 64, "SHA-256 hex must be 64 chars, got {}", hash.len());
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 hex must be lowercase hex: {hash}");
    }

    #[test]
    fn test_sha256_hex_deterministic() {
        let h1 = sha256_hex(b"redistricting");
        let h2 = sha256_hex(b"redistricting");
        assert_eq!(h1, h2, "SHA-256 must be deterministic");
    }

    #[test]
    fn test_sha256_hex_different_inputs() {
        let h1 = sha256_hex(b"plan_a");
        let h2 = sha256_hex(b"plan_b");
        assert_ne!(h1, h2, "different inputs must produce different hashes");
    }

    // ── 15 additional tests for comparison ────────────────────────────────────

    #[test]
    fn test_comparison_report_serializes_to_json() {
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(), fixture_plan_b(), None, DiffSummary::default()
        );
        let json = serde_json::to_value(&report);
        assert!(json.is_ok(), "ComparisonReport must serialize to JSON");
        let v = json.unwrap();
        assert!(v["plan_a"].is_object());
        assert!(v["plan_b"].is_object());
    }

    #[test]
    fn test_plan_side_submission_type_civic_round_trips() {
        let plan = fixture_plan_b();
        let json = serde_json::to_value(&plan).unwrap();
        assert_eq!(json["submission_type"], "civic_counter_proposal");
        assert_eq!(json["submitted_by"], "League of Women Voters Vermont");
    }

    #[test]
    fn test_plan_side_submission_type_none_serializes_as_null() {
        let plan = fixture_plan_a();
        let json = serde_json::to_value(&plan).unwrap();
        assert!(json["submission_type"].is_null(), "None submission_type must serialize as null");
    }

    #[test]
    fn test_diff_summary_districts_with_changes_sorted() {
        // DiffSummary guarantees sorted list per spec comment.
        let d = DiffSummary {
            tracts_changed: 5,
            population_changed: 50_000,
            districts_with_changes: vec![1, 2, 3, 7, 9],
        };
        let sorted: Vec<usize> = {
            let mut v = d.districts_with_changes.clone();
            v.sort();
            v
        };
        assert_eq!(d.districts_with_changes, sorted,
            "districts_with_changes must be in ascending order");
    }

    #[test]
    fn test_leaning_seats_count_at_threshold() {
        // Seats with dem_share exactly at threshold count as leaning.
        let plan = PlanSide {
            per_district_dem_share: vec![0.50, 0.50, 0.50, 0.49, 0.49],
            leaning_seats: 3,
            n_districts: 5,
            ..fixture_plan_a()
        };
        assert_eq!(plan.leaning_seats, 3,
            "districts at exactly 0.50 should be counted as leaning (>= threshold)");
    }

    #[test]
    fn test_comparison_report_diff_tracts_changed_stored() {
        let diff = DiffSummary {
            tracts_changed: 42,
            population_changed: 100_000,
            districts_with_changes: vec![1, 5, 10],
        };
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(), fixture_plan_b(), None, diff.clone()
        );
        assert_eq!(report.diff.tracts_changed, 42);
        assert_eq!(report.diff.population_changed, 100_000);
        assert_eq!(report.diff.districts_with_changes, vec![1, 5, 10]);
    }

    #[test]
    fn test_load_plan_side_from_dir_missing_dir_returns_error() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("nonexistent_plan");
        let result = load_plan_side_from_dir(&missing, 0.5);
        assert!(result.is_err(), "missing plan dir must return Err");
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("[INPUT]"), "error must use [INPUT] prefix: {msg}");
    }

    #[test]
    fn test_load_plan_side_from_dir_missing_manifest_returns_error() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // Directory exists but no manifest.json
        let plan_dir = tmp.path().join("empty_plan");
        std::fs::create_dir_all(&plan_dir).unwrap();
        let result = load_plan_side_from_dir(&plan_dir, 0.5);
        assert!(result.is_err(), "dir without manifest must return Err");
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("manifest"), "error must mention manifest: {msg}");
    }

    #[test]
    fn test_load_plan_side_from_dir_minimal_manifest() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("vt_plan");
        std::fs::create_dir_all(&plan_dir).unwrap();
        let manifest_json = serde_json::json!({
            "label": "vt_congressional_2020",
            "num_districts": 1,
        });
        std::fs::write(plan_dir.join("manifest.json"), manifest_json.to_string()).unwrap();
        let result = load_plan_side_from_dir(&plan_dir, 0.5);
        assert!(result.is_ok(), "minimal manifest must load successfully: {:?}", result.err());
        let side = result.unwrap();
        assert_eq!(side.label, "vt_congressional_2020");
        assert_eq!(side.n_districts, 1);
    }

    #[test]
    fn test_load_plan_side_from_dir_uses_dir_name_when_no_label() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("tx_plan_2020");
        std::fs::create_dir_all(&plan_dir).unwrap();
        // manifest has no "label" field
        let manifest_json = serde_json::json!({"num_districts": 38});
        std::fs::write(plan_dir.join("manifest.json"), manifest_json.to_string()).unwrap();
        let side = load_plan_side_from_dir(&plan_dir, 0.5).unwrap();
        // Falls back to directory name
        assert_eq!(side.label, "tx_plan_2020",
            "label must fall back to directory name when absent from manifest");
    }

    #[test]
    fn test_load_plan_side_partisan_shape1_districts_array() {
        // Shape 1: { "districts": [{"dem_share": 0.55}, ...] }
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("vt_plan");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(plan_dir.join("manifest.json"),
            serde_json::json!({"label": "vt_test", "num_districts": 3}).to_string()).unwrap();
        let partisan = serde_json::json!({
            "districts": [
                {"dem_share": 0.55},
                {"dem_share": 0.45},
                {"dem_share": 0.60}
            ]
        });
        std::fs::write(analysis_dir.join("partisan.json"), partisan.to_string()).unwrap();
        let side = load_plan_side_from_dir(&plan_dir, 0.5).unwrap();
        assert_eq!(side.leaning_seats, 2, "2 districts have dem_share >= 0.5");
        assert_eq!(side.per_district_dem_share.len(), 3);
    }

    #[test]
    fn test_load_plan_side_partisan_shape2_flat_array() {
        // Shape 2: { "per_district_dem_share": [0.55, 0.42, 0.61] }
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("vt_plan2");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(plan_dir.join("manifest.json"),
            serde_json::json!({"label": "vt_test2", "num_districts": 3}).to_string()).unwrap();
        let partisan = serde_json::json!({"per_district_dem_share": [0.55, 0.42, 0.61]});
        std::fs::write(analysis_dir.join("partisan.json"), partisan.to_string()).unwrap();
        let side = load_plan_side_from_dir(&plan_dir, 0.5).unwrap();
        assert_eq!(side.leaning_seats, 2, "0.55 and 0.61 are >= 0.5");
        assert_eq!(side.per_district_dem_share, vec![0.55, 0.42, 0.61]);
    }

    #[test]
    fn test_load_plan_side_mm_count_from_vra_analysis() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("vt_plan3");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(plan_dir.join("manifest.json"),
            serde_json::json!({"label": "vt_test3", "num_districts": 5}).to_string()).unwrap();
        std::fs::write(analysis_dir.join("vra_analysis.json"),
            serde_json::json!({"mm_count": 3}).to_string()).unwrap();
        let side = load_plan_side_from_dir(&plan_dir, 0.5).unwrap();
        assert_eq!(side.mm_count, 3, "mm_count must be read from vra_analysis.json");
    }

    #[test]
    fn test_load_plan_side_mean_pp_from_compactness_flat() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("vt_plan4");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(plan_dir.join("manifest.json"),
            serde_json::json!({"label": "vt_test4", "num_districts": 2}).to_string()).unwrap();
        std::fs::write(analysis_dir.join("compactness.json"),
            serde_json::json!({"mean_polsby_popper": 0.42}).to_string()).unwrap();
        let side = load_plan_side_from_dir(&plan_dir, 0.5).unwrap();
        assert!((side.mean_pp - 0.42).abs() < 1e-9, "mean_pp must be read from compactness.json");
    }

    #[test]
    fn test_diff_from_assignments_no_files_returns_zeros() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let a_dir = tmp.path().join("plan_a");
        let b_dir = tmp.path().join("plan_b");
        std::fs::create_dir_all(&a_dir).unwrap();
        std::fs::create_dir_all(&b_dir).unwrap();
        // No final_assignments.json in either dir
        let diff = diff_from_assignments(&a_dir, &b_dir).unwrap();
        assert_eq!(diff.tracts_changed, 0);
        assert!(diff.districts_with_changes.is_empty());
    }

    #[test]
    fn test_diff_from_assignments_identical_returns_zero_changes() {
        use tempfile::TempDir;
        use std::collections::BTreeMap;
        let tmp = TempDir::new().unwrap();
        let a_dir = tmp.path().join("plan_a");
        let b_dir = tmp.path().join("plan_b");
        std::fs::create_dir_all(&a_dir).unwrap();
        std::fs::create_dir_all(&b_dir).unwrap();
        let assignments: BTreeMap<&str, usize> = [
            ("53001000100", 1usize), ("53001000200", 1), ("53001000300", 2),
        ].iter().cloned().collect();
        let json = serde_json::to_string(&assignments).unwrap();
        std::fs::write(a_dir.join("final_assignments.json"), &json).unwrap();
        std::fs::write(b_dir.join("final_assignments.json"), &json).unwrap();
        let diff = diff_from_assignments(&a_dir, &b_dir).unwrap();
        assert_eq!(diff.tracts_changed, 0, "identical assignments must produce 0 changed tracts");
        assert!(diff.districts_with_changes.is_empty());
    }
}
