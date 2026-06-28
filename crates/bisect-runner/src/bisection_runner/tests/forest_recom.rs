use super::*;

// ── ForestRecom tests ─────────────────────────────────────────────────────

// L0: run_forest_recom must return a valid 2-district plan on a 4×4 grid.
#[test]
fn forest_recom_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.0)
        .expect("forest_recom k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: p=0.0 EC <= p=1.0 EC (ascending sort).
#[test]
fn forest_recom_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn_p0 = run_forest_recom(&adj, &pop, &ew, 2, 0.10, 10, 77, 100, 0.0)
        .expect("forest_recom p=0.0 must succeed");
    let asgn_p1 = run_forest_recom(&adj, &pop, &ew, 2, 0.10, 10, 77, 100, 1.0)
        .expect("forest_recom p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn forest_recom_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: steps=0 returns the initial METIS plan (no chain steps).
#[test]
fn forest_recom_zero_steps_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 99, 0, 0.0)
        .expect("forest_recom steps=0 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
}

// L0: no district is empty in the result.
#[test]
fn forest_recom_all_districts_populated() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 55, 100, 0.5)
        .expect("forest_recom must succeed");
    let mut dist_pops: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
    for (&tract, &d) in &asgn {
        *dist_pops.entry(d).or_default() += pop[tract];
    }
    assert_eq!(dist_pops.len(), 2, "both districts must be non-empty");
    for (&d, &p) in &dist_pops {
        assert!(p > 0, "district {d} must have positive population");
    }
}

// L1: k=4 on 4×4 grid — all four districts populated and valid.
#[test]
fn forest_recom_k4_all_districts_populated() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 4, 0.10, 10, 123, 50, 0.0)
        .expect("forest_recom k=4 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let mut dist_pops: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
    for (&tract, &d) in &asgn {
        *dist_pops.entry(d).or_default() += pop[tract];
    }
    assert_eq!(dist_pops.len(), 4, "all 4 districts must be non-empty");
}

// L2: NC 2020 k=14, T=1000 — acceptance rate > 0 (ignored by default).
#[test]
#[ignore]
fn forest_recom_nc_acceptance_rate() {
    // Placeholder: load NC 2020 adjacency + population, run 1000 steps.
    // Assert acceptance_rate > 0. Skipped unless --include-ignored is passed.
}
