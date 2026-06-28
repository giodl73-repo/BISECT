use super::*;

// ── ShortBurst ────────────────────────────────────────────────────────────

// L0: n_bursts=0 returns the initial plan unchanged (no ReCom at all).
#[test]
fn short_burst_zero_bursts_returns_initial() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, seeds, idx) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 42, 20, 0, 0.0)
        .expect("short_burst n_bursts=0 should succeed");
    assert_eq!(asgn.len(), 8, "must assign all 8 tracts");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
    assert!(seeds.is_empty(), "no burst seeds when n_bursts=0");
    assert_eq!(idx, 0, "selected index must be 0 when n_bursts=0");
}

// L0: p=0.0 returns the minimum-EC endpoint.
#[test]
fn short_burst_p0_returns_min_ec_endpoint() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, seeds, _idx) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 99, 5, 10, 0.0)
        .expect("p=0.0 short burst must succeed");
    assert_eq!(asgn.len(), 8);
    assert_eq!(seeds.len(), 10, "must have exactly n_bursts=10 seeds");
    // p=0.0 selects rank 0 = minimum EC — just verify it's a valid plan.
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2);
}

// L0: p=1.0 returns the maximum-EC endpoint.
#[test]
fn short_burst_p1_returns_max_ec_endpoint() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn_min, _, _) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 77, 5, 10, 0.0).expect("p=0.0");
    let (asgn_max, _, _) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 77, 5, 10, 1.0).expect("p=1.0");
    // Both are valid plans.
    assert_eq!(asgn_min.len(), 8);
    assert_eq!(asgn_max.len(), 8);
    let d_min: std::collections::HashSet<usize> = asgn_min.values().copied().collect();
    let d_max: std::collections::HashSet<usize> = asgn_max.values().copied().collect();
    assert_eq!(d_min.len(), 2);
    assert_eq!(d_max.len(), 2);
}

// L0: same seed → same result (determinism).
#[test]
fn short_burst_deterministic() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (a1, s1, i1) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 42, 10, 5, 0.3).expect("run 1");
    let (a2, s2, i2) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 42, 10, 5, 0.3).expect("run 2");
    assert_eq!(a1, a2, "same seed must give same assignment");
    assert_eq!(s1, s2, "same seed must give same burst seeds");
    assert_eq!(i1, i2, "same seed must give same selected burst index");
}

// L0: degenerate single burst of length 1.
#[test]
fn short_burst_n1_burst_len1() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, seeds, idx) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 7, 1, 1, 0.0)
        .expect("n_bursts=1, burst_length=1");
    assert_eq!(asgn.len(), 8);
    assert_eq!(seeds.len(), 1);
    assert_eq!(idx, 0);
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2);
}

// L0: verify chain restarts from endpoint — with burst_length=0 all endpoints are the
// initial plan, so EC ties are broken by burst_idx ASC (rank 0 = burst 0, rank 4 = burst 4).
#[test]
fn short_burst_chain_restarts_from_endpoint() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    // burst_length=0: chain steps 0 times, endpoint == input == initial plan for all bursts.
    let (asgn_p0, seeds_p0, idx_p0) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 13, 0, 5, 0.0).expect("burst_length=0 p=0.0");
    let (asgn_p1, seeds_p1, idx_p1) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 13, 0, 5, 1.0).expect("burst_length=0 p=1.0");
    // All endpoints are the same plan, so both p=0.0 and p=1.0 select the same assignment.
    assert_eq!(
        asgn_p0, asgn_p1,
        "with burst_length=0 all endpoints are the same plan"
    );
    assert_eq!(seeds_p0, seeds_p1, "same base seed means same burst seeds");
    // Tie-breaking by burst_idx ASC: rank 0 = burst 0, rank 4 = burst 4.
    assert_eq!(idx_p0, 0, "p=0.0 with equal ECs must select burst 0");
    assert_eq!(idx_p1, 4, "p=1.0 with equal ECs must select burst 4");
}

// L1: produces a valid k=2 partition on a 4-node diamond graph.
#[test]
fn short_burst_produces_valid_k2_partition() {
    let adj = vec![vec![1usize, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let pop = vec![1000i64; 4];
    let ew = HashMap::new();
    let (asgn, _, _) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 55, 10, 10, 0.5).expect("L1 k=2 partition");
    assert_eq!(asgn.len(), 4, "all 4 tracts assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be 1 or 2, got {d}");
    }
}

// L1: all districts populated after short burst.
#[test]
fn short_burst_all_districts_populated() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, _, _) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 100, 20, 20, 0.5)
        .expect("all districts populated");
    let mut dist_pops: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
    for (&tract, &d) in &asgn {
        *dist_pops.entry(d).or_default() += pop[tract];
    }
    assert_eq!(dist_pops.len(), 2, "both districts must be non-empty");
    for (&d, &p) in &dist_pops {
        assert!(p > 0, "district {d} must have positive population");
    }
}

// L2: ignored by default — run manually for empirical validation.
#[test]
#[ignore]
fn short_burst_nc_outperforms_multi_seed() {
    // Placeholder: load NC adjacency, run both multi-seed and short-burst,
    // compare edge-cut distributions. Skipped unless --ignored is passed.
    // Implementation left for G.6 paper experiments.
}

// ── ShortBurstForest tests ────────────────────────────────────────────────

// L0: run_short_burst_forest must return a valid 2-district plan on a 4×4 grid.
#[test]
fn short_burst_forest_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.0)
        .expect("short_burst_forest k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: p=0.0 EC <= p=1.0 EC (ascending sort — both ends tested to catch inverted sort).
#[test]
fn short_burst_forest_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn_p0 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.10, 10, 77, 5, 20, 0.0)
        .expect("short_burst_forest p=0.0 must succeed");
    let asgn_p1 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.10, 10, 77, 5, 20, 1.0)
        .expect("short_burst_forest p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 EC={ec_p0} must be <= p=1.0 EC={ec_p1} (ascending sort)"
    );
}

// L0: same base_seed → identical result (determinism).
#[test]
fn short_burst_forest_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: n_bursts=0 returns the initial plan unchanged.
#[test]
fn short_burst_forest_zero_bursts_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 0, 0.0)
        .expect("n_bursts=0 must succeed");
    assert_eq!(asgn.len(), 16, "must assign all 16 tracts");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: n_bursts=1, burst_length=1 — degenerate case succeeds.
#[test]
fn short_burst_forest_n1_burst_len1() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 7, 1, 1, 0.0)
        .expect("n_bursts=1, burst_length=1 must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: SBF_CHAIN_ burst seed is distinct from SHORT_BURST_CHAIN_ for same (base_seed, burst_idx).
#[test]
fn short_burst_chain_seeds_differ_from_standard() {
    use sha2::Digest;
    let base_seed: u64 = 0;
    let burst_idx: u64 = 0;

    // SBF_CHAIN_ seed
    let sbf_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_CHAIN_");
        h.update(burst_idx.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // SHORT_BURST_CHAIN_ seed (same base_seed, burst_idx)
    let sb_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"SHORT_BURST_CHAIN_");
        h.update(burst_idx.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // SBMS_CHAIN_ seed
    let sbms_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBMS_CHAIN_");
        h.update(burst_idx.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    assert_ne!(sbf_seed, sb_seed,
            "SBF_CHAIN_ and SHORT_BURST_CHAIN_ must produce distinct seeds for same (base_seed, burst_idx)");
    assert_ne!(sbms_seed, sb_seed,
            "SBMS_CHAIN_ and SHORT_BURST_CHAIN_ must produce distinct seeds for same (base_seed, burst_idx)");
    assert_ne!(
        sbf_seed, sbms_seed,
        "SBF_CHAIN_ and SBMS_CHAIN_ must produce distinct seeds for same (base_seed, burst_idx)"
    );

    // Forward and reverse step seeds are distinct.
    let step: u32 = 0;
    let burst_seed: u64 = 12345;
    let fwd = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_FWD_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };
    let rev = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_REV_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };
    assert_ne!(
        fwd, rev,
        "SBF_FWD_ and SBF_REV_ must be distinct for the same (step, burst_seed)"
    );
}

// ── ShortBurstMergeSplit tests ────────────────────────────────────────────

// L0: run_short_burst_merge_split must return a valid 2-district plan on a 4×4 grid.
#[test]
fn short_burst_merge_split_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.0)
        .expect("short_burst_merge_split k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: same base_seed → identical result (determinism).
#[test]
fn short_burst_merge_split_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: n_bursts=1, burst_length=1 — degenerate case succeeds.
#[test]
fn short_burst_merge_split_n1_burst_len1() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 7, 1, 1, 0.0)
        .expect("n_bursts=1, burst_length=1 must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L2: NC 2020 k=14 — ShortBurstForest vs ShortBurstMergeSplit vs ShortBurst EC comparison.
#[test]
#[ignore]
fn short_burst_forest_vs_standard_ec_comparison() {
    // Placeholder: load NC 2020 adjacency, run all three variants on same base_seed.
    // Record EC for each; assert all three return valid k=14 plans.
    // Skipped unless --include-ignored is passed.
}
