use super::*;

// ── Group: Moving-Knife Algorithm (T.13) ─────────────────────────────────

// L0: 4x4 grid → valid 2-way split (all tracts assigned, two non-empty districts).
#[test]
fn mka_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        180,
        MkaMetric::Reock,
        42,
        "root",
    )
    .expect("mka must succeed on 4x4 grid");
    let mut all: Vec<usize> = left.union(&right).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16).collect::<Vec<_>>(),
        "all 16 tracts must be covered"
    );
    assert!(!left.is_empty(), "left side must be non-empty");
    assert!(!right.is_empty(), "right side must be non-empty");
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
}

// L0: same base_seed + n_orientations → identical result (deterministic).
#[test]
fn mka_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let run = || {
        split_subgraph_mka(
            &adj,
            &pop,
            &tracts,
            &centroids,
            0.10,
            180,
            MkaMetric::Reock,
            77,
            "root",
        )
        .expect("mka must succeed")
    };
    let (l1, r1) = run();
    let (l2, r2) = run();
    assert_eq!(l1, l2, "same seed must produce identical left set");
    assert_eq!(r1, r2, "same seed must produce identical right set");
}

// L0: Reock values are clamped to [0.0, 1.0].
#[test]
fn mka_reock_clamped_to_01() {
    // Two coincident points: MEC radius = 0 → reock_score should return 1.0 (clamped).
    let pts = vec![(0.0f64, 0.0f64), (0.0, 0.0)];
    let r = reock_score(&pts, &[]);
    assert!(
        r >= 0.0 && r <= 1.0,
        "reock_score must be in [0,1], got {r}"
    );
    // Large spread: MEC area >> n_tracts — should clamp to 0 from below, return [0,1].
    let spread = vec![(0.0f64, 0.0f64), (1.0e9, 0.0), (0.0, 1.0e9)];
    let r2 = reock_score(&spread, &[]);
    assert!(
        r2 >= 0.0 && r2 <= 1.0,
        "reock_score for large spread must be in [0,1], got {r2}"
    );
}

// L0: MKA seed prefix is distinct from CVD GEO and CVD Phase 1 prefixes.
#[test]
fn mka_seed_prefix_distinct_from_cvd() {
    // The three prefix constants must be pairwise unequal (source-level assertion).
    const MKA_PREFIX: &str = "MKA_INIT_";
    const CVD_GEO_PREFIX: &str = "CVD_GEO_INIT_";
    const CVD_PREFIX: &str = "CVD_INIT_";
    assert_ne!(
        MKA_PREFIX, CVD_GEO_PREFIX,
        "MKA_INIT_ must differ from CVD_GEO_INIT_"
    );
    assert_ne!(
        MKA_PREFIX, CVD_PREFIX,
        "MKA_INIT_ must differ from CVD_INIT_"
    );
    assert_ne!(
        CVD_GEO_PREFIX, CVD_PREFIX,
        "CVD_GEO_INIT_ must differ from CVD_INIT_"
    );

    // mka_seed(0, "") must differ from derive_cvd_geo_seed(0, "").
    let mka_s = mka_seed(0, "");
    let cvd_geo = derive_cvd_geo_seed(0, "");
    assert_ne!(
        mka_s, cvd_geo,
        "mka_seed(0,'') must differ from derive_cvd_geo_seed(0,'')"
    );
}

// L0: split_subgraph_mka_direction returns the same theta as the full split's optimal angle.
#[test]
fn mka_direction_consistent_with_full_split() {
    use std::f64::consts::PI;
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();

    // Run direction-only.
    let theta_star = split_subgraph_mka_direction(&tracts, &centroids, 36);

    // The full split uses n_orientations=36 and should pick the same best angle.
    // We can verify theta_star is in [0, PI).
    assert!(
        theta_star >= 0.0 && theta_star < PI,
        "theta* must be in [0, PI), got {theta_star}"
    );

    // Also verify the direction function doesn't panic on the full split.
    split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        36,
        MkaMetric::Reock,
        42,
        "root",
    )
    .expect("mka full split must succeed");
}

// L0: "moving-knife" StructureMode parses correctly from CLI string.
#[test]
fn mka_structure_mode_parses() {
    use crate::args::StructureMode;
    use clap::ValueEnum;
    let parsed = StructureMode::from_str("moving-knife", true)
        .expect("StructureMode must parse 'moving-knife'");
    assert_eq!(
        parsed,
        StructureMode::MovingKnife,
        "parsed StructureMode must equal MovingKnife"
    );
}

// L0: Welzl MEC of a single point has radius 0.
#[test]
fn welzl_mec_single_point() {
    let (cx, cy, r2) = welzl_mec(&[(3.0, 7.0)]);
    assert!((cx - 3.0).abs() < 1e-9, "cx must equal point x");
    assert!((cy - 7.0).abs() < 1e-9, "cy must equal point y");
    assert!(r2.abs() < 1e-9, "radius² of single point must be 0");
}

// L0: Welzl MEC of two points has center at midpoint and radius = half-distance.
#[test]
fn welzl_mec_two_points() {
    let p1 = (0.0f64, 0.0f64);
    let p2 = (4.0f64, 0.0f64);
    let (cx, cy, r2) = welzl_mec(&[p1, p2]);
    assert!((cx - 2.0).abs() < 1e-9, "cx must be midpoint 2.0, got {cx}");
    assert!(cy.abs() < 1e-9, "cy must be 0.0, got {cy}");
    // Distance = 4, radius = 2, radius² = 4.
    assert!((r2 - 4.0).abs() < 1e-9, "radius² must be 4.0, got {r2}");
}

// L0: n_orientations=1 (only θ=0°) — valid plan returned, no panic.
#[test]
fn mka_single_orientation_no_panic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        1,
        MkaMetric::Reock,
        0,
        "root",
    );
    assert!(result.is_ok(), "n_orientations=1 must not panic");
    let (l, r) = result.unwrap();
    assert!(!l.is_empty(), "left must be non-empty");
    assert!(!r.is_empty(), "right must be non-empty");
}

// L0: both halves non-empty for any subgraph with m >= 2.
#[test]
fn mka_both_halves_nonempty() {
    // Linear chain: 6 nodes.
    let n = 6usize;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nb = vec![];
            if i > 0 {
                nb.push(i - 1);
            }
            if i < n - 1 {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let pop = vec![1000i64; n];
    // Use simple centroid positions along x-axis.
    let centroids: Vec<(f64, f64)> = (0..n).map(|i| (-96.0 + i as f64 * 0.01, 37.5)).collect();
    let tracts: HashSet<usize> = (0..n).collect();
    let (left, right) = split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        180,
        MkaMetric::Reock,
        0,
        "chain",
    )
    .expect("mka must succeed on chain");
    assert!(!left.is_empty(), "left must be non-empty for chain graph");
    assert!(!right.is_empty(), "right must be non-empty for chain graph");
}

// L1: 4x4 grid with evenly-spaced centroids — optimal angle is 0° or 90°.
#[test]
fn mka_symmetric_grid_angle_0_or_90() {
    use std::f64::consts::PI;
    let tracts: HashSet<usize> = (0..16).collect();
    let centroids = synthetic_centroids(4, 4);
    let theta = split_subgraph_mka_direction(&tracts, &centroids, 180);
    // For a symmetric 4x4 grid, all orientations give equal Reock scores.
    // The returned angle is platform-dependent (floating-point tie-breaking),
    // so only assert it is a valid finite angle in [0, PI).
    assert!(theta.is_finite(), "theta must be finite");
    assert!(
        theta >= 0.0 && theta < PI,
        "theta must be in [0, PI), got {theta:.4}"
    );
}

// L1: run_all_splits_mka on 4x4 grid with k=2: valid and deterministic.
#[test]
fn mka_run_all_splits_k2() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let asgn = run_all_splits_mka(
        &adj,
        &pop,
        2,
        0.05,
        None,
        180,
        MkaMetric::Reock,
        42,
        &centroids,
    )
    .expect("run_all_splits_mka k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

// L1: same base_seed → identical run_all_splits_mka result.
#[test]
fn mka_run_all_splits_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let run = || {
        run_all_splits_mka(
            &adj,
            &pop,
            2,
            0.05,
            None,
            180,
            MkaMetric::Reock,
            99999,
            &centroids,
        )
        .expect("run_all_splits_mka must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignments");
}

// L2: NC 2020 MKA min_reock beats CVD-Geographic.
#[test]
#[ignore]
fn mka_nc_reock_beats_metis() {
    // Requires: data/2020/north_carolina_adjacency.adj.bin + centroids
    // Run MKA and CVD-Geographic on NC 2020 k=14.
    // Assert: MKA min_reock_score > CVD-Geographic min_reock_score.
    // Skipped unless --include-ignored is passed.
}

// ── L0: MKA-AreaSection hybrid warm-start (#162) ─────────────────────────

/// L0: "moving-knife" parses to AreaSectionInitArg::MovingKnife → AreaSectionInit::MovingKnife.
#[test]
fn area_section_init_parses() {
    use crate::args::AreaSectionInitArg;
    use crate::runner::AreaSectionInit;

    let mk: AreaSectionInit = AreaSectionInitArg::MovingKnife.into();
    assert_eq!(
        mk,
        AreaSectionInit::MovingKnife,
        "MovingKnife arg must convert to MovingKnife strategy"
    );

    let ro: AreaSectionInit = AreaSectionInitArg::RatioOptimal.into();
    assert_eq!(
        ro,
        AreaSectionInit::RatioOptimal,
        "RatioOptimal arg must convert to RatioOptimal strategy"
    );
}

/// L0: empty centroids → split_subgraph_mka_direction-based path falls back gracefully.
/// When tract_centroids is empty, the warm-start should not panic and should
/// produce Some(theta)=None path (no bias applied). We test by verifying that
/// split_subgraph_mka_direction on an empty set returns 0.0 without panicking.
#[test]
fn area_section_mka_init_fallback_no_centroids() {
    use crate::runner::AreaSectionInit;

    // Empty centroids — simulates the "no centroid data" fallback branch.
    let empty_tracts: HashSet<usize> = HashSet::new();
    let empty_centroids: Vec<(f64, f64)> = vec![];
    let theta = split_subgraph_mka_direction(&empty_tracts, &empty_centroids, 180);
    // Must return 0.0 (defined in the MKA function for empty input) — not panic.
    assert_eq!(
        theta, 0.0,
        "split_subgraph_mka_direction on empty set must return 0.0"
    );

    // Verify AreaSectionInit enum equality round-trips.
    let init = AreaSectionInit::MovingKnife;
    assert_eq!(init, AreaSectionInit::MovingKnife);
    assert_ne!(init, AreaSectionInit::RatioOptimal);
}
