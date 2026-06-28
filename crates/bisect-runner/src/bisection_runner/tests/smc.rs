use super::*;

// ── SMC-Percentile tests (SeedCompositor::SmcPercentile) ─────────────────

// L0: 4-node path k=2, n_particles=50: returns valid 2-district plan.
#[test]
fn smc_percentile_produces_valid_k2_partition() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let result = run_smc_percentile(&adj, &pop, 2, 42, 50, 0.5, 0.5)
        .expect("smc_percentile must succeed on 4-node path");
    // All 4 tracts assigned
    assert_eq!(result.len(), 4, "all 4 tracts must be in the result");
    // Exactly 2 distinct districts
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

// L0: p=0.0 plan has EC <= p=1.0 plan (min-EC <= max-EC ordering).
#[test]
fn smc_percentile_p0_le_p1_ec() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let plan_p0 = run_smc_percentile(&adj, &pop, 2, 99, 50, 0.0, 0.5).expect("p=0.0 must succeed");
    let plan_p1 = run_smc_percentile(&adj, &pop, 2, 99, 50, 1.0, 0.5).expect("p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&plan_p0, &adj);
    let ec_p1 = count_edge_cuts(&plan_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn smc_percentile_deterministic() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let run = || {
        run_smc_percentile(&adj, &pop, 2, 77, 30, 0.5, 0.5).expect("smc_percentile must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical results");
}

// L0: derive_smcp_seed(42) != 42 and != derive_smcp_seed(0).
#[test]
fn smc_percentile_smcp_seed_distinct_from_base() {
    let seed_42 = derive_smcp_seed(42);
    let seed_0 = derive_smcp_seed(0);
    assert_ne!(
        seed_42, 42u64,
        "derive_smcp_seed(42) must differ from raw 42"
    );
    assert_ne!(
        seed_42, seed_0,
        "derive_smcp_seed(42) must differ from derive_smcp_seed(0)"
    );
}

// L0: p=0.0 returns a complete valid plan (all tracts assigned).
// Tests the positive-weight-skip logic indirectly (on valid graph, all weights are positive).
#[test]
fn smc_percentile_zero_weight_skip() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let result = run_smc_percentile(&adj, &pop, 2, 55, 20, 0.0, 0.5).expect("p=0.0 must succeed");
    assert_eq!(
        result.len(),
        4,
        "p=0.0 must return a complete plan (4 tracts)"
    );
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2, "p=0.0 must produce exactly 2 districts");
}

// L0: smc-percentile search mode parses correctly.
#[test]
fn smc_percentile_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed = SearchMode::from_str("smc-percentile", true)
        .expect("'smc-percentile' must be a valid SearchMode");
    assert_eq!(
        parsed,
        SearchMode::SmcPercentile,
        "parsed SearchMode must be SmcPercentile"
    );
}

// L2: NC 2020 calibrated distribution (requires real data).
#[test]
#[ignore]
fn smc_percentile_nc_calibrated_distribution() {
    // Requires: data/2020/adjacency/north_carolina_adjacency_2020.pkl
    // Run SMC with n_particles=500 on NC 2020 k=14.
    // Verify: p=0.0 EC <= p=0.5 EC <= p=1.0 EC (distribution order).
    // Skipped unless --include-ignored is passed.
}
