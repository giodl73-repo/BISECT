/// Multi-state Rayon parallel runner.
///
/// Replaces run_states_parallel.py's multiprocessing pool. Rayon operates
/// without the GIL, enabling true CPU parallelism for METIS subprocess calls.
///
/// Each state runs as an independent task: load adjacency → run METIS
/// (subprocess, Phase 1c) → write outputs → assert balance.
/// States that fail write to an error log and do not block other states.
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::status::ascii_safe;

/// Result of processing a single state.
#[derive(Debug, Clone)]
pub struct StateResult {
    pub state_code: String,
    pub success: bool,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

/// Configuration for a single state run.
#[derive(Debug, Clone)]
pub struct StateConfig {
    pub state_code: String,
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

/// Run multiple states in parallel using Rayon.
///
/// Workers cap: min(workers, rayon::current_num_threads()).
/// Each state gets a unique tqdm position (2 + index) to avoid progress bar
/// collision — matching the Python run_states_parallel.py tqdm_position logic.
pub fn run_states_parallel(
    configs: Vec<StateConfig>,
    workers: usize,
) -> Vec<StateResult> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers.min(rayon::current_num_threads()))
        .build()
        .expect("failed to build Rayon thread pool");

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let results: Vec<StateResult> = pool.install(|| {
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
                Err(e) => {
                    let msg = format!("{}: {}", cfg.state_code, ascii_safe(&e.to_string()));
                    errors.lock().unwrap().push(msg.clone());
                    StateResult {
                        state_code: cfg.state_code.clone(),
                        success: false,
                        error: Some(msg),
                        elapsed_ms,
                    }
                }
            }
        }).collect()
    });

    results
}

/// Run a single state redistricting task.
/// Returns Ok(()) on success, Err(msg) on failure.
fn run_single_state(cfg: &StateConfig) -> Result<(), String> {
    // Phase 3d placeholder: invoke the Python pipeline via subprocess for now,
    // transitioning to pure Rust in later phases.
    // This stub demonstrates the interface contract.
    Err(format!(
        "run_single_state not yet implemented for {} (Phase 3d)",
        cfg.state_code
    ))
}

/// Check if a state's outputs already exist and are complete.
/// Returns true if outputs exist and --reprocess is not set.
pub fn state_already_complete(output_dir: &PathBuf, state_code: &str, reprocess: bool) -> bool {
    if reprocess {
        return false;
    }
    // A state is complete if its final_assignments.pkl exists
    let marker = output_dir
        .join("states")
        .join(state_code.to_lowercase())
        .join("data")
        .join("final_assignments.pkl");
    marker.exists()
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
    use std::path::Path;
    use tempfile::TempDir;

    fn make_config(state: &str) -> StateConfig {
        StateConfig {
            state_code: state.to_string(),
            year: "2020".to_string(),
            version: "V3".to_string(),
            output_dir: PathBuf::from("/tmp/test"),
            partition_mode: "edge-weighted".to_string(),
            position: 2,
            debug: false,
            reset: false,
            reprocess: false,
            ufactor: 5,
            niter: 100,
            seed: None,
        }
    }

    #[test]
    fn test_run_states_parallel_returns_one_result_per_state() {
        let configs = vec![
            make_config("VT"),
            make_config("DE"),
            make_config("AK"),
        ];
        let results = run_states_parallel(configs, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_run_states_parallel_all_fail_gracefully() {
        // The stub implementation fails each state — verify no panic
        let configs = vec![make_config("CA"), make_config("TX")];
        let results = run_states_parallel(configs, 2);
        for r in &results {
            assert!(!r.success); // stub always fails
            assert!(r.error.is_some());
            assert!(r.elapsed_ms < 10_000); // should not hang
        }
    }

    #[test]
    fn test_workers_cap_does_not_panic() {
        // workers=999 should be capped to available threads
        let configs = vec![make_config("VT")];
        let results = run_states_parallel(configs, 999);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_state_already_complete_reprocess_override() {
        // reprocess=true always returns false (force re-run)
        let dir = PathBuf::from("/nonexistent");
        assert!(!state_already_complete(&dir, "VT", true));
    }

    #[test]
    fn test_state_already_complete_missing_marker() {
        // File doesn't exist → not complete
        let dir = PathBuf::from("/nonexistent");
        assert!(!state_already_complete(&dir, "VT", false));
    }

    #[test]
    fn test_state_already_complete_with_marker() {
        // Create the marker file and verify detection
        let tmp = TempDir::new().unwrap();
        let marker_dir = tmp.path().join("states").join("vt").join("data");
        std::fs::create_dir_all(&marker_dir).unwrap();
        std::fs::write(marker_dir.join("final_assignments.pkl"), b"").unwrap();
        assert!(state_already_complete(&tmp.path().to_path_buf(), "VT", false));
    }

    #[test]
    fn test_filter_incomplete_removes_done_states() {
        let tmp = TempDir::new().unwrap();
        // Create marker for VT (done)
        let marker_dir = tmp.path().join("states").join("vt").join("data");
        std::fs::create_dir_all(&marker_dir).unwrap();
        std::fs::write(marker_dir.join("final_assignments.pkl"), b"").unwrap();

        let mut configs = vec![make_config("VT"), make_config("DE")];
        for c in &mut configs {
            c.output_dir = tmp.path().to_path_buf();
        }
        let remaining = filter_incomplete(configs);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].state_code, "DE");
    }

    #[test]
    fn test_filter_incomplete_reprocess_keeps_all() {
        let tmp = TempDir::new().unwrap();
        // Create marker for VT (done) but reprocess=true
        let marker_dir = tmp.path().join("states").join("vt").join("data");
        std::fs::create_dir_all(&marker_dir).unwrap();
        std::fs::write(marker_dir.join("final_assignments.pkl"), b"").unwrap();

        let mut configs = vec![make_config("VT")];
        configs[0].output_dir = tmp.path().to_path_buf();
        configs[0].reprocess = true;
        let remaining = filter_incomplete(configs);
        assert_eq!(remaining.len(), 1, "reprocess=true should keep all states");
    }
}
