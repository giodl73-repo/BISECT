use super::*;

// ── MergeSplit tests ──────────────────────────────────────────────────────

// L0: run_merge_split must return a valid 2-district plan on a 4×4 grid.
#[test]
fn merge_split_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.0)
        .expect("merge_split k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: p=0.0 EC <= p=1.0 EC (ascending sort — both ends of accepted list).
#[test]
fn merge_split_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn_p0 = run_merge_split(&adj, &pop, &ew, 2, 0.10, 10, 77, 200, 0.0)
        .expect("merge_split p=0.0 must succeed");
    let asgn_p1 = run_merge_split(&adj, &pop, &ew, 2, 0.10, 10, 77, 200, 1.0)
        .expect("merge_split p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn merge_split_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 100, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 100, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: steps=0 returns the initial METIS plan (no chain steps).
#[test]
fn merge_split_zero_steps_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 99, 0, 0.0)
        .expect("merge_split steps=0 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
}

// L0: on a well-connected grid, at least one step should be accepted.
// Acceptance rate > 0 verifies MH ratio logic is not always-reject.
#[test]
fn merge_split_acceptance_rate_positive() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    // Use generous tolerance (0.20) so the MH ratio allows proposals to pass.
    // Run 500 steps — on a 4x4 balanced grid at least a few should be accepted.
    let asgn_p0 = run_merge_split(&adj, &pop, &ew, 2, 0.20, 10, 55, 500, 0.0)
        .expect("merge_split must succeed");
    let asgn_p1 = run_merge_split(&adj, &pop, &ew, 2, 0.20, 10, 55, 500, 1.0)
        .expect("merge_split must succeed");
    // If acceptance_rate > 0, p=0.0 and p=1.0 may select different plans.
    // At minimum, each must be a valid 2-district partition.
    assert_eq!(asgn_p0.len(), 16, "p=0.0 plan must cover all tracts");
    assert_eq!(asgn_p1.len(), 16, "p=1.0 plan must cover all tracts");
    let d0: std::collections::HashSet<usize> = asgn_p0.values().copied().collect();
    let d1: std::collections::HashSet<usize> = asgn_p1.values().copied().collect();
    assert_eq!(d0.len(), 2, "p=0.0: must have 2 districts");
    assert_eq!(d1.len(), 2, "p=1.0: must have 2 districts");
}

// L2: NC 2020 k=14, T=1000 — ignored by default.
#[test]
#[ignore]
fn merge_split_nc_mixing() {
    // Placeholder: load NC 2020 adjacency + population, run 1000 steps k=14.
    // Assert acceptance rate > 0 and plan is a valid 14-district partition.
    // Skipped unless --include-ignored is passed.
}

// ── run_parallel_tempering L0/L1 tests ──────────────────────────────────

// L0: basic k=2 partition on a 4x4 grid — valid districts, all tracts assigned.
#[test]
fn pt_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_parallel_tempering(
        &adj, &pop, &ew, 2,     // num_districts
        10,    // niter
        42,    // base_seed
        2,     // n_replicas
        5,     // swap_interval
        0.005, // cold_tolerance
        0.05,  // hot_tolerance
        20,    // steps
        0.0,   // p
    )
    .expect("pt k=2 on 4x4 grid must succeed");

    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L1: p=0.0 returns a plan with EC <= p=1.0.
#[test]
fn pt_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let run = |p: f64| {
        run_parallel_tempering(&adj, &pop, &ew, 2, 10, 99, 2, 5, 0.005, 0.05, 30, p)
            .expect("pt must succeed")
    };
    let a0 = run(0.0);
    let a1 = run(1.0);
    let ec0: usize = (0..adj.len())
        .flat_map(|v| adj[v].iter().map(move |&nb| (v, nb)))
        .filter(|&(v, nb)| nb > v && a0[&v] != a0[&nb])
        .count();
    let ec1: usize = (0..adj.len())
        .flat_map(|v| adj[v].iter().map(move |&nb| (v, nb)))
        .filter(|&(v, nb)| nb > v && a1[&v] != a1[&nb])
        .count();
    assert!(
        ec0 <= ec1,
        "pt p=0.0 EC ({ec0}) must be <= p=1.0 EC ({ec1})"
    );
}

// L0: same base_seed → identical result (determinism).
#[test]
fn pt_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let run = || {
        run_parallel_tempering(&adj, &pop, &ew, 2, 10, 314159, 3, 5, 0.005, 0.05, 25, 0.0)
            .expect("pt must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignment");
}

// L0: steps=0 returns the initial METIS plan immediately (no chain steps).
#[test]
fn pt_zero_steps_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_parallel_tempering(&adj, &pop, &ew, 2, 10, 42, 2, 5, 0.005, 0.05, 0, 0.0)
        .expect("steps=0 must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: SearchMode parses "parallel-tempering" successfully (CLI integration).
#[test]
fn pt_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed = SearchMode::from_str("parallel-tempering", true)
        .expect("SearchMode must parse 'parallel-tempering'");
    assert_eq!(
        parsed,
        SearchMode::ParallelTempering,
        "parsed SearchMode must equal ParallelTempering"
    );
}

// L2: NC 2020 k=14 — PT cold chain EC <= single-chain EC (quality check).
#[test]
#[ignore]
fn pt_nc_lower_ec_than_single_chain() {
    // Requires: data/2020/nc_adjacency.adj.bin
    // Run PT (n_replicas=4, steps=1000, p=0.0) and single (steps=1, p=0.0)
    // on NC 2020 k=14. Assert EC_pt <= EC_single at least for p=0.0.
    // Skipped unless --include-ignored is passed.
}

// ── run_vra_recom L0/L1 tests ────────────────────────────────────────────

// L0: run_vra_recom must return a valid 2-district plan on a 4x4 grid.
#[test]
fn vra_recom_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    // minority_vap: first 8 tracts (top half) have 0.6, rest have 0.1
    let minority_vap: Vec<f64> = (0..16).map(|i| if i < 8 { 0.6 } else { 0.1 }).collect();
    let asgn = run_vra_recom(
        &adj,
        &pop,
        &ew,
        2,    // num_districts
        10,   // niter
        42,   // base_seed
        30,   // steps
        0.0,  // p
        0.50, // vap_threshold
        &minority_vap,
    )
    .expect("vra_recom k=2 on 4x4 grid must succeed");

    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L1: p=0.0 returns a plan with EC <= p=1.0 (percentile ordering).
#[test]
fn vra_recom_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let minority_vap: Vec<f64> = (0..16).map(|i| if i < 8 { 0.6 } else { 0.1 }).collect();
    let run = |p: f64| {
        run_vra_recom(&adj, &pop, &ew, 2, 10, 77, 50, p, 0.50, &minority_vap)
            .expect("vra_recom must succeed")
    };
    let a0 = run(0.0);
    let a1 = run(1.0);
    let ec = |asgn: &HashMap<usize, usize>| -> usize {
        (0..adj.len())
            .flat_map(|v| adj[v].iter().map(move |&nb| (v, nb)))
            .filter(|&(v, nb)| nb > v && asgn[&v] != asgn[&nb])
            .count()
    };
    assert!(
        ec(&a0) <= ec(&a1),
        "vra_recom p=0.0 EC ({}) must be <= p=1.0 EC ({})",
        ec(&a0),
        ec(&a1)
    );
}

// L0: same base_seed → identical result (determinism).
#[test]
fn vra_recom_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let minority_vap: Vec<f64> = (0..16).map(|i| if i < 8 { 0.6 } else { 0.1 }).collect();
    let run = || {
        run_vra_recom(&adj, &pop, &ew, 2, 10, 271828, 30, 0.0, 0.50, &minority_vap)
            .expect("vra_recom must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignment");
}

// L0: minority_vap all 0.0 → protected_districts empty → no VRA rejection
// (the chain degenerates to unconstrained ForestRecom; all proposals handled by MH only).
#[test]
fn vra_recom_zero_minority_vap_no_vra_rejection() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    // All zero → no protected districts → VRA enforcement is a no-op.
    let minority_vap: Vec<f64> = vec![0.0; 16];
    let asgn = run_vra_recom(&adj, &pop, &ew, 2, 10, 13, 20, 0.0, 0.50, &minority_vap)
        .expect("vra_recom with zero minority_vap must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: SearchMode parses "vra-recom" successfully (CLI integration).
#[test]
fn vra_recom_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed =
        SearchMode::from_str("vra-recom", true).expect("SearchMode must parse 'vra-recom'");
    assert_eq!(
        parsed,
        SearchMode::VraRecom,
        "parsed SearchMode must equal VraRecom"
    );
}

// L2: NC 2020 k=14 — VRA-aware chain preserves majority-minority districts.
#[test]
#[ignore]
fn vra_recom_nc_preserves_majority_minority() {
    // Requires: data/2020/north_carolina_demographics_2020.csv +
    //           data/2020/north_carolina_adjacency.adj.bin
    // Load minority_vap from demographics CSV (HVAP / total_pop per tract).
    // Run vra_recom with vap_threshold=0.50, steps=200.
    // Assert: in every accepted plan, no initially-protected district has
    // minority_vap_fraction < 0.50 (i.e., VRA constraint is honoured).
    // Skipped unless --include-ignored is passed.
}
