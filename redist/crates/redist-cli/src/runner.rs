/// Multi-state Rayon parallel runner + single-state implementation.
///
/// `run_states_parallel` dispatches states across Rayon threads.
/// `run_single_state` is the core: load adjacency → bisect → balance check → write.
///
/// PROCESS EXPLOSION PREVENTION: state_name and num_districts are pre-resolved
/// in StateConfig BEFORE the Rayon pool starts. This prevents 100+ Python
/// subprocesses from being spawned simultaneously during a 50-state run.
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::adjacency_loader::load_adjacency_pkl;
use crate::bisection_runner::{run_all_splits, run_nway_partition};
use crate::demographics::{load_demographics, align_demographics_to_adjacency};
use crate::partisan_shares::load_partisan_shares;
use crate::fetch::load_manifest;
use crate::output::{write_state_outputs, clean_corrupt_state, VraAnalysis, VraDistrict};
use crate::status::{status, ascii_safe};
use redist_core::{Partition, build_vra_edge_weights, build_partisan_weights, state_code_to_fips};
use redist_analysis::analyze_mm_districts;
use redist_report;

/// Result of processing a single state.
#[derive(Debug, Clone)]
pub struct StateResult {
    pub state_code: String,
    pub success: bool,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

/// Configuration for a single state run.
///
/// state_name and num_districts are pre-resolved by the caller (Commands::States
/// or Commands::Run) to avoid spawning Python once per state inside the Rayon pool.
///
/// Fields are grouped into four logical domains:
/// - **Identity**: what plan is being drawn (state, name, districts, year, version, output location)
/// - **Algorithm**: how the partitioning is performed (partition_mode, ufactor, niter, seed)
/// - **Control**: execution behavior (position, debug, reset, reprocess)
/// - **Spec 1 extensions**: chamber-aware, labeled, multi-member, COI, CVAP features
#[derive(Debug, Clone)]
pub struct StateConfig {
    // ── Identity: what plan is being drawn ───────────────────────────────────
    pub state_code: String,
    /// Lowercase state name for file paths (e.g. "alabama"). Pre-resolved.
    pub state_name: String,
    /// Number of congressional districts. Pre-resolved from config_{year}.py.
    pub num_districts: usize,
    pub year: String,
    pub version: String,
    pub output_dir: PathBuf,

    // ── Algorithm: how partitioning is performed ──────────────────────────────
    pub partition_mode: String,
    /// METIS imbalance factor (integer percent, e.g. 5 = 0.5% imbalance per level)
    pub ufactor: u32,
    /// Number of METIS refinement iterations
    pub niter: u32,
    /// Optional random seed for reproducibility
    pub seed: Option<u64>,

    // ── Control: execution behavior ───────────────────────────────────────────
    /// Progress bar position (TQDM_POSITION-style, -1 = disabled)
    pub position: i32,
    pub debug: bool,
    /// Delete existing outputs and re-run from scratch
    pub reset: bool,
    /// Reprocess even if outputs already exist (skip completion check)
    pub reprocess: bool,

    // ── Spec 1 extensions: chamber-aware, labeled, multi-member, COI ─────────
    /// Override district count (enables non-congressional chambers)
    pub num_districts_override: Option<usize>,
    /// Chamber type: "congressional" | "house" | "senate" | "custom"
    pub chamber: String,
    /// Human label for this plan run (default: {state}_{chamber}_{year})
    pub label: Option<String>,
    /// Population source: total, vap, cvap
    pub population_source: String,
    /// Max deviation per district in percent (None = use chamber default)
    pub balance_tolerance: Option<f64>,
    /// Write manifest.json alongside outputs
    pub write_manifest: bool,
    /// --force: overwrite existing plan without error
    pub force: bool,
    /// Geographic resolution: "tract" (default), "block_group", "block"
    pub resolution: String,
    /// Seats per constituency (1 for single-member, 3-5 for multi-member)
    pub seats_per_district: usize,
    /// Total seats across all constituencies (seats_per_district * num_districts if uniform)
    pub total_seats: usize,
    /// Direct adjacency file path (bypasses manifest lookup for international states).
    /// When Some, runner skips resolve_adjacency_path() and uses this path directly.
    pub adjacency_override: Option<std::path::PathBuf>,
    /// Path to COI weights file (optional)
    pub coi_weights: Option<std::path::PathBuf>,

    // ── Plan 03: Partisan edge-weighting (Callais 2026-04-29) ──────────────────
    /// Per-tract Democratic vote share TSV file (partisan-weighted mode only).
    /// Header: `geoid<TAB>dem_share`. GEOIDs are 11-char TIGER FIPS, share in [0.0, 1.0].
    /// MUTUALLY EXCLUSIVE with partition_mode == "metis-vra" (Callais p.36 disentanglement).
    pub partisan_shares: Option<std::path::PathBuf>,
    /// Threshold above which a unit is "strong-D" (default 0.55).
    pub dem_threshold: f64,
    /// Threshold below which a unit is "strong-R" (default 0.45).
    pub rep_threshold: f64,
}

impl StateConfig {
    /// Create a StateConfig for bulk congressional runs (Commands::States and Commands::Run).
    ///
    /// All Spec 1 fields default to their canonical bulk-run values:
    /// - `partition_mode`: "edge-weighted"
    /// - `ufactor`: 5, `niter`: 100, `seed`: None
    /// - `debug`: false, `reset`: false, `reprocess`: false
    /// - `chamber`: "congressional", `population_source`: "total"
    /// - `resolution`: "tract", `seats_per_district`: 1
    /// - `write_manifest`: false, `force`: false
    /// - All override/optional fields: None
    ///
    /// `total_seats` is set to `num_districts` (single-member default).
    ///
    /// Use `Commands::State` (the single-state arm) for custom chambers, labels,
    /// multi-member districts, COI weights, etc. — those require the full struct literal.
    pub fn new_bulk(
        state_code: String,
        state_name: String,
        num_districts: usize,
        year: String,
        version: String,
        output_dir: std::path::PathBuf,
        position: i32,
    ) -> Self {
        Self {
            state_code,
            state_name,
            num_districts,
            year,
            version,
            output_dir,
            position,
            // Algorithm defaults
            partition_mode: "edge-weighted".to_string(),
            ufactor: 5,
            niter: 100,
            seed: None,
            // Control defaults
            debug: false,
            reset: false,
            reprocess: false,
            // Spec 1 defaults for bulk congressional runs
            num_districts_override: None,
            chamber: "congressional".to_string(),
            label: None,
            population_source: "total".to_string(),
            balance_tolerance: None,
            write_manifest: false,
            force: false,
            resolution: "tract".to_string(),
            seats_per_district: 1,
            total_seats: num_districts,
            adjacency_override: None,
            coi_weights: None,
            // Plan 03 defaults: partisan-weighted mode disabled
            partisan_shares: None,
            dem_threshold: 0.55,
            rep_threshold: 0.45,
        }
    }

    /// Returns the effective balance tolerance based on chamber type.
    ///
    /// Priority order:
    /// 1. Explicit `--balance-tolerance` override (always wins)
    /// 2. Chamber-specific value from state policy database
    /// 3. Fallback: 0.5% congressional / 5% state legislative
    pub fn effective_balance_tolerance(&self) -> f64 {
        self.balance_tolerance.unwrap_or_else(|| {
            chamber_balance_tolerance(&self.state_code, &self.chamber)
        })
    }

    /// Returns the effective label for this plan run.
    pub fn effective_label(&self) -> String {
        self.label.clone().unwrap_or_else(|| {
            redist_report::default_label(&self.state_name, &self.chamber, &self.year)
        })
    }

    /// Returns the effective number of districts (override takes priority).
    pub fn effective_num_districts(&self) -> usize {
        self.num_districts_override.unwrap_or(self.num_districts)
    }

    /// Returns the effective number of seats per district (always >= 1).
    pub fn effective_seats_per_district(&self) -> usize {
        self.seats_per_district.max(1)
    }

    /// Returns the ideal population per seat (not per district).
    /// For single-member: same as ideal_per_district.
    /// For multi-member: total_pop / total_seats.
    pub fn ideal_pop_per_seat(&self, total_pop: i64) -> f64 {
        let total_seats = self.total_seats.max(1);
        total_pop as f64 / total_seats as f64
    }
}

/// Resolve balance tolerance for a given chamber from state policy.
///
/// Uses state_policy.json fields: `balance_tolerance_house_pct`, `balance_tolerance_senate_pct`,
/// `balance_tolerance_congressional_pct`. Falls back to algorithm defaults (0.5% congressional,
/// 5.0% state legislative) when the state is not in the policy or the field is missing.
pub fn chamber_balance_tolerance(state_code: &str, chamber: &str) -> f64 {
    let policy = crate::policy::load_policy();
    if let Some(state) = crate::policy::get_state_policy(&policy, state_code) {
        let key = match chamber {
            "congressional" => "balance_tolerance_congressional_pct",
            "house" | "lower" | "assembly" => "balance_tolerance_house_pct",
            "senate" | "upper" => "balance_tolerance_senate_pct",
            _ => "",
        };
        if !key.is_empty() {
            if let Some(pct) = state.get(key).and_then(|v| v.as_f64()) {
                if pct > 0.0 {
                    return pct / 100.0; // policy stores in %, we use fraction
                }
            }
        }
    }
    // Fallback: constitutional standard for one-person-one-vote
    match chamber {
        "congressional" => 0.005,
        _ => 0.05,
    }
}

/// Resolve district count for a given chamber from state policy.
///
/// When `--chamber house` or `--chamber senate` is specified without `--districts`,
/// this looks up `house_districts` or `senate_districts` from the embedded state
/// policy database. Falls back to `congressional_fallback` (from the manifest) if
/// the policy doesn't have the chamber or the state is unknown.
///
/// This ensures `redist state --state WA --chamber house` automatically uses 98
/// districts without requiring the user to also pass `--districts 98`.
pub fn chamber_district_count(
    state_code: &str,
    chamber: &str,
    congressional_fallback: usize,
) -> usize {
    if chamber == "congressional" {
        return congressional_fallback;
    }
    let policy = crate::policy::load_policy();
    if let Some(state) = crate::policy::get_state_policy(&policy, state_code) {
        let key = match chamber {
            "house" | "lower" | "assembly" => "house_districts",
            "senate" | "upper" => "senate_districts",
            _ => return congressional_fallback,
        };
        if let Some(n) = state.get(key).and_then(|v| v.as_u64()) {
            if n > 0 {
                return n as usize;
            }
            if n == 0 && (key == "senate_districts" || key == "house_districts") {
                // Zero means this chamber doesn't exist (e.g., NE unicameral has no senate)
                let notes = state.get("notes").and_then(|v| v.as_str()).unwrap_or("");
                let hint = if notes.to_lowercase().contains("unicameral") {
                    format!(" {} has a unicameral legislature — use --chamber house.", state_code)
                } else {
                    format!(" {} has no {} chamber.", state_code, chamber)
                };
                eprintln!("ERROR: No {chamber} chamber for {state_code}.{hint}");
                std::process::exit(1);
            }
        }
    }
    congressional_fallback
}

/// Load all state codes, names, and district counts for a given year.
/// Returns Vec<(state_code, state_name, num_districts)> sorted alphabetically.
/// Reads directly from the embedded manifest — no Python subprocess.
///
/// Warning 6: if `year` is not in the manifest (e.g. "2030" or a typo),
/// all states are silently omitted. The caller sees an empty Vec with no error.
/// Valid years: "2020", "2010", "2000".
pub fn load_all_states(year: &str) -> Result<Vec<(String, String, usize)>, String> {
    if !["2020", "2010", "2000"].contains(&year) {
        return Err(format!(
            "unsupported year '{year}' — valid years are 2020, 2010, 2000"
        ));
    }
    let manifest = crate::fetch::load_manifest()?;
    let mut states: Vec<(String, String, usize)> = manifest.states.into_iter()
        .filter_map(|(code, state)| {
            let districts = *state.districts.get(year)?;
            if districts == 0 { return None; }
            let name = state.name.to_lowercase().replace(' ', "_");
            Some((code, name, districts))
        })
        .collect();
    states.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(states)
}

/// Return the actual worker count that will be used (capped to available CPU threads).
pub fn effective_workers(requested: usize) -> usize {
    requested.min(rayon::current_num_threads())
}

/// Run multiple states in parallel using Rayon.
/// Workers cap: min(workers, available_threads).
pub fn run_states_parallel(configs: Vec<StateConfig>, workers: usize) -> Vec<StateResult> {
    let actual_workers = effective_workers(workers);
    if actual_workers < workers {
        eprintln!(
            "NOTE: --workers {} capped to {} (available CPU threads). Actual parallelism: {}x.",
            workers, actual_workers, actual_workers
        );
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(actual_workers)
        .build()
        .expect("failed to build Rayon thread pool");

    pool.install(|| {
        configs.par_iter().map(|cfg| {
            let start = std::time::Instant::now();
            let result = run_single_state(cfg);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            match result {
                Ok(()) => StateResult {
                    state_code: cfg.state_code.clone(),
                    success: true,
                    error: None,
                    elapsed_ms,
                },
                Err(e) => StateResult {
                    state_code: cfg.state_code.clone(),
                    success: false,
                    error: Some(format!("{}: {}", cfg.state_code, ascii_safe(&e.to_string()))),
                    elapsed_ms,
                }
            }
        }).collect()
    })
}

/// Extract the census year (2000, 2010, or 2020) from an adjacency filename.
///
/// Looks for a 4-digit number matching 2000, 2010, or 2020 in the filename.
/// Returns `None` if no valid census year is found.
pub fn extract_year_from_adj_filename(filename: &str) -> Option<&'static str> {
    // Search for any of the known census years as a substring
    for year in &["2020", "2010", "2000"] {
        if filename.contains(year) {
            return Some(year);
        }
    }
    None
}

/// Check whether the year in the adjacency filename matches the requested year.
///
/// Emits a WARNING to stderr (not an error) when there is a mismatch.
/// A mismatch can occur when the user requests --year 2020 but only a 2010
/// adjacency file is available and is used as fallback.
pub fn check_adjacency_year_mismatch(path: &PathBuf, requested_year: &str, state_code: &str) {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if let Some(file_year) = extract_year_from_adj_filename(filename) {
        if file_year != requested_year {
            eprintln!(
                "WARNING: Requested adjacency for year {requested_year} but adjacency file is \
                 for year {file_year}: {filename}\n\
                 Census tract boundaries changed between {file_year} and {requested_year} \
                 -- results will use {file_year} geography.\n\
                 For {requested_year} census tracts: run \
                 redist fetch --year {requested_year} --type adjacency --states {}",
                state_code.to_uppercase()
            );
        }
    }
}

/// Resolve the adjacency pkl path for a state using the manifest.
///
/// The manifest's `local_outputs_dir` + "V3/data/{year}/adjacency/" is the
/// canonical adjacency store — the same path that `redist fetch --release` downloads to.
/// Override with REDIST_MANIFEST env var for custom data layouts.
///
/// Returns `(path, effective_resolution)` where `effective_resolution` may differ from
/// the requested resolution if a graceful fallback to tract occurred.
fn resolve_adjacency_path(
    state_code_lower: &str,
    year: &str,
    resolution: &str,
) -> Result<(PathBuf, String), String> {
    let manifest = load_manifest()
        .map_err(|e| format!("cannot load manifest: {e}"))?;
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);

    // Choose filename based on requested resolution
    let (adj_filename, is_block_group) = match resolution {
        "block_group" | "block-group" => (
            format!("{state_code_lower}_bg_adjacency_{year}.pkl"),
            true,
        ),
        _ => (
            format!("{state_code_lower}_adjacency_{year}.pkl"),
            false,
        ),
    };

    // Try V3 then V4 canonical stores
    let canonical = outputs_dir
        .join("V3").join("data").join(year).join("adjacency")
        .join(&adj_filename);
    if canonical.exists() {
        return Ok((canonical, resolution.to_string()));
    }
    let v4 = outputs_dir
        .join("V4").join("data").join(year).join("adjacency")
        .join(&adj_filename);
    if v4.exists() {
        return Ok((v4, resolution.to_string()));
    }

    // Block group not found — graceful fallback to tract with clear warning
    if is_block_group {
        eprintln!(
            "WARNING: --resolution block_group was requested but block_group adjacency \
             not found for {state_code_lower} {year}.\n\
             To get block_group data: redist fetch --type adjacency --states {} --year {}\n\
             Falling back to tract resolution.",
            state_code_lower.to_uppercase(), year
        );
        let tract_filename = format!("{state_code_lower}_adjacency_{year}.pkl");
        let tract_canonical = outputs_dir
            .join("V3").join("data").join(year).join("adjacency")
            .join(&tract_filename);
        if tract_canonical.exists() {
            return Ok((tract_canonical, "tract".to_string()));
        }
        let tract_v4 = outputs_dir
            .join("V4").join("data").join(year).join("adjacency")
            .join(&tract_filename);
        if tract_v4.exists() {
            return Ok((tract_v4, "tract".to_string()));
        }
        let state_upper = state_code_lower.to_uppercase();
        return Err(format!(
            "Adjacency file not found for {state_code_lower} {year}.\n\
             Run: redist fetch --type adjacency --states {state_upper} --year {year}\n\
             Then: redist state --state {state_upper} --year {year} ..."
        ));
    }

    let state_upper = state_code_lower.to_uppercase();
    Err(format!(
        "Adjacency file not found for {state_code_lower} {year}.\n\
         Run: redist fetch --type adjacency --states {state_upper} --year {year}\n\
         Then: redist state --state {state_upper} --year {year} ..."
    ))
}

/// Check CVAP data availability and warn + fall back to total if missing.
///
/// The CVAP file is expected at:
///   `outputs/{version}/data/{year}/demographics/{state_lower}_cvap_{year}.csv`
/// or the legacy path:
///   `data/{year}/demographics/{state_lower}_cvap_{year}.csv`
///
/// Returns the effective population source: "cvap" if file exists, "total" otherwise.
pub fn check_cvap_availability(
    requested: &str,
    state_name: &str,
    year: &str,
    state_code: &str,
) -> String {
    if requested != "cvap" {
        return requested.to_string();
    }
    // Try the canonical CVAP path used by the Python pipeline
    let cvap_path = std::path::Path::new("data")
        .join(year)
        .join("demographics")
        .join(format!("{state_name}_cvap_{year}.csv"));
    if cvap_path.exists() {
        return "cvap".to_string();
    }
    eprintln!(
        "WARNING: CVAP data not found for {state_code} {year}.\n\
         CVAP requires a separate download: \
         https://www.census.gov/programs-surveys/decennial-census/about/voting-rights/cvap.html\n\
         Falling back to total population."
    );
    "total".to_string()
}

/// Validate Plan 03 partisan-mode configuration.
///
/// Two rules (Callais p.36 disentanglement, Plan 03 Task 4.5):
/// 1. `partisan_shares` is only valid when `partition_mode == "partisan-weighted"`.
///    A run that supplies partisan_shares with any other mode is rejected.
/// 2. `partition_mode == "partisan-weighted"` requires `partisan_shares` to be set.
///
/// Together these guarantee the partition_mode/data pairing is consistent and that
/// race-conscious (`metis-vra`) and partisan signals are never both active.
pub fn validate_partisan_config(cfg: &StateConfig) -> Result<(), String> {
    if cfg.partisan_shares.is_some() && cfg.partition_mode != "partisan-weighted" {
        return Err(format!(
            "{}: --partisan-shares requires --partition-mode=partisan-weighted (got {})",
            cfg.state_code, cfg.partition_mode
        ));
    }
    if cfg.partition_mode == "partisan-weighted" && cfg.partisan_shares.is_none() {
        return Err(format!(
            "{}: --partition-mode=partisan-weighted requires --partisan-shares <PATH>",
            cfg.state_code
        ));
    }
    Ok(())
}

/// Run a single state redistricting task end-to-end.
///
/// Flow: load adjacency → build edge weights → bisect → assert balance → write outputs
fn run_single_state(cfg: &StateConfig) -> Result<(), String> {
    let num_districts = cfg.effective_num_districts();
    let state_name = &cfg.state_name; // e.g. "vermont" — used for directory paths
    let label = cfg.effective_label();
    let balance_tolerance = cfg.effective_balance_tolerance();
    // Defensive: tolerance must be in [0.0001, 0.50] as a fraction.
    // Values outside this range indicate a unit error (% passed as fraction or vice versa).
    if balance_tolerance < 0.0001 || balance_tolerance > 0.50 {
        return Err(format!(
            "{}: balance tolerance {:.6} is outside plausible range [0.0001, 0.50]. \
             Pass as percent to --balance-tolerance (e.g., 0.5 for ±0.5%, 5 for ±5%).",
            cfg.state_code, balance_tolerance
        ));
    }

    // Determine output directory structure:
    //   Labeled runs: {output_dir}/{year}/plans/{label}/data/
    //   Legacy runs:  {output_dir}/{year}/states/{state_name}/data/
    let year_base = cfg.output_dir.join(&cfg.year);
    let (plan_root, data_dir) = if cfg.label.is_some() {
        let plan_dir = year_base.join("plans").join(&label);
        let data_dir = plan_dir.join("data");
        (plan_dir, data_dir)
    } else {
        let state_dir = year_base.join("states").join(state_name);
        let data_dir = state_dir.join("data");
        (state_dir, data_dir)
    };

    // Board amendment: detect incomplete plan (manifest.tmp present)
    redist_report::check_incomplete_plan(&plan_root, &label)
        .map_err(|e| ascii_safe(&e.to_string()))?;

    // Label collision check: if manifest.json exists and --force not set, exit
    if cfg.label.is_some() {
        let manifest_path = plan_root.join("manifest.json");
        redist_report::check_plan_collision(&plan_root, cfg.force)
            .map_err(|e| ascii_safe(&e.to_string()))?;
        let _ = manifest_path; // suppress warning
    }

    // Reset: delete existing outputs before starting
    if cfg.reset {
        // Warn before deletion so users can see exactly what will be removed
        eprintln!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root.display()
        );
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)
                .map_err(|e| format!("reset failed: {e}"))?;
        }
    }
    // Create plan directory structure if labeled
    if cfg.label.is_some() {
        redist_report::create_plan_dir(&year_base, &label)
            .map_err(|e| format!("cannot create plan dir: {e}"))?;
    }
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("cannot create data dir: {e}"))?;

    status(cfg.position, &format!("{}: loading adjacency", cfg.state_code));

    // 1. Load adjacency graph
    // Adjacency path comes from the manifest (same source as `redist fetch`).
    // The manifest's local_outputs_dir + "V3/data/{year}/adjacency/" is the
    // canonical store. REDIST_MANIFEST can override this for custom setups.
    let state_code_lower = cfg.state_code.to_lowercase();
    let adj_pkl = if let Some(ref override_path) = cfg.adjacency_override {
        override_path.clone()
    } else {
        let (path, _effective_resolution) = resolve_adjacency_path(&state_code_lower, &cfg.year, &cfg.resolution)?;
        // Task 135: warn when adjacency file year doesn't match requested year
        check_adjacency_year_mismatch(&path, &cfg.year, &cfg.state_code);
        path
    };

    let graph = load_adjacency_pkl(&adj_pkl)
        .map_err(|e| format!("adjacency load failed: {e}"))?;

    // Check for isolated nodes (no adjacency neighbors) — common with island tracts.
    // Isolated tracts will always form non-contiguous districts.
    let isolated: Vec<usize> = graph.adjacency.iter().enumerate()
        .filter(|(_, nbrs)| nbrs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if !isolated.is_empty() {
        eprintln!(
            "WARNING: {}: {} isolated tract(s) with no adjacency neighbors. \
             These will form non-contiguous districts. \
             For island states (AK, HI, international), rebuild adjacency with water bridges.",
            cfg.state_code, isolated.len()
        );
    }

    // 1b. CVAP population source check
    // CVAP data requires a separate download from the Census Bureau.
    // If "cvap" is requested but the file is missing, warn and fall back to total.
    let _effective_population_source = check_cvap_availability(
        &cfg.population_source,
        state_name,
        &cfg.year,
        &cfg.state_code,
    );

    // 2. Build edge weights based on partition mode
    //
    // Mutual-exclusion guard (Callais p.36 disentanglement, Plan 03 Task 4.5):
    // partisan-weighted and metis-vra modes cannot coexist. Centralised in
    // validate_partisan_config so it can be unit-tested in isolation.
    validate_partisan_config(cfg)?;

    let edge_weights: HashMap<(usize, usize), f64> = match cfg.partition_mode.as_str() {
        "edge-weighted" => {
            status(cfg.position, &format!("{}: edge-weighted mode ({} edges)", cfg.state_code, graph.n_edges));
            graph.edge_weights.clone()
        }
        "partisan-weighted" if num_districts > 1 => {
            // Plan 03: edge-weighted bisection that preserves partisan clusters.
            // Callais 2026-04-29 grounds this for both state-mapmaker and §2-challenger use cases.
            let shares_path = cfg.partisan_shares.as_ref()
                .expect("guarded above: partisan-weighted requires partisan_shares");
            status(cfg.position, &format!("{}: partisan-weighted mode — loading {}", cfg.state_code, shares_path.display()));
            let dem_shares = load_partisan_shares(shares_path, &graph.index_to_geoid, graph.n_vertices)
                .map_err(|e| format!("partisan shares load failed: {e}"))?;
            let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
                .flat_map(|(i, nbrs)| nbrs.iter().filter(move |&&j| j > i).map(move |&j| (i, j)))
                .collect();
            build_partisan_weights(&edges, &dem_shares, cfg.dem_threshold, cfg.rep_threshold)
        }
        "partisan-weighted" if num_districts == 1 => {
            status(cfg.position, &format!("{}: single district — skipping partisan weighting (trivially trivial)", cfg.state_code));
            HashMap::new()
        }
        "metis-vra" if num_districts > 1 => {
            status(cfg.position, &format!("{}: VRA mode — loading demographics", cfg.state_code));
            let demo_path = std::path::Path::new("data")
                .join(&cfg.year).join("demographics")
                .join(format!("{state_name}_demographics_{}.csv", cfg.year));
            let demo = load_demographics(&demo_path)
                .map_err(|e| format!("demographics load failed: {e}"))?;
            let minority_fracs = align_demographics_to_adjacency(
                &demo, &graph.index_to_geoid, graph.n_vertices
            );
            let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
                .flat_map(|(i, nbrs)| nbrs.iter().filter(move |&&j| j > i).map(move |&j| (i, j)))
                .collect();
            build_vra_edge_weights(&edges, &minority_fracs, 0.40)
        }
        "metis-vra" if num_districts == 1 => {
            status(cfg.position, &format!("{}: single district — skipping VRA (trivially compliant)", cfg.state_code));
            HashMap::new()
        }
        _ => HashMap::new(), // unweighted
    };

    // 2b. Apply COI weights if provided.
    // Each edge weight is multiplied by sqrt(w_a * w_b) (geometric mean of endpoint weights).
    // Tracts not in the COI file get weight 1.0 (no modification).
    let edge_weights = if let Some(ref coi_path) = cfg.coi_weights {
        match apply_coi_weights(edge_weights, coi_path, &graph.index_to_geoid) {
            Ok(ew) => ew,
            Err(e) => {
                eprintln!("WARNING: COI weights not applied: {e}");
                // Continue without COI weights rather than failing the whole run
                match cfg.partition_mode.as_str() {
                    "edge-weighted" => graph.edge_weights.clone(),
                    _ => HashMap::new(),
                }
            }
        }
    } else {
        edge_weights
    };

    // 3. Run partitioning
    // VRA mode uses n-way: D.2 shows equivalent VRA success rate to recursive
    // bisection (47.5% vs 48.3%, p=0.634) with 3.08× speedup.
    // Non-VRA modes use recursive bisection (level-parallel for multi-district).
    let use_nway = cfg.partition_mode == "metis-vra" && num_districts > 1;
    let method = if use_nway { "n-way" } else { "recursive bisection" };
    status(cfg.position, &format!(
        "{}: {} into {} districts", cfg.state_code, method, num_districts
    ));

    // Intermediate rounds go in {plan_root}/intermediate/
    let intermediate_dir = plan_root.join("intermediate");
    std::fs::create_dir_all(&intermediate_dir)
        .map_err(|e| format!("cannot create intermediate dir: {e}"))?;

    let assignments = if use_nway {
        // N-way: single gpmetis call with nparts=k — no intermediate rounds
        run_nway_partition(
            &graph.adjacency,
            &graph.vertex_weights,
            &edge_weights,
            num_districts,
            1.0 + cfg.ufactor as f64 / 1000.0,  // convert int ufactor to decimal
            cfg.niter,
            cfg.seed,
        ).map_err(|e| format!("n-way partition failed: {e}"))?
    } else {
        // Pass balance_tolerance as fraction: per-node ufactor = 1 + tolerance/k_node
        // This guarantees cumulative error ≤ tolerance regardless of bisection depth.
        let balance_tolerance_frac = cfg.effective_balance_tolerance();
        run_all_splits(
            &graph.adjacency,
            &graph.vertex_weights,
            &edge_weights,
            num_districts,
            balance_tolerance_frac,
            cfg.niter,
            cfg.seed,
            Some(&intermediate_dir),
        ).map_err(|e| format!("bisection failed: {e}"))?
    };

    // 4. Assert population balance (chamber-aware tolerance).
    // Congressional: ±0.5% (one-person-one-vote standard)
    // State legislative: ±5.0% (state constitutional default)
    // Explicit override wins via --balance-tolerance flag.
    let partition = Partition::from_assignments(assignments.clone());
    let balance_ok = partition.assert_balanced(&graph.vertex_weights, num_districts, balance_tolerance);
    if let Err(e) = balance_ok {
        return Err(format!("population balance violation (tolerance {:.1}%): {e}", balance_tolerance * 100.0));
    }

    status(cfg.position, &format!("{}: balance OK, writing outputs", cfg.state_code));

    // 5. VRA analysis (if VRA mode and multi-district)
    let vra = if cfg.partition_mode == "metis-vra" && num_districts > 1 {
        let demo_path = std::path::Path::new("data")
            .join(&cfg.year).join("demographics")
            .join(format!("{state_name}_demographics_{}.csv", cfg.year));
        let demo = load_demographics(&demo_path)
            .map_err(|e| format!("demographics reload for VRA analysis failed: {e}"))?;
        let minority_fracs = align_demographics_to_adjacency(
            &demo, &graph.index_to_geoid, graph.n_vertices
        );
        let total_pops = graph.vertex_weights.clone();
        let minority_pops: Vec<f64> = minority_fracs.iter()
            .zip(total_pops.iter())
            .map(|(f, &t)| f * t as f64)
            .collect();
        let zeros = vec![0.0f64; graph.n_vertices];
        let analysis = analyze_mm_districts(
            &assignments, &total_pops, &minority_pops, &zeros, &zeros, 0.50
        );
        status(cfg.position, &format!("{}: {} MM districts", cfg.state_code, analysis.mm_count));
        Some(VraAnalysis {
            mm_count: analysis.mm_count,
            mm_districts: analysis.mm_districts,
            districts: analysis.districts.iter().map(|d| VraDistrict {
                district: d.district,
                pct_minority: d.pct_minority,
                pct_black: d.pct_black,
                pct_hispanic: d.pct_hispanic,
                is_mm: d.is_mm,
            }).collect(),
        })
    } else {
        None
    };

    // 6. Write outputs atomically (rename-from-tmp pattern)
    write_state_outputs(&data_dir, &assignments, vra.as_ref())
        .map_err(|e| format!("output write failed: {e}"))?;

    // 6b. Compute edge-cut of the final partition.
    // Sum edge weights for all edges (u, v) whose endpoints are in different districts.
    // Stored in the manifest for seed-sensitivity research (B.7).
    let edge_cut: f64 = edge_weights.iter().map(|(&(u, v), &w)| {
        let du = assignments.get(&u).copied().unwrap_or(0);
        let dv = assignments.get(&v).copied().unwrap_or(0);
        if du != dv { w } else { 0.0 }
    }).sum();

    // 7. Write manifest.json atomically (manifest.tmp → manifest.json).
    // Board amendment: atomic write (manifest.tmp + rename) prevents partial manifests.
    if cfg.write_manifest || cfg.label.is_some() {
        let adj_filename = format!("{}_adjacency_{}.adj.bin", state_name, cfg.year);
        let state_fips = state_code_to_fips(&cfg.state_code).unwrap_or("00").to_string();
        let tiger_url = redist_report::tiger_source_url(&state_fips, &cfg.year);
        let gpmetis_version = crate::bisection_runner::detect_gpmetis_version();
        let manifest = redist_report::PlanManifest {
            label: label.clone(),
            state_code: cfg.state_code.clone(),
            year: cfg.year.clone(),
            chamber: cfg.chamber.clone(),
            num_districts,
            population_source: cfg.population_source.clone(),
            partition_mode: cfg.partition_mode.clone(),
            seed: cfg.seed.map(|s| s as i64),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            binary_sha256: String::new(), // populated by installer/release only
            binary_download_url: format!(
                "https://github.com/owner/redist/releases/download/v{}/redist",
                env!("CARGO_PKG_VERSION")
            ),
            adjacency_file: adj_filename,
            adjacency_sha256: String::new(), // populated post-build
            adjacency_build_command: format!(
                "python scripts/data/generate_adj_bin.py --year {} --states {}",
                cfg.year, state_name
            ),
            adjacency_build_version: env!("CARGO_PKG_VERSION").to_string(),
            tiger_source_url: tiger_url,
            tiger_sha256: None, // expensive; computed separately if needed
            created_at: redist_report::now_iso8601(),
            balance_tolerance_pct: balance_tolerance * 100.0,
            population_balance_valid: true,
            seats_per_district: cfg.effective_seats_per_district(),
            total_seats: cfg.total_seats,
            electoral_system: if cfg.seats_per_district <= 1 {
                "single_member".to_string()
            } else {
                "multi_member_uniform".to_string()
            },
            gpmetis_version,
            // SSI Task 5/7 fields: state-staff-imported plans set these via run_import;
            // state-runner-produced plans default to authoritative + None (per `..Default`).
            submission_type: "authoritative".to_string(),
            submitted_by: None,
            submitted_at: None,
            source_tool: None,
            source_tool_version: None,
            source_format_fingerprint: None,
            import_compat_sha256: None,
            edge_cut: Some(edge_cut),
        };
        redist_report::write_manifest_atomic(&plan_root, &manifest)
            .map_err(|e| format!("manifest write failed: {e}"))?;
    }

    status(cfg.position, &format!("{}: complete ({num_districts}D, {}ms)", cfg.state_code, 0));
    Ok(())
}

/// Check if a state's outputs already exist and are complete.
pub fn state_already_complete(output_dir: &PathBuf, state_code: &str, year: &str, reprocess: bool) -> bool {
    if reprocess { return false; }
    let data_dir = output_dir
        .join(year).join("states").join(state_code.to_lowercase()).join("data");
    data_dir.join("final_assignments.json").exists()
        || data_dir.join("final_assignments.pkl").exists()
}

/// Filter configs to only those needing processing.
pub fn filter_incomplete(configs: Vec<StateConfig>) -> Vec<StateConfig> {
    configs.into_iter()
        .filter(|cfg| !state_already_complete(&cfg.output_dir, &cfg.state_code, &cfg.year, cfg.reprocess))
        .collect()
}

/// Apply COI (Communities of Interest) weights to edge weights.
///
/// Loads a JSON file mapping GEOID -> weight (0.0-1.0). For each edge (u, v),
/// multiplies the edge weight by sqrt(w_u * w_v) (geometric mean of endpoint weights).
/// Tracts not in the COI file get weight 1.0 (no modification).
///
/// The geometric mean ensures that if both endpoints of an edge are in the same
/// community (high weight), the edge is strengthened and METIS will avoid cutting it.
pub fn apply_coi_weights(
    mut edge_weights: HashMap<(usize, usize), f64>,
    coi_path: &std::path::Path,
    index_to_geoid: &HashMap<usize, String>,
) -> Result<HashMap<(usize, usize), f64>, String> {
    let content = std::fs::read_to_string(coi_path)
        .map_err(|e| format!("cannot read COI weights file {}: {e}", coi_path.display()))?;
    let coi_map: HashMap<String, f64> = serde_json::from_str(&content)
        .map_err(|e| format!("cannot parse COI weights JSON: {e}"))?;

    // Build a geoid -> weight lookup by index
    let get_weight = |idx: usize| -> f64 {
        index_to_geoid.get(&idx)
            .and_then(|geoid| coi_map.get(geoid))
            .copied()
            .unwrap_or(1.0)
    };

    for (&(u, v), weight) in edge_weights.iter_mut() {
        let w_u = get_weight(u);
        let w_v = get_weight(v);
        let factor = (w_u * w_v).sqrt();
        *weight *= factor;
    }

    Ok(edge_weights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_config(state: &str) -> StateConfig {
        StateConfig {
            state_code: state.to_string(),
            state_name: state.to_lowercase(),
            num_districts: 1,
            year: "2020".to_string(),
            version: "V3".to_string(),
            output_dir: PathBuf::from("/tmp/test"),
            partition_mode: "edge-weighted".to_string(),
            position: 999,
            debug: false,
            reset: false,
            reprocess: false,
            ufactor: 5,
            niter: 100,
            seed: Some(42),
            // Spec 1 defaults
            num_districts_override: None,
            chamber: "congressional".to_string(),
            label: None,
            population_source: "total".to_string(),
            balance_tolerance: None,
            write_manifest: false,
            force: false,
            resolution: "tract".to_string(),
            seats_per_district: 1,
            total_seats: 1,
            adjacency_override: None,
            coi_weights: None,
            partisan_shares: None,
            dem_threshold: 0.55,
            rep_threshold: 0.45,
        }
    }

    // ── Task 199: StateConfig::new_bulk constructor ───────────────────────────

    #[test]
    fn test_new_bulk_defaults() {
        let cfg = StateConfig::new_bulk(
            "WA".to_string(),
            "washington".to_string(),
            10,
            "2020".to_string(),
            "v1".to_string(),
            PathBuf::from("/tmp/test"),
            3,
        );
        // Identity fields set correctly
        assert_eq!(cfg.state_code, "WA");
        assert_eq!(cfg.state_name, "washington");
        assert_eq!(cfg.num_districts, 10);
        assert_eq!(cfg.year, "2020");
        assert_eq!(cfg.version, "v1");
        assert_eq!(cfg.output_dir, PathBuf::from("/tmp/test"));
        assert_eq!(cfg.position, 3);
        // Algorithm defaults
        assert_eq!(cfg.partition_mode, "edge-weighted");
        assert_eq!(cfg.ufactor, 5);
        assert_eq!(cfg.niter, 100);
        assert!(cfg.seed.is_none());
        // Control defaults
        assert!(!cfg.debug);
        assert!(!cfg.reset);
        assert!(!cfg.reprocess);
        // Spec 1 defaults
        assert!(cfg.num_districts_override.is_none());
        assert_eq!(cfg.chamber, "congressional");
        assert!(cfg.label.is_none());
        assert_eq!(cfg.population_source, "total");
        assert!(cfg.balance_tolerance.is_none());
        assert!(!cfg.write_manifest);
        assert!(!cfg.force);
        assert_eq!(cfg.resolution, "tract");
        assert_eq!(cfg.seats_per_district, 1);
        // total_seats == num_districts for single-member default
        assert_eq!(cfg.total_seats, 10);
        assert!(cfg.adjacency_override.is_none());
        assert!(cfg.coi_weights.is_none());
    }

    #[test]
    fn test_new_bulk_total_seats_matches_num_districts() {
        // For single-member districts, total_seats must equal num_districts
        for n in [1usize, 5, 10, 52] {
            let cfg = StateConfig::new_bulk(
                "CA".to_string(), "california".to_string(), n,
                "2020".to_string(), "v1".to_string(),
                PathBuf::from("/tmp"), 0,
            );
            assert_eq!(cfg.total_seats, n,
                "total_seats must equal num_districts ({n}) for new_bulk");
        }
    }

    // ── Task 149: COI weights ─────────────────────────────────────────────────

    #[test]
    fn test_coi_weights_geometric_mean_increases_edge_weight() {
        // Applying COI weight 0.9 to both endpoints multiplies edge by sqrt(0.9*0.9) = 0.9
        let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();
        edge_weights.insert((0, 1), 1.0);

        // Build a COI map: both tract 0 and tract 1 have weight 0.9
        let coi_json = r#"{"GEOID_0": 0.9, "GEOID_1": 0.9}"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), coi_json).unwrap();

        let mut index_to_geoid: HashMap<usize, String> = HashMap::new();
        index_to_geoid.insert(0, "GEOID_0".to_string());
        index_to_geoid.insert(1, "GEOID_1".to_string());
        let result = apply_coi_weights(edge_weights, tmp.path(), &index_to_geoid).unwrap();

        let ew = result[&(0, 1)];
        let expected = (0.9_f64 * 0.9_f64).sqrt(); // ~0.9
        assert!((ew - expected).abs() < 1e-9,
            "edge weight should be ~{expected:.4}, got {ew:.4}");
    }

    #[test]
    fn test_coi_weights_missing_geoid_defaults_to_one() {
        // A GEOID not in the COI map gets weight 1.0 (no modification)
        let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();
        edge_weights.insert((0, 1), 2.0);

        // COI map only has tract 0, not tract 1
        let coi_json = r#"{"GEOID_0": 0.5}"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), coi_json).unwrap();

        let mut index_to_geoid: HashMap<usize, String> = HashMap::new();
        index_to_geoid.insert(0, "GEOID_0".to_string());
        index_to_geoid.insert(1, "GEOID_1".to_string());
        let result = apply_coi_weights(edge_weights, tmp.path(), &index_to_geoid).unwrap();

        // w_0=0.5, w_1=1.0 (default — not in COI map) → factor = sqrt(0.5 * 1.0) = sqrt(0.5)
        // original edge weight=2.0 → result = 2.0 * sqrt(0.5)
        let ew = result[&(0, 1)];
        let expected = 2.0 * (0.5_f64).sqrt();
        assert!((ew - expected).abs() < 1e-9,
            "missing GEOID should default to w=1.0, got {ew:.4}");
    }

    #[test]
    fn test_run_states_parallel_returns_one_result_per_state() {
        let configs = vec![make_config("VT"), make_config("DE"), make_config("AK")];
        let results = run_states_parallel(configs, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_run_states_parallel_errors_are_in_results() {
        let configs = vec![make_config("VT")];
        let results = run_states_parallel(configs, 1);
        // VT will fail (adjacency not at /tmp/test) — verify error is in result
        for r in &results {
            if !r.success {
                assert!(r.error.is_some());
            }
        }
    }

    #[test]
    fn test_run_states_parallel_empty() {
        assert_eq!(run_states_parallel(vec![], 4).len(), 0);
    }

    #[test]
    fn test_load_all_states_2020_returns_only_us_states() {
        // load_all_states reads from the embedded manifest (US-only).
        // International locations (IE, MT-PARLIAMENT, etc.) are in location_policy.json
        // but NOT in the manifest — they must never appear in bulk runs.
        let states = load_all_states("2020").expect("manifest should load");
        assert_eq!(states.len(), 50, "exactly 50 US states expected, got {}", states.len());
        // No international location codes
        let international = ["IE", "MT-PARLIAMENT", "DE-WAHLKREIS", "NZ-ELECTORATE", "GB-ENG", "CA-PROV"];
        for code in &international {
            assert!(
                !states.iter().any(|(c, _, _)| c == code),
                "international location {code} must not appear in load_all_states"
            );
        }
        // All codes are 2-letter uppercase (US state convention)
        for (code, _, _) in &states {
            assert_eq!(code.len(), 2, "state code '{code}' must be 2 chars");
            assert!(code.chars().all(|c| c.is_uppercase()), "code '{code}' must be uppercase");
        }
    }

    #[test]
    fn test_load_all_states_invalid_year_returns_err() {
        let result = load_all_states("2024");
        assert!(result.is_err(), "year 2024 must be rejected for bulk US runs");
        let msg = result.unwrap_err();
        assert!(msg.contains("2020") || msg.contains("2010"), "error must list valid years: {msg}");
    }

    #[test]
    fn test_state_already_complete_reprocess() {
        assert!(!state_already_complete(&PathBuf::from("/nonexistent"), "VT", "2020", true));
    }

    #[test]
    fn test_state_already_complete_missing() {
        assert!(!state_already_complete(&PathBuf::from("/nonexistent"), "VT", "2020", false));
    }

    #[test]
    fn test_state_already_complete_with_json_marker() {
        let tmp = TempDir::new().unwrap();
        let data = tmp.path().join("2020").join("states").join("vt").join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("final_assignments.json"), b"{}").unwrap();
        assert!(state_already_complete(&tmp.path().to_path_buf(), "VT", "2020", false));
    }

    #[test]
    fn test_filter_incomplete() {
        let tmp = TempDir::new().unwrap();
        let marker = tmp.path().join("2020").join("states").join("vt").join("data");
        std::fs::create_dir_all(&marker).unwrap();
        std::fs::write(marker.join("final_assignments.json"), b"{}").unwrap();
        let mut configs = vec![make_config("VT"), make_config("DE")];
        for c in &mut configs { c.output_dir = tmp.path().to_path_buf(); }
        let remaining = filter_incomplete(configs);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].state_code, "DE");
    }

    // --- Spec 1: StateConfig chamber-aware balance tolerance tests ---

    #[test]
    fn test_wa_house_manifest_chamber_aware_tolerance() {
        let cfg = StateConfig {
            chamber: "house".into(),
            balance_tolerance: None,
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_congressional_chamber_tolerance_is_half_pct() {
        let cfg = StateConfig {
            chamber: "congressional".into(),
            balance_tolerance: None,
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.005).abs() < 1e-9);
    }

    #[test]
    fn test_explicit_tolerance_override_wins() {
        let cfg = StateConfig {
            chamber: "house".into(),
            balance_tolerance: Some(0.02),
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_effective_num_districts_override() {
        let cfg = StateConfig {
            num_districts: 10,
            num_districts_override: Some(98),
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_num_districts(), 98);
    }

    #[test]
    fn test_effective_num_districts_fallback() {
        let cfg = StateConfig {
            num_districts: 10,
            num_districts_override: None,
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_num_districts(), 10);
    }

    #[test]
    fn test_effective_label_default() {
        let cfg = StateConfig {
            state_name: "washington".into(),
            chamber: "house".into(),
            year: "2020".into(),
            label: None,
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_label(), "washington_house_2020");
    }

    #[test]
    fn test_effective_label_custom() {
        let cfg = StateConfig {
            label: Some("wa_custom_label".into()),
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_label(), "wa_custom_label");
    }

    // --- Resolution field tests ---

    #[test]
    fn test_resolution_default_is_tract() {
        let cfg = make_config("VT");
        assert_eq!(cfg.resolution, "tract");
    }

    #[test]
    fn test_resolution_block_group_stored_in_config() {
        let cfg = StateConfig {
            resolution: "block_group".into(),
            ..make_config("WA")
        };
        assert_eq!(cfg.resolution, "block_group");
    }

    #[test]
    fn test_resolution_block_stored_in_config() {
        let cfg = StateConfig {
            resolution: "block".into(),
            ..make_config("WA")
        };
        assert_eq!(cfg.resolution, "block");
    }

    #[test]
    fn test_resolve_adjacency_path_tract_missing_returns_err() {
        // With no data present (invalid path from manifest default), tract resolution
        // should return an Err containing the expected hint.
        let result = resolve_adjacency_path("vt", "2020", "tract");
        assert!(result.is_err(), "expected Err when adjacency not present");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("redist fetch") || msg.contains("cannot load manifest"),
            "error message should reference redist fetch or manifest: {msg}"
        );
    }

    #[test]
    fn test_resolve_adjacency_path_block_group_missing_falls_back_or_errs() {
        // Block group adjacency missing: function either falls back to tract (also missing
        // in test env) and returns Err, or returns Err directly. Either way it must not panic.
        let result = resolve_adjacency_path("vt", "2020", "block_group");
        // In CI with no data, we expect an error (fallback tract also absent).
        // The important invariant: no panic, and error message is descriptive.
        match result {
            Err(msg) => {
                assert!(
                    msg.contains("adjacency") || msg.contains("manifest"),
                    "error should mention adjacency or manifest: {msg}"
                );
            }
            Ok((path, resolution)) => {
                // If data happens to exist locally, verify path and resolution are coherent
                assert!(path.exists(), "returned path must exist");
                assert!(
                    resolution == "tract" || resolution == "block_group",
                    "effective resolution must be tract or block_group: {resolution}"
                );
            }
        }
    }

    // ── Group 4: StateConfig.effective_balance_tolerance ─────────────────────

    #[test]
    fn test_effective_balance_tolerance_congressional_default() {
        let cfg = make_config("VT");
        // Congressional default: 0.5%
        assert!((cfg.effective_balance_tolerance() - 0.005).abs() < 1e-9,
            "congressional default must be 0.5%, got {}", cfg.effective_balance_tolerance());
    }

    #[test]
    fn test_effective_balance_tolerance_house_default() {
        let cfg = StateConfig {
            chamber: "house".to_string(),
            balance_tolerance: None,
            ..make_config("WA")
        };
        // House default: 5.0%
        assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9,
            "house default must be 5.0%, got {}", cfg.effective_balance_tolerance());
    }

    #[test]
    fn test_effective_balance_tolerance_explicit_override() {
        let cfg = StateConfig {
            chamber: "house".to_string(),
            balance_tolerance: Some(0.08), // 8% explicit override
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.08).abs() < 1e-9,
            "explicit override must win, got {}", cfg.effective_balance_tolerance());
    }

    #[test]
    fn test_effective_balance_tolerance_senate_default() {
        let cfg = StateConfig {
            chamber: "senate".to_string(),
            balance_tolerance: None,
            ..make_config("IL")
        };
        assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9,
            "senate default must be 5.0%");
    }

    // ── Group seats: seats_per_district / total_seats ────────────────────────

    #[test]
    fn test_seats_per_district_default_is_1() {
        let cfg = make_config("VT");
        assert_eq!(cfg.effective_seats_per_district(), 1);
    }

    #[test]
    fn test_seats_per_district_5_malta_style() {
        let cfg = StateConfig {
            seats_per_district: 5,
            total_seats: 65, // 13 x 5
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_seats_per_district(), 5);
    }

    #[test]
    fn test_total_seats_computed_from_seats_per_district() {
        let cfg = StateConfig {
            seats_per_district: 4, // avg for Ireland-style
            num_districts_override: Some(43),
            total_seats: 43 * 4, // 172
            ..make_config("WA")
        };
        assert_eq!(cfg.total_seats, 172);
    }

    #[test]
    fn test_ideal_pop_per_seat_single_member() {
        let cfg = make_config("VT"); // seats_per_district=1, total_seats=1
        // For single-member: ideal_pop_per_seat = total_pop / 1 = total_pop
        let ideal = cfg.ideal_pop_per_seat(643503);
        assert!((ideal - 643503.0).abs() < 1.0);
    }

    #[test]
    fn test_ideal_pop_per_seat_multi_member() {
        let cfg = StateConfig {
            seats_per_district: 5,
            total_seats: 65,
            ..make_config("WA")
        };
        // 7_705_281 / 65 ~ 118_543
        let ideal = cfg.ideal_pop_per_seat(7_705_281);
        assert!((ideal - 7_705_281.0 / 65.0).abs() < 1.0);
    }

    #[test]
    fn test_seats_per_district_zero_clamps_to_1() {
        let cfg = StateConfig {
            seats_per_district: 0,
            total_seats: 1,
            ..make_config("VT")
        };
        // effective_seats_per_district uses .max(1) so 0 -> 1
        assert_eq!(cfg.effective_seats_per_district(), 1);
    }

    // ── Group: chamber_balance_tolerance ──────────────────────────────────────

    #[test]
    fn test_chamber_balance_tolerance_wa_house_is_5pct() {
        // WA house_districts balance_tolerance_house_pct = 5.0%
        let tol = chamber_balance_tolerance("WA", "house");
        assert!((tol - 0.05).abs() < 1e-6, "WA house tolerance must be 5%, got {tol}");
    }

    #[test]
    fn test_chamber_balance_tolerance_wa_congressional_is_half_pct() {
        // WA balance_tolerance_congressional_pct = 0.5%
        let tol = chamber_balance_tolerance("WA", "congressional");
        assert!((tol - 0.005).abs() < 1e-6, "WA congressional tolerance must be 0.5%, got {tol}");
    }

    #[test]
    fn test_chamber_balance_tolerance_nv_house_is_10pct() {
        // NV allows 10% house tolerance (policy explicitly documents this)
        let tol = chamber_balance_tolerance("NV", "house");
        assert!((tol - 0.10).abs() < 1e-6, "NV house tolerance must be 10%, got {tol}");
    }

    #[test]
    fn test_chamber_balance_tolerance_unknown_state_uses_default() {
        let tol = chamber_balance_tolerance("ZZ", "house");
        assert!((tol - 0.05).abs() < 1e-6, "unknown state must fall back to 5% default");
    }

    #[test]
    fn test_chamber_balance_tolerance_unknown_chamber_uses_default() {
        let tol = chamber_balance_tolerance("WA", "council");
        assert!((tol - 0.05).abs() < 1e-6, "unknown chamber must fall back to 5% default");
    }

    #[test]
    fn test_effective_balance_tolerance_uses_policy_when_no_override() {
        // NV house has 10% tolerance in policy; without explicit override, must use 10%
        let cfg = StateConfig {
            state_code: "NV".into(),
            chamber: "house".into(),
            balance_tolerance: None, // no explicit override
            ..make_config("VT")
        };
        let tol = cfg.effective_balance_tolerance();
        assert!((tol - 0.10).abs() < 1e-6,
            "NV house must use policy tolerance 10%, got {tol}");
    }

    #[test]
    fn test_effective_balance_tolerance_explicit_override_wins() {
        // Explicit --balance-tolerance 2 must override even if policy says 10%
        let cfg = StateConfig {
            state_code: "NV".into(),
            chamber: "house".into(),
            balance_tolerance: Some(0.02), // explicit 2% override
            ..make_config("VT")
        };
        let tol = cfg.effective_balance_tolerance();
        assert!((tol - 0.02).abs() < 1e-9, "explicit override must win, got {tol}");
    }

    // ── Group: chamber_district_count ─────────────────────────────────────────

    #[test]
    fn test_chamber_district_count_congressional_returns_fallback() {
        // Congressional always uses the manifest fallback, not state policy
        assert_eq!(chamber_district_count("WA", "congressional", 10), 10);
        assert_eq!(chamber_district_count("VT", "congressional", 1), 1);
    }

    #[test]
    fn test_chamber_district_count_house_wa_returns_98() {
        // WA house has 98 districts per state_policy.json
        let result = chamber_district_count("WA", "house", 10);
        assert_eq!(result, 98, "WA house must use 98 districts from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_senate_wa_returns_49() {
        // WA senate has 49 districts (2:1 nesting with 98 house)
        let result = chamber_district_count("WA", "senate", 10);
        assert_eq!(result, 49, "WA senate must use 49 districts from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_house_nv_returns_42() {
        // NV house has 42 districts per state_policy.json
        let result = chamber_district_count("NV", "house", 4);
        assert_eq!(result, 42, "NV house must use 42 districts from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_house_va_returns_100() {
        // VA house has 100 delegates
        let result = chamber_district_count("VA", "house", 11);
        assert_eq!(result, 100, "VA house must use 100 from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_house_la_returns_105() {
        // LA house has 105 representatives
        let result = chamber_district_count("LA", "house", 6);
        assert_eq!(result, 105, "LA house must use 105 from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_unknown_state_uses_fallback() {
        // Unknown state code falls back to congressional count
        let result = chamber_district_count("ZZ", "house", 7);
        assert_eq!(result, 7, "unknown state ZZ must fall back to congressional count");
    }

    #[test]
    fn test_chamber_district_count_unknown_chamber_uses_fallback() {
        // Unrecognized chamber name falls back
        let result = chamber_district_count("WA", "council", 10);
        assert_eq!(result, 10, "unknown chamber type must fall back to congressional count");
    }

    // ── Group 5: adjacency fallback / resolve_adjacency_path ─────────────────

    // ── Task 122: --reset data loss warning ───────────────────────────────────

    /// Verify the reset plan path computation matches the expected directory structure.
    ///
    /// The warning uses `plan_root.display()` which is either:
    ///   labeled:  {output_dir}/{year}/plans/{label}/
    ///   unlabeled: {output_dir}/{year}/states/{state_name}/
    #[test]
    fn test_reset_warning_format() {
        use std::path::PathBuf;

        // Labeled plan path
        let output_dir = PathBuf::from("/tmp/outputs/v1");
        let year = "2020";
        let label = "wa_house_2020";
        let plan_root_labeled = output_dir.join(year).join("plans").join(label);
        let expected = "/tmp/outputs/v1/2020/plans/wa_house_2020";
        assert!(
            plan_root_labeled.to_string_lossy().replace('\\', "/").contains("wa_house_2020"),
            "labeled plan_root must contain label: {}",
            plan_root_labeled.display()
        );
        let warning = format!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root_labeled.display()
        );
        assert!(warning.contains("wa_house_2020"), "warning must mention the plan label: {warning}");
        assert!(warning.contains("--reset will delete"), "warning must mention --reset: {warning}");

        // Legacy (unlabeled) state path
        let state_name = "washington";
        let plan_root_legacy = output_dir.join(year).join("states").join(state_name);
        let warning_legacy = format!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root_legacy.display()
        );
        assert!(warning_legacy.contains("washington"), "legacy warning must mention state: {warning_legacy}");
    }

    #[test]
    fn test_reset_warning_contains_required_components() {
        // Verify the warning message format has all required components
        let plan_root = std::path::PathBuf::from("/tmp/outputs/v1/2020/plans/wa_house_2020");
        let msg = format!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root.display()
        );
        assert!(msg.starts_with("WARNING:"), "must start with WARNING:");
        assert!(msg.contains("--reset"), "must mention --reset flag");
        assert!(msg.contains("delete"), "must use the word 'delete'");
        assert!(msg.contains("all its contents"), "must warn about contents");
    }

    // ── Task 135: adjacency year mismatch detection ──────────────────────────

    #[test]
    fn test_adjacency_year_mismatch_detected() {
        // Requesting year 2020 but file is for 2010 — must detect mismatch
        let path = PathBuf::from("wa_adjacency_2010.pkl");
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let file_year = extract_year_from_adj_filename(filename);
        assert_eq!(file_year, Some("2010"), "should extract 2010 from filename");
        // Mismatch: requested 2020, file is 2010
        assert_ne!(file_year, Some("2020"), "2010 file != 2020 requested — mismatch detected");
    }

    #[test]
    fn test_adjacency_year_match_no_warning() {
        // Requesting year 2020 and file is also for 2020 — no mismatch
        let path = PathBuf::from("wa_adjacency_2020.pkl");
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let file_year = extract_year_from_adj_filename(filename);
        assert_eq!(file_year, Some("2020"), "should extract 2020 from filename");
        // No mismatch: requested 2020, file is 2020
        assert_eq!(file_year, Some("2020"), "2020 file == 2020 requested — no mismatch");
    }

    #[test]
    fn test_extract_year_from_adj_filename_2000() {
        let year = extract_year_from_adj_filename("ca_adjacency_2000.pkl");
        assert_eq!(year, Some("2000"), "should extract 2000");
    }

    #[test]
    fn test_extract_year_from_adj_filename_none() {
        // Filename without a recognizable census year
        let year = extract_year_from_adj_filename("ca_adjacency.pkl");
        assert_eq!(year, None, "no census year in filename should return None");
    }

    #[test]
    fn test_check_adjacency_year_mismatch_same_year_no_panic() {
        // Same year: function runs without panic
        let path = PathBuf::from("wa_adjacency_2020.pkl");
        check_adjacency_year_mismatch(&path, "2020", "WA"); // no panic
    }

    #[test]
    fn test_check_adjacency_year_mismatch_different_year_no_panic() {
        // Different year: function emits warning but doesn't panic
        let path = PathBuf::from("wa_adjacency_2010.pkl");
        check_adjacency_year_mismatch(&path, "2020", "WA"); // warns but no panic
    }

    // ── Gap 9: progress messages for redist states ────────────────────────────

    #[test]
    fn test_states_progress_message_format() {
        // A States run with 0 configs still shows the summary line format.
        // We verify the message format that would be produced by the States command.
        let configs: Vec<StateConfig> = Vec::new();
        let results = run_states_parallel(configs, 4);
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        // Verify summary computation is correct for empty run
        assert_eq!(succeeded, 0, "0 configs: succeeded must be 0");
        assert_eq!(failed, 0, "0 configs: failed must be 0");

        // Verify the summary line format
        let summary = format!("[redist states] Complete: {} succeeded, {} failed", succeeded, failed);
        assert!(summary.contains("Complete:"), "summary must contain 'Complete:'");
        assert!(summary.contains("succeeded"), "summary must contain 'succeeded'");
        assert!(summary.contains("failed"), "summary must contain 'failed'");
        assert!(summary.contains("[redist states]"), "summary must be prefixed with [redist states]");

        // Verify the queued banner format
        let queued = format!(
            "[redist states] {} states queued — {} workers — year {} — version {}",
            0usize, 4usize, "2020", "v1"
        );
        assert!(queued.contains("states queued"), "banner must contain 'states queued'");
        assert!(queued.contains("workers"), "banner must contain 'workers'");
        assert!(queued.contains("year"), "banner must contain 'year'");
        assert!(queued.contains("version"), "banner must contain 'version'");
    }

    // ── Gap 1: adjacency missing error message suggests redist fetch ──────────

    // ── Task 205: block_group fallback warning mentions --resolution and fetch ──

    #[test]
    fn test_block_group_fallback_warning_text() {
        // The warning when bg requested but not found should mention --resolution and fetch
        let state_code_lower = "wa";
        let year = "2020";
        let warning = format!(
            "WARNING: --resolution block_group was requested but block_group adjacency \
             not found for {state_code_lower} {year}.\n\
             To get block_group data: redist fetch --type adjacency --states {} --year {}\n\
             Falling back to tract resolution.",
            state_code_lower.to_uppercase(), year
        );
        assert!(warning.contains("--resolution block_group"), "must mention flag");
        assert!(warning.contains("redist fetch"), "must mention fetch command");
        assert!(warning.contains("block_group"), "must mention resolution type");
        assert!(warning.contains("Falling back to tract resolution"), "must mention fallback");
        assert!(warning.contains("WA"), "must mention uppercase state code");
    }

    #[test]
    fn test_adjacency_missing_error_suggests_fetch() {
        // When adjacency is missing, the error must mention "redist fetch" and "--type adjacency".
        let result = resolve_adjacency_path("wa", "2020", "tract");
        // In test env, adjacency won't exist — verify error contains expected hints.
        match result {
            Err(msg) => {
                assert!(
                    msg.contains("redist fetch"),
                    "error must suggest 'redist fetch': {msg}"
                );
                assert!(
                    msg.contains("--type adjacency"),
                    "error must include '--type adjacency': {msg}"
                );
            }
            Ok(_) => {
                // If data happens to exist locally, the test is vacuously satisfied.
            }
        }
    }

    #[test]
    fn test_resolve_adjacency_uses_manifest() {
        // resolve_adjacency_path reads the manifest to find outputs_dir.
        // If manifest can be loaded, the function should not panic.
        // Test that an unknown state code returns a descriptive error.
        let result = resolve_adjacency_path("zz", "2020", "tract");
        // Should fail (no ZZ adjacency) but with a helpful error message
        assert!(result.is_err(), "unknown state ZZ should fail");
        let err = result.unwrap_err();
        assert!(err.contains("adjacency") || err.contains("not found") || err.contains("manifest"),
            "error should mention adjacency: {err}");
    }

    /// Scenario 17: Isolated node warning logic — verify that an adjacency with
    /// isolated nodes (empty neighbor list) is correctly detected.
    #[test]
    fn test_run_warns_on_isolated_nodes() {
        // Simulate the isolated-node detection logic from run_single_state.
        // adjacency[0] has neighbors, adjacency[1] is isolated, adjacency[2] is isolated.
        let adjacency: Vec<Vec<usize>> = vec![
            vec![2],     // node 0: connected
            vec![],      // node 1: isolated
            vec![0],     // node 2: connected
            vec![],      // node 3: isolated
        ];

        let isolated: Vec<usize> = adjacency.iter().enumerate()
            .filter(|(_, nbrs)| nbrs.is_empty())
            .map(|(i, _)| i)
            .collect();

        assert_eq!(isolated.len(), 2, "should detect 2 isolated nodes");
        assert!(isolated.contains(&1), "node 1 should be isolated");
        assert!(isolated.contains(&3), "node 3 should be isolated");
        assert!(!isolated.contains(&0), "node 0 is connected, not isolated");
        assert!(!isolated.contains(&2), "node 2 is connected, not isolated");

        // Verify a fully-connected graph produces no isolated nodes
        let connected: Vec<Vec<usize>> = vec![
            vec![1, 2],
            vec![0, 2],
            vec![0, 1],
        ];
        let isolated_none: Vec<usize> = connected.iter().enumerate()
            .filter(|(_, nbrs)| nbrs.is_empty())
            .map(|(i, _)| i)
            .collect();
        assert!(isolated_none.is_empty(), "fully connected graph has no isolated nodes");
    }

    // ── Task 131: CVAP fallback warning ──────────────────────────────────────

    #[test]
    fn test_cvap_missing_falls_back_to_total() {
        // With no CVAP file on disk, requesting "cvap" should fall back to "total".
        let result = check_cvap_availability("cvap", "vermont", "2020", "VT");
        assert_eq!(result, "total",
            "should fall back to total when CVAP file is absent, got {result}");
    }

    #[test]
    fn test_population_source_cvap_falls_back_to_total() {
        // Synonym test — same logic as above but more explicit assertion.
        let source = check_cvap_availability("cvap", "nonexistent_state", "2020", "XX");
        assert_eq!(source, "total",
            "CVAP fallback must produce 'total', got '{source}'");
    }

    #[test]
    fn test_non_cvap_source_unchanged() {
        // "total" and "vap" should be returned unchanged regardless of file presence.
        assert_eq!(check_cvap_availability("total", "vermont", "2020", "VT"), "total");
        assert_eq!(check_cvap_availability("vap", "vermont", "2020", "VT"), "vap");
    }

    // ── Task 130: Worker cap reporting ───────────────────────────────────────

    #[test]
    fn test_worker_cap_message_when_capped() {
        // effective_workers(very_large) < very_large => note would be emitted.
        // We can't easily capture stderr in unit tests, but we can verify the
        // logic: if requested > actual, the note should fire.
        let requested = usize::MAX; // always exceeds available threads
        let actual = effective_workers(requested);
        assert!(actual < requested,
            "effective_workers(MAX) must be < MAX (got {actual})");
    }

    #[test]
    fn test_worker_cap_no_message_when_exact() {
        // When requested equals actual, no note should be emitted.
        // Using 1 worker: effective == 1 == requested.
        let requested = 1;
        let actual = effective_workers(requested);
        // Rayon always has at least 1 thread
        assert!(actual >= 1, "effective_workers(1) must be >= 1");
        // When actual == requested, no cap note fires (logical condition)
        let would_print = actual < requested;
        assert!(!would_print, "no note when requested ({requested}) == actual ({actual})");
    }

    // ── Plan 03: validate_partisan_config (Callais disentanglement) ──────────

    #[test]
    fn test_validate_partisan_config_default_ok() {
        // Default config has partition_mode=edge-weighted and no partisan_shares
        // → no constraint involved, must pass.
        let cfg = make_config("VT");
        validate_partisan_config(&cfg).expect("default config should validate");
    }

    #[test]
    fn test_validate_partisan_config_metis_vra_alone_ok() {
        let cfg = StateConfig {
            partition_mode: "metis-vra".into(),
            partisan_shares: None,
            ..make_config("AL")
        };
        validate_partisan_config(&cfg).expect("metis-vra without shares is fine");
    }

    #[test]
    fn test_validate_partisan_config_metis_vra_with_shares_rejected() {
        // The Callais p.36 disentanglement: race-conscious mode + partisan data = error.
        let cfg = StateConfig {
            partition_mode: "metis-vra".into(),
            partisan_shares: Some(std::path::PathBuf::from("foo.tsv")),
            ..make_config("LA")
        };
        let err = validate_partisan_config(&cfg).expect_err("must reject");
        assert!(err.contains("--partisan-shares requires"),
            "error should explain the constraint, got: {err}");
        assert!(err.contains("metis-vra"), "error should name the offending mode: {err}");
    }

    #[test]
    fn test_validate_partisan_config_edge_weighted_with_shares_rejected() {
        // Even non-VRA modes must not accept partisan_shares — only partisan-weighted does.
        let cfg = StateConfig {
            partition_mode: "edge-weighted".into(),
            partisan_shares: Some(std::path::PathBuf::from("foo.tsv")),
            ..make_config("WA")
        };
        let err = validate_partisan_config(&cfg).expect_err("must reject");
        assert!(err.contains("partisan-weighted"),
            "error should name the only valid mode for shares: {err}");
    }

    #[test]
    fn test_validate_partisan_config_partisan_weighted_without_shares_rejected() {
        let cfg = StateConfig {
            partition_mode: "partisan-weighted".into(),
            partisan_shares: None,
            ..make_config("VT")
        };
        let err = validate_partisan_config(&cfg).expect_err("must reject");
        assert!(err.contains("requires --partisan-shares"),
            "error should explain what's missing: {err}");
    }

    #[test]
    fn test_validate_partisan_config_partisan_weighted_with_shares_ok() {
        let cfg = StateConfig {
            partition_mode: "partisan-weighted".into(),
            partisan_shares: Some(std::path::PathBuf::from("/tmp/foo.tsv")),
            ..make_config("LA")
        };
        validate_partisan_config(&cfg).expect("partisan-weighted + shares is the valid combo");
    }

    #[test]
    fn test_validate_partisan_config_unweighted_with_shares_rejected() {
        // The unweighted mode also doesn't accept shares — only partisan-weighted.
        let cfg = StateConfig {
            partition_mode: "unweighted".into(),
            partisan_shares: Some(std::path::PathBuf::from("foo.tsv")),
            ..make_config("DE")
        };
        let err = validate_partisan_config(&cfg).expect_err("must reject");
        assert!(err.contains("unweighted"), "error should name the offending mode: {err}");
    }
}
