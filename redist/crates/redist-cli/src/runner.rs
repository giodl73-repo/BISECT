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
use crate::bisection_runner::run_all_splits;
use crate::demographics::{load_demographics, align_demographics_to_adjacency};
use crate::output::{write_state_outputs, clean_corrupt_state, VraAnalysis, VraDistrict};
use crate::status::{status, ascii_safe};
use redist_core::{Partition, build_vra_edge_weights};
use redist_analysis::analyze_mm_districts;

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
}

/// Load all state codes, names, and district counts for a given year.
/// Returns Vec<(state_code, state_name, num_districts)> sorted alphabetically.
/// Calls Python ONCE — not once per state.
pub fn load_all_states(year: &str) -> Result<Vec<(String, String, usize)>, String> {
    let script = format!(
        "from scripts.config_{year} import STATE_CONFIG_{year}; \
         import json; \
         print(json.dumps({{k: {{'districts': v['districts'], 'name': v['name'].lower().replace(' ','_')}} \
         for k, v in STATE_CONFIG_{year}.items()}}))",
        year = year
    );
    let python_cmd = std::env::var("REDIST_PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) { "py".to_string() } else { "python3".to_string() }
    });
    let out = std::process::Command::new(&python_cmd)
        .args(["-c", &script])
        .output()
        .map_err(|e| format!("failed to invoke {python_cmd} for state config: {e}"))?;

    if !out.status.success() {
        return Err(format!(
            "python config load failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let map: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;

    let mut states: Vec<(String, String, usize)> = map.into_iter()
        .filter_map(|(code, val)| {
            let name = val["name"].as_str()?.to_string();
            let districts = val["districts"].as_u64()? as usize;
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

/// Run a single state redistricting task end-to-end.
///
/// Flow: load adjacency → build edge weights → bisect → assert balance → write outputs
fn run_single_state(cfg: &StateConfig) -> Result<(), String> {
    let num_districts = cfg.num_districts;
    let state_name = &cfg.state_name; // e.g. "vermont" — used for directory paths

    // Data directory: use full lowercase state name (e.g. "vermont"), matching Python pipeline
    let data_dir = cfg.output_dir
        .join("states")
        .join(state_name)
        .join("data");

    // Reset: delete existing outputs before starting
    if cfg.reset && data_dir.exists() {
        std::fs::remove_dir_all(&data_dir)
            .map_err(|e| format!("reset failed: {e}"))?;
    }
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("cannot create data dir: {e}"))?;

    status(cfg.position, &format!("{}: loading adjacency", cfg.state_code));

    // 1. Load adjacency graph (Python subprocess shim: pkl → JSON)
    // Adjacency lives at outputs/{version}/data/{year}/adjacency/ — independent
    // of where state results are written (output_dir may be a different path).
    // Adjacency uses two-letter state code (lowercase), e.g. "vt_adjacency_2020.pkl"
    let state_code_lower = cfg.state_code.to_lowercase();
    let adjacency_base = PathBuf::from(format!("outputs/{}", cfg.version));
    let adj_pkl = adjacency_base
        .join("data").join(&cfg.year).join("adjacency")
        .join(format!("{state_code_lower}_adjacency_{}.pkl", cfg.year));

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

    // 3. Run bisection
    status(cfg.position, &format!("{}: bisecting into {} districts", cfg.state_code, num_districts));
    let assignments = run_all_splits(
        &graph.adjacency,
        &graph.vertex_weights,
        &edge_weights,
        num_districts,
        cfg.ufactor,
        cfg.niter,
        cfg.seed,
    ).map_err(|e| format!("bisection failed: {e}"))?;

    // 4. Assert constitutional population balance (±0.5%)
    // BOUNDARY invariant: 0.5% matches one-person-one-vote standard.
    let partition = Partition::from_assignments(assignments.clone());
    partition.assert_balanced(&graph.vertex_weights, num_districts, 0.005)
        .map_err(|e| format!("population balance violation: {e}"))?;

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

    status(cfg.position, &format!("{}: complete ({num_districts}D, {}ms)", cfg.state_code, 0));
    Ok(())
}

/// Check if a state's outputs already exist and are complete.
pub fn state_already_complete(output_dir: &PathBuf, state_code: &str, reprocess: bool) -> bool {
    if reprocess { return false; }
    let data_dir = output_dir
        .join("states").join(state_code.to_lowercase()).join("data");
    data_dir.join("final_assignments.json").exists()
        || data_dir.join("final_assignments.pkl").exists()
}

/// Filter configs to only those needing processing.
pub fn filter_incomplete(configs: Vec<StateConfig>) -> Vec<StateConfig> {
    configs.into_iter()
        .filter(|cfg| !state_already_complete(&cfg.output_dir, &cfg.state_code, cfg.reprocess))
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
        assert!(!state_already_complete(&PathBuf::from("/nonexistent"), "VT", true));
    }

    #[test]
    fn test_state_already_complete_missing() {
        assert!(!state_already_complete(&PathBuf::from("/nonexistent"), "VT", false));
    }

    #[test]
    fn test_state_already_complete_with_json_marker() {
        let tmp = TempDir::new().unwrap();
        let data = tmp.path().join("states").join("vt").join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("final_assignments.json"), b"{}").unwrap();
        assert!(state_already_complete(&tmp.path().to_path_buf(), "VT", false));
    }

    #[test]
    fn test_filter_incomplete() {
        let tmp = TempDir::new().unwrap();
        let marker = tmp.path().join("states").join("vt").join("data");
        std::fs::create_dir_all(&marker).unwrap();
        std::fs::write(marker.join("final_assignments.json"), b"{}").unwrap();
        let mut configs = vec![make_config("VT"), make_config("DE")];
        for c in &mut configs { c.output_dir = tmp.path().to_path_buf(); }
        let remaining = filter_incomplete(configs);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].state_code, "DE");
    }
}
