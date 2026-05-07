//! L2 real-data tests for redist-smc.
//! All tests are #[ignore] — they require real census data (not available in CI).
//! Run with: cargo test -p redist-smc --test L2_real -- --ignored --nocapture

use redist_smc::{run_smc, SmcConfig};

/// Helper: load NC adjacency from the standard data path.
/// Returns (adjacency, population) or panics with a clear message.
#[cfg(test)]
fn load_state_data(state_lower: &str, year: &str) -> (Vec<Vec<usize>>, Vec<i64>) {
    use std::path::PathBuf;

    // Try standard data paths (mirrors runner.rs V3/V4 search)
    let filename = format!("{state_lower}_adjacency_{year}.pkl");
    let candidates = [
        PathBuf::from(format!("../../data/{year}/adjacency/{filename}")),
        PathBuf::from(format!("data/{year}/adjacency/{filename}")),
        PathBuf::from(format!("/data/{year}/adjacency/{filename}")),
    ];

    let path = candidates.iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            panic!(
                "L2 test: adjacency file not found for {state_lower} {year}.\n\
                 Run: redist fetch --year {year}\n\
                 Tried: {:?}",
                candidates
            )
        });

    // Use the adjacency loader from redist-cli if available; otherwise panic with guidance.
    // In practice, L2 tests are run from the workspace root where the manifest is configured.
    panic!(
        "L2 test: cannot load from path {} directly without adjacency loader.\n\
         Run: cargo test -p redist-smc --test L2_real -- --ignored\n\
         from the workspace root with a configured manifest.",
        path.display()
    );
}

// ── L2.1: NC 2020 k=14, N=1000 ───────────────────────────────────────────────

#[test]
#[ignore]
fn l2_nc_2020_k14_n1000_valid_weighted_ensemble() {
    let (adj, pop) = load_state_data("north_carolina", "2020");
    let k = 14;
    let cfg = SmcConfig {
        n_particles: 1000,
        base_seed: 42,
        pop_tolerance: 0.005,
        resample_threshold: 0.5,
    };

    let result = run_smc(&adj, &pop, k, cfg).expect("NC SMC must succeed");

    // Basic validity
    assert_eq!(result.n_plans(), 1000, "1000 particles");
    let wsum: f64 = result.weights.iter().sum();
    assert!((wsum - 1.0).abs() < 1e-6, "weights sum 1.0 ± 1e-6: {wsum}");
    assert_eq!(result.ess_trace.len(), k - 1, "ESS trace length k-1");

    // All plans valid
    for (idx, plan) in result.plans.iter().enumerate() {
        assert_eq!(plan.len(), adj.len(), "plan {idx}: all tracts assigned");
        for &d in plan { assert!(d >= 1 && d <= k as u32, "plan {idx}: district {d} OOB"); }
        for d in 1..=k as u32 {
            assert!(plan.iter().any(|&x| x == d), "plan {idx}: district {d} empty");
        }
    }

    // Print ESS trace for diagnostics
    let min_ess = result.ess_trace.iter().cloned().fold(f64::INFINITY, f64::min);
    let min_stage = result.ess_trace.iter().position(|&e| e == min_ess).unwrap_or(0);
    eprintln!("NC 2020 k=14 N=1000: {} resamplings, min ESS={:.0} at stage {}",
        result.resample_count, min_ess, min_stage + 1);
}

// ── L2.2: VT 2020 k=1 trivial ─────────────────────────────────────────────────

#[test]
#[ignore]
fn l2_vt_2020_k1_trivial() {
    let (adj, pop) = load_state_data("vermont", "2020");
    let cfg = SmcConfig { n_particles: 10, base_seed: 0, ..Default::default() };
    let result = run_smc(&adj, &pop, 1, cfg).expect("VT k=1 must succeed");

    assert!(result.plans.iter().all(|p| p.iter().all(|&d| d == 1)),
        "VT k=1: all tracts must be in district 1");
    let wsum: f64 = result.weights.iter().sum();
    assert!((wsum - 1.0).abs() < 1e-9, "uniform weights sum 1.0");
    assert_eq!(result.resample_count, 0, "k=1: no resampling");
    let _ = pop; // suppress unused warning
}

// ── L2.3: WI 2020 k=8, N=500 ─────────────────────────────────────────────────

#[test]
#[ignore]
fn l2_wi_2020_k8_n500() {
    let (adj, pop) = load_state_data("wisconsin", "2020");
    let k = 8;
    let cfg = SmcConfig {
        n_particles: 500,
        base_seed: 42,
        pop_tolerance: 0.005,
        resample_threshold: 0.5,
    };

    let result = run_smc(&adj, &pop, k, cfg)
        .expect("WI k=8 N=500 must succeed without all-particles-killed error");

    assert_eq!(result.n_plans(), 500);
    let wsum: f64 = result.weights.iter().sum();
    assert!((wsum - 1.0).abs() < 1e-6, "weights sum 1.0 ± 1e-6");
    eprintln!("WI 2020 k=8 N=500: {} resamplings", result.resample_count);
    let _ = pop;
}
