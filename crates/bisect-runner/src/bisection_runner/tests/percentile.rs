use super::*;

// ── PercentileSweep tests ─────────────────────────────────────────────────

#[test]
fn percentile_sweep_k1_returns_all_district_1() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let result = run_all_splits_percentile(&adj, &pop, &ew, 1, 0.05, 10, 42, 5, 0.5, None)
        .expect("k=1 must succeed");
    assert!(
        result.values().all(|&d| d == 1),
        "k=1: all tracts in district 1"
    );
}

#[test]
fn percentile_sweep_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let result = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 0.5, None)
        .expect("k=2 must succeed");
    assert_eq!(result.len(), 16);
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

#[test]
fn percentile_sweep_p0_same_as_minimum() {
    // p=0.0 should always return the minimum-EC plan.
    let (adj, pop) = small_grid(5, 4);
    let ew = HashMap::new();
    let min_plan = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 99, 10, 0.0, None)
        .expect("p=0.0 must succeed");
    let ec_min = count_edge_cuts(&min_plan, &adj);
    let max_plan = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 99, 10, 1.0, None)
        .expect("p=1.0 must succeed");
    let ec_max = count_edge_cuts(&max_plan, &adj);
    assert!(
        ec_min <= ec_max,
        "p=0.0 plan must have fewer or equal cuts than p=1.0"
    );
}

#[test]
fn percentile_sweep_deterministic() {
    let (adj, pop) = small_grid(4, 5);
    let ew = HashMap::new();
    let r1 = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 7, 5, 0.5, None).unwrap();
    let r2 = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 7, 5, 0.5, None).unwrap();
    assert_eq!(r1, r2, "same seed must produce identical result");
}
