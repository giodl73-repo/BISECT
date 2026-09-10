use super::*;

// ── Centroidal Voronoi Districts (CVD) tests ──────────────────────────────

// L0: 4x4 grid, 2 seeds — valid partition covering all 16 tracts.
#[test]
fn cvd_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_cvd(
        &adj, &pop, &tracts, 2,    // k=2 (bisection)
        0.10, // balance_tolerance
        20,   // n_iter
        42,   // base_seed
    )
    .expect("CVD 4x4 must succeed");

    // Completeness
    assert_eq!(left.len() + right.len(), 16, "all 16 tracts covered");
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides must be non-empty"
    );

    // All tracts in 0..16
    let mut all: Vec<usize> = left.iter().chain(right.iter()).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16usize).collect::<Vec<_>>(),
        "must partition exactly 0..16"
    );
}

// L0: same base_seed -> identical result (determinism).
#[test]
fn cvd_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();

    let run = |seed: u64| {
        split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, seed).expect("CVD must succeed")
    };

    let (l1, r1) = run(77);
    let (l2, r2) = run(77);

    let mut s1: Vec<usize> = l1.iter().copied().collect();
    s1.sort_unstable();
    let mut s2: Vec<usize> = l2.iter().copied().collect();
    s2.sort_unstable();
    assert_eq!(s1, s2, "same seed must give identical left set");

    let mut t1: Vec<usize> = r1.iter().copied().collect();
    t1.sort_unstable();
    let mut t2: Vec<usize> = r2.iter().copied().collect();
    t2.sort_unstable();
    assert_eq!(t1, t2, "same seed must give identical right set");
}

// L0: two seeds must be distinct for non-trivial graphs.
#[test]
fn cvd_seeds_are_distinct() {
    // On a 4x4 grid, seed0 = 42 % 16 = 10.
    // seed1 = farthest from 10 → must be != 10.
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    // We test by verifying the partition is non-trivial (not all one side)
    // which implies seeds[0] != seeds[1] since equal seeds → identical Voronoi → all assigned to 0.
    let (_, right) =
        split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, 42).expect("CVD must succeed");
    assert!(
        !right.is_empty(),
        "right must be non-empty (seeds must be distinct)"
    );
}

// L0: n_iter=0 returns the initial Voronoi assignment without crashing.
#[test]
fn cvd_n_iter_zero_no_crash() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 0, 42);
    assert!(
        result.is_ok(),
        "n_iter=0 must not panic: {:?}",
        result.err()
    );
    let (left, right) = result.unwrap();
    assert_eq!(
        left.len() + right.len(),
        16,
        "all tracts covered even with n_iter=0"
    );
}

// L1: CVD on a 4x4 grid with k=2 produces a contiguous split in <= 20 iterations.
#[test]
fn cvd_convergence_within_n_iter() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    // Run with n_iter=20 and verify the result is a valid partition.
    // We can't directly inspect iteration count from the public API (iters_done is internal),
    // so we verify the algorithm terminates and produces a valid result.
    let (left, right) = split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, 99)
        .expect("CVD must succeed within 20 iterations");
    assert_eq!(left.len() + right.len(), 16);
    assert!(left.is_disjoint(&right));
}

// L1: balance check — each side within 10% of half total pop after rebalance.
#[test]
fn cvd_balance_within_tolerance() {
    let (adj, pop) = small_grid(4, 4);
    let total_pop: i64 = pop.iter().sum();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, 42).expect("CVD must succeed");
    let left_pop: i64 = left.iter().map(|&v| pop[v]).sum();
    let right_pop: i64 = right.iter().map(|&v| pop[v]).sum();
    let imbalance_l = (left_pop as f64 - total_pop as f64 / 2.0).abs() / total_pop as f64;
    let imbalance_r = (right_pop as f64 - total_pop as f64 / 2.0).abs() / total_pop as f64;
    assert!(
        imbalance_l <= 0.10,
        "left imbalance {imbalance_l:.3} must be within 10%"
    );
    assert!(
        imbalance_r <= 0.10,
        "right imbalance {imbalance_r:.3} must be within 10%"
    );
}

// L0: CVD prefix constant is "CVD_INIT_" exactly (audit chain invariant).
#[test]
fn cvd_prefix_constant() {
    // SHA-256("CVD_INIT_" || "" || "_" || 0u64le) should be a fixed, known value.
    // We test that derive_cvd_seed(0, "") matches the expected hash.
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"CVD_INIT_");
    h.update(b"");
    h.update(b"_");
    h.update(0u64.to_le_bytes());
    let d = h.finalize();
    let expected = u64::from_le_bytes(d[..8].try_into().unwrap());
    assert_eq!(
        derive_cvd_seed(0, ""),
        expected,
        "derive_cvd_seed(0, '') must equal SHA-256('CVD_INIT__' || 0:le64) first 8 bytes"
    );
    // Verify prefix: the prefix must be exactly "CVD_INIT_"
    let h2: Vec<u8> = {
        let mut h = Sha256::new();
        h.update(b"CVD_INIT_");
        h.update(b"root");
        h.update(b"_");
        h.update(42u64.to_le_bytes());
        h.finalize().to_vec()
    };
    let from_derive = derive_cvd_seed(42, "root");
    let from_expected = u64::from_le_bytes(h2[..8].try_into().unwrap());
    assert_eq!(from_derive, from_expected, "prefix must be CVD_INIT_");
}

// L0: cvd_structure_mode_parses — "centroidal-voronoi" parses as PartitionMode::CentroidalVoronoi.
#[test]
fn cvd_structure_mode_parses() {
    use crate::args::PartitionMode;
    use clap::ValueEnum;
    let parsed = PartitionMode::from_str("centroidal-voronoi", true)
        .expect("PartitionMode must parse 'centroidal-voronoi'");
    assert!(
        matches!(parsed, PartitionMode::CentroidalVoronoi),
        "Expected CentroidalVoronoi, got {:?}",
        parsed
    );
}

// L2 (ignored): CVD on NC — compare mean Polsby-Popper vs single METIS call.
#[test]
#[ignore]
fn cvd_nc_pp_vs_metis() {
    // Requires real NC adjacency data at data/2020/ — L2 only.
    // Expected: CVD PP >= METIS PP (geographic proximity improves compactness).
    panic!("L2 test: requires real NC data — run manually with --ignored");
}

// ── CVD Phase 2 (Geographic) tests ──────────────────────────────────────────

// Helper: build a 4x4 grid with evenly-spaced centroids (1.0 degree apart)
// spanning -100..-97 lon, 37..40 lat (central US, all valid Albers inputs)

// L0: 4x4 grid with synthetic centroids — valid k=2 partition covering all 16 tracts.
#[test]
fn cvd_geographic_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = grid4x4_centroids();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_cvd_geographic(&adj, &pop, &centroids, &tracts, 0.10, 20, 42, "root")
            .expect("CVD geographic 4x4 must succeed");

    assert_eq!(left.len() + right.len(), 16, "all 16 tracts covered");
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides non-empty"
    );

    let mut all: Vec<usize> = left.iter().chain(right.iter()).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16usize).collect::<Vec<_>>(),
        "must partition exactly 0..16"
    );
}

// L0: same seed and node_path -> identical result (determinism).
#[test]
fn cvd_geographic_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = grid4x4_centroids();
    let tracts: HashSet<usize> = (0..16).collect();

    let run = || {
        split_subgraph_cvd_geographic(&adj, &pop, &centroids, &tracts, 0.10, 20, 77, "node01")
            .expect("must succeed")
    };

    let (l1, r1) = run();
    let (l2, r2) = run();

    let mut s1: Vec<usize> = l1.iter().copied().collect();
    s1.sort_unstable();
    let mut s2: Vec<usize> = l2.iter().copied().collect();
    s2.sort_unstable();
    assert_eq!(s1, s2, "same seed must give identical left set");

    let mut t1: Vec<usize> = r1.iter().copied().collect();
    t1.sort_unstable();
    let mut t2: Vec<usize> = r2.iter().copied().collect();
    t2.sort_unstable();
    assert_eq!(t1, t2, "same seed must give identical right set");
}

// L0: Phase 2 seed prefix is distinct from Phase 1 seed prefix.
#[test]
fn cvd_geo_seed_distinct_from_phase1_seed() {
    // SHA-256("CVD_GEO_INIT_"...) != SHA-256("CVD_INIT_"...) for same inputs
    let geo_seed = derive_cvd_geo_seed(42, "root");
    let phase1_seed = derive_cvd_seed(42, "root");
    assert_ne!(
        geo_seed, phase1_seed,
        "CVD_GEO_INIT_ prefix must produce different seeds than CVD_INIT_"
    );
}

// L0: albers_project(-96, 37.5): x near 0 (central meridian), y finite and positive.
#[test]
fn albers_project_central_meridian_near_zero() {
    let (x, y) = albers_project(-96.0, 37.5);
    assert!(
        x.abs() < 1000.0,
        "x at central meridian must be within 1km of 0, got {x}"
    );
    assert!(y.is_finite(), "y must be finite, got {y}");
    assert!(
        y >= 0.0,
        "y must be non-negative (positive northing), got {y}"
    );
}

// L0: empty centroids with Geographic metric returns Err containing "centroid".
#[test]
fn cvd_metric_geographic_missing_centroids_returns_err() {
    let (adj, pop) = small_grid(4, 4);
    let empty_centroids: Vec<(f64, f64)> = vec![];
    let tracts: HashSet<usize> = (0..16).collect();
    let result =
        split_subgraph_cvd_geographic(&adj, &pop, &empty_centroids, &tracts, 0.10, 20, 42, "root");
    assert!(result.is_err(), "empty centroids must return Err");
    let err = result.unwrap_err();
    assert!(
        err.contains("centroid"),
        "error must mention centroid, got: {err}"
    );
}

// L2 (ignored): NC geographic CVD vs graph-distance CVD -- Phase 2 paper comparison.
#[test]
#[ignore]
fn cvd_geographic_nc_pp_vs_graph_distance() {
    // Requires real NC adjacency + centroid data at data/2020/ -- L2 only.
    // Expected: geographic CVD mean PP > graph-distance CVD mean PP (coastal NC tracts).
    panic!("L2 test: requires real NC data with centroids -- run manually with --ignored");
}
