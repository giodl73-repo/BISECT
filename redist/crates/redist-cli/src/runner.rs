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
use crate::fetch::load_manifest;
use crate::output::{write_state_outputs, clean_corrupt_state, VraAnalysis, VraDistrict};
use crate::status::{status, ascii_safe};
use redist_core::{Partition, build_vra_edge_weights};
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
#[derive(Debug, Clone)]
pub struct StateConfig {
    pub state_code: String,
    /// Lowercase state name for file paths (e.g. "alabama"). Pre-resolved.
    pub state_name: String,
    /// Number of congressional districts. Pre-resolved from config_{year}.py.
    pub num_districts: usize,
    pub year: String,
    pub version: String,
    pub output_dir: PathBuf,
    pub partition_mode: String,
    pub position: i32,
    pub debug: bool,
    pub reset: bool,
    pub reprocess: bool,
    pub ufactor: u32,
    pub niter: u32,
    pub seed: Option<u64>,
    // Spec 1: custom parameters
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
}

impl StateConfig {
    /// Returns the effective balance tolerance based on chamber type.
    /// Congressional: ±0.5% (strict one-person-one-vote)
    /// State legislative (house/senate/custom): ±5.0% (looser standard)
    /// Explicit override always wins.
    pub fn effective_balance_tolerance(&self) -> f64 {
        self.balance_tolerance.unwrap_or_else(|| match self.chamber.as_str() {
            "congressional" => 0.005,
            _ => 0.05, // house, senate, custom — state legislative default
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

/// Run multiple states in parallel using Rayon.
/// Workers cap: min(workers, available_threads).
pub fn run_states_parallel(configs: Vec<StateConfig>, workers: usize) -> Vec<StateResult> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers.min(rayon::current_num_threads()))
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

/// Resolve the adjacency pkl path for a state using the manifest.
///
/// The manifest's `local_outputs_dir` + "V3/data/{year}/adjacency/" is the
/// canonical adjacency store — the same path that `redist fetch --release` downloads to.
/// Override with REDIST_MANIFEST env var for custom data layouts.
fn resolve_adjacency_path(state_code_lower: &str, year: &str) -> Result<PathBuf, String> {
    let manifest = load_manifest()
        .map_err(|e| format!("cannot load manifest: {e}"))?;
    let adj_filename = format!("{state_code_lower}_adjacency_{year}.pkl");
    // Manifest points to the canonical adjacency store (outputs_dir/V3/data/{year}/adjacency/)
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);
    let canonical = outputs_dir
        .join("V3").join("data").join(year).join("adjacency")
        .join(&adj_filename);
    if canonical.exists() {
        return Ok(canonical);
    }
    // Fallback: check V4 store (VRA adjacency lives there for some setups)
    let v4 = outputs_dir
        .join("V4").join("data").join(year).join("adjacency")
        .join(&adj_filename);
    if v4.exists() {
        return Ok(v4);
    }
    Err(format!(
        "adjacency pkl not found: {canonical}. \
         Run: redist fetch --year {year} --release && \
         python scripts/data/generate_adj_bin.py --year {year} --states {state_code_lower}",
        canonical = canonical.display()
    ))
}

/// Run a single state redistricting task end-to-end.
///
/// Flow: load adjacency → build edge weights → bisect → assert balance → write outputs
fn run_single_state(cfg: &StateConfig) -> Result<(), String> {
    let num_districts = cfg.effective_num_districts();
    let state_name = &cfg.state_name; // e.g. "vermont" — used for directory paths
    let label = cfg.effective_label();
    let balance_tolerance = cfg.effective_balance_tolerance();

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
    if cfg.reset && data_dir.exists() {
        std::fs::remove_dir_all(&data_dir)
            .map_err(|e| format!("reset failed: {e}"))?;
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
    let adj_pkl = resolve_adjacency_path(&state_code_lower, &cfg.year)?;

    let graph = load_adjacency_pkl(&adj_pkl)
        .map_err(|e| format!("adjacency load failed: {e}"))?;

    // 2. Build edge weights based on partition mode
    let edge_weights: HashMap<(usize, usize), f64> = match cfg.partition_mode.as_str() {
        "edge-weighted" => {
            status(cfg.position, &format!("{}: edge-weighted mode ({} edges)", cfg.state_code, graph.n_edges));
            graph.edge_weights.clone()
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
        run_all_splits(
            &graph.adjacency,
            &graph.vertex_weights,
            &edge_weights,
            num_districts,
            cfg.ufactor,
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

    // 7. Write manifest.json atomically (manifest.tmp → manifest.json).
    // Board amendment: atomic write (manifest.tmp + rename) prevents partial manifests.
    if cfg.write_manifest || cfg.label.is_some() {
        let adj_filename = format!("{}_adjacency_{}.adj.bin", state_name, cfg.year);
        let state_fips = resolve_state_fips(&cfg.state_code);
        let tiger_url = redist_report::tiger_source_url(&state_fips, &cfg.year);
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
            created_at: current_iso8601(),
            balance_tolerance_pct: balance_tolerance * 100.0,
            population_balance_valid: true,
        };
        redist_report::write_manifest_atomic(&plan_root, &manifest)
            .map_err(|e| format!("manifest write failed: {e}"))?;
    }

    status(cfg.position, &format!("{}: complete ({num_districts}D, {}ms)", cfg.state_code, 0));
    Ok(())
}

/// Resolve a 2-letter state code to its FIPS code string.
/// Falls back to "00" if not found.
fn resolve_state_fips(state_code: &str) -> String {
    // Standard FIPS codes for common states
    match state_code.to_uppercase().as_str() {
        "AL" => "01", "AK" => "02", "AZ" => "04", "AR" => "05", "CA" => "06",
        "CO" => "08", "CT" => "09", "DE" => "10", "FL" => "12", "GA" => "13",
        "HI" => "15", "ID" => "16", "IL" => "17", "IN" => "18", "IA" => "19",
        "KS" => "20", "KY" => "21", "LA" => "22", "ME" => "23", "MD" => "24",
        "MA" => "25", "MI" => "26", "MN" => "27", "MS" => "28", "MO" => "29",
        "MT" => "30", "NE" => "31", "NV" => "32", "NH" => "33", "NJ" => "34",
        "NM" => "35", "NY" => "36", "NC" => "37", "ND" => "38", "OH" => "39",
        "OK" => "40", "OR" => "41", "PA" => "42", "RI" => "44", "SC" => "45",
        "SD" => "46", "TN" => "47", "TX" => "48", "UT" => "49", "VT" => "50",
        "VA" => "51", "WA" => "53", "WV" => "54", "WI" => "55", "WY" => "56",
        _ => "00",
    }.to_string()
}

/// Return current time as ISO 8601 string (UTC).
fn current_iso8601() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let date = epoch_days_to_iso_date(days);
    format!("{date}T{h:02}:{m:02}:{s:02}Z")
}

fn epoch_days_to_iso_date(days: u64) -> String {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}")
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
        }
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
}
