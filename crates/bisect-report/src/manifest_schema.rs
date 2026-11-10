//! Portable native plan manifest schema.
use serde::{Deserialize, Serialize};
/// Full audit chain for a redistricting plan run.
pub type PlanManifest = PlanManifestWithCounts<usize>;
/// Read-only metadata preserves the native 64-bit count domain on wasm32.
pub type ReadOnlyPlanManifest = PlanManifestWithCounts<u64>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(bound(deserialize = "C: Deserialize<'de> + Default + From<u8>", serialize = "C: Serialize + Default + PartialEq"))]
pub struct PlanManifestWithCounts<C> {
    pub label: String,
    pub state_code: String,
    pub year: String,
    pub chamber: String,
    pub num_districts: C,
    pub population_source: String,
    pub partition_mode: String,
    pub seed: Option<i64>,
    /// Exact unsigned engine seed when it exceeds the legacy signed seed domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_seed: Option<u64>,
    /// Versioned resolved native replay recipe; recorded metadata, not execution authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_replay_recipe: Option<serde_json::Value>,
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
    pub seats_per_district: C,
    /// Total seats across all constituencies
    #[serde(default)]
    pub total_seats: C,
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
    pub split_seeds: Option<C>,
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

    // ── Spectral partitioning audit fields ───────────────────────────────────
    /// Spectral smoothing iterations per recursive bisection node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spectral_iters: Option<C>,

    // ── ILP / exact optimization audit fields ────────────────────────────────
    /// ILP method requested by the runner ("formulation-only", "branch-and-cut",
    /// or "iterative-separation"). Absent for non-ILP plans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ilp_method: Option<String>,
    /// ILP fallback policy requested by the runner ("metis" or "error").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ilp_fallback: Option<String>,
    /// Relative directory containing per-node `ilp-solve-report-v1` JSON files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ilp_solve_report_dir: Option<String>,
    /// Number of per-node ILP solve reports written for this run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ilp_solve_report_count: Option<C>,
    /// Relative path to the aggregate ILP audit summary JSON artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ilp_audit_summary_path: Option<String>,
    /// SHA-256 of the aggregate ILP audit summary JSON artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ilp_audit_summary_sha256: Option<String>,

    // ── Flip search audit fields ──────────────────────────────────────────────
    /// Search mode string when Flip was used (value: "flip").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip_search: Option<String>,
    /// Total flip proposals attempted (flip_steps parameter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip_steps: Option<C>,
    /// Percentile used to select from the visited plan distribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip_percentile: Option<f64>,
    /// Base seed used for the flip chain RNG.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip_base_seed: Option<u64>,
    /// Number of plans in the visited list (initial plan + accepted flips).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip_visited_count: Option<C>,
    /// Rank of the selected plan within the sorted visited list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip_selected_plan_rank: Option<C>,

    // ── Short-Burst search audit fields ──────────────────────────────────────
    /// Search mode string when ShortBurst was used (value: "short-burst").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_burst_search: Option<String>,
    /// ReCom steps per burst (burst_length parameter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst_length: Option<C>,
    /// Number of bursts run (n_bursts parameter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_bursts: Option<C>,
    /// Percentile used to select from burst endpoints (p parameter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_burst_percentile: Option<f64>,
    /// Base seed used for deriving per-burst chain seeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_burst_base_seed: Option<u64>,
    /// Per-burst chain seeds derived from the base seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst_seeds: Option<Vec<u64>>,
    /// Burst index of the selected endpoint (0-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_burst_idx: Option<C>,

    // ── Plan resolution fields ────────────────────────────────────────────────
    /// Geographic resolution of the plan assignments ("tract", "bg", "county")
    #[serde(
        default = "default_resolution",
        skip_serializing_if = "is_tract_resolution"
    )]
    pub plan_resolution: String,

    /// Number of geographic units in this plan (tracts, BGs, or counties)
    #[serde(default, skip_serializing_if = "is_zero")]
    pub n_units: C,

    /// Human-readable unit type label
    #[serde(default = "default_unit_type", skip_serializing_if = "is_census_tract")]
    pub unit_type: String,

    // ── Multi-scale fields (absent for non-multiscale runs) ───────────────────
    /// Fine resolution level for multiscale runs (e.g., "tract")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiscale_fine: Option<String>,

    /// Coarse resolution level for multiscale runs (e.g., "county")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiscale_coarse: Option<String>,

    /// GEOID prefix formula for fine->coarse mapping (e.g., "geoid_prefix[:5]")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_to_coarse_formula: Option<String>,

    /// Human-readable note on the formula (e.g., "first 5 chars of tract GEOID = county FIPS")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_to_coarse_comment: Option<String>,

    /// Filename of the geoids.json file for independent verification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_to_geoid_file: Option<String>,

    // ── RPLAN audit-certificate fields ──────────────────────────────────────
    /// Relative path to the emitted RPLAN v0.2 plan assignment artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rplan_path: Option<String>,
    /// Relative path to the emitted RPLAN context artifact used for audit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rctx_path: Option<String>,
    /// Relative path to the emitted audit certificate JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_certificate_path: Option<String>,
    /// SHA-256 of the full audit certificate document as written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_certificate_sha256: Option<String>,
    /// Stable canonical certificate content hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_certificate_content_hash: Option<String>,
    /// Audit result: pass, fail, or pass-with-warnings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_result: Option<String>,
    /// Legal profile id used to produce the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_profile_id: Option<String>,
    /// Context hash used to audit the final plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_hash: Option<String>,
}

fn default_seats_per_district<C: From<u8>>() -> C {
    1u8.into()
}
fn default_electoral_system() -> String {
    "single_member".to_string()
}
fn default_submission_type() -> String {
    "authoritative".to_string()
}
fn default_ufactor() -> u32 {
    5
}
fn default_niter() -> u32 {
    100
}
fn default_resolution() -> String {
    "tract".to_string()
}
fn default_unit_type() -> String {
    "census tract".to_string()
}
fn is_tract_resolution(s: &str) -> bool {
    s == "tract"
}
fn is_census_tract(s: &str) -> bool {
    s == "census tract"
}
fn is_zero<C: Default + PartialEq>(n: &C) -> bool {
    *n == C::default()
}


impl<C> PlanManifestWithCounts<C> {
    pub fn recorded_seed_json(&self)->serde_json::Value {
        self.engine_seed.map(serde_json::Value::from).unwrap_or_else(||serde_json::json!(self.seed))
    }
}
/// Native default plan label, independent of filesystem support.
pub fn default_label(state_name: &str, chamber: &str, year: &str) -> String {
    let normalized_state = state_name.to_lowercase().replace(' ', "_");
    format!("{}_{}_{}", normalized_state, chamber, year)
}

#[cfg(test)]
mod count_width_tests {
    use super::*;
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn readonly_counts_keep_native_schema_and_exact_integer_domain() {
        let mut value = serde_json::to_value(PlanManifest::default()).unwrap();
        for field in ["num_districts", "seats_per_district", "total_seats", "split_seeds", "spectral_iters", "ilp_solve_report_count", "flip_steps", "flip_visited_count", "flip_selected_plan_rank", "burst_length", "n_bursts", "selected_burst_idx", "n_units"] {
            value[field] = serde_json::json!(u64::MAX);
        }
        let text = serde_json::to_string(&value).unwrap();
        let native: PlanManifest = serde_json::from_str(&text).unwrap();
        let readonly: ReadOnlyPlanManifest = serde_json::from_str(&text).unwrap();
        assert_eq!(serde_json::to_string(&native).unwrap(), serde_json::to_string(&readonly).unwrap());
        assert_eq!(readonly.num_districts, u64::MAX);
    }
    #[test]
    fn readonly_defaults_match_native_and_duplicates_still_refuse() {
        let mut value = serde_json::to_value(PlanManifest::default()).unwrap();
        for field in ["seats_per_district", "total_seats", "n_units"] { value.as_object_mut().unwrap().remove(field); }
        let text = serde_json::to_string(&value).unwrap();
        let native: PlanManifest = serde_json::from_str(&text).unwrap();
        let readonly: ReadOnlyPlanManifest = serde_json::from_str(&text).unwrap();
        assert_eq!(native.seats_per_district, 1);
        assert_eq!(readonly.seats_per_district, 1);
        assert_eq!(serde_json::to_value(native).unwrap(), serde_json::to_value(readonly).unwrap());
        let duplicated = text.replace("\"num_districts\":0", "\"num_districts\":0,\"num_districts\":1");
        assert!(serde_json::from_str::<PlanManifest>(&duplicated).is_err());
        assert!(serde_json::from_str::<ReadOnlyPlanManifest>(&duplicated).is_err());
    }
    #[test]
    fn readonly_rejects_negative_fractional_and_overflow_counts() {
        let text = serde_json::to_string(&PlanManifest::default()).unwrap();
        for number in ["-1", "1.5", "18446744073709551616"] {
            let changed = text.replace("\"num_districts\":0", &format!("\"num_districts\":{number}"));
            assert!(serde_json::from_str::<PlanManifest>(&changed).is_err());
            assert!(serde_json::from_str::<ReadOnlyPlanManifest>(&changed).is_err());
        }
    }
}

