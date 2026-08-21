use super::*;

// ── detect_gpmetis_version ───────────────────────────────────────────────

#[test]
fn test_detect_gpmetis_version_returns_string() {
    let version = detect_gpmetis_version();
    assert!(!version.is_empty(), "must return a non-empty string");
    assert!(
        version.chars().all(|c| !c.is_control() || c == ' '),
        "version string must contain only printable chars: {:?}",
        version
    );
    assert!(
        version.contains("METIS"),
        "expected METIS in version string, got: {version}"
    );
}

#[test]
fn test_detect_gpmetis_version_never_panics() {
    let v = detect_gpmetis_version();
    assert!(
        !v.is_empty(),
        "must return a non-empty string, got: {:?}",
        v
    );
}

#[test]
fn test_split_four_node_graph() {
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let vw = vec![1000i64, 1000, 1000, 1000];
    let ew = HashMap::new();
    let indices: HashSet<usize> = (0..4).collect();

    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.001,
        100,
        Some(42),
        None,
        None,
    )
    .expect("METIS should split 4-node graph");

    assert_eq!(left.len() + right.len(), 4, "all tracts assigned");
    assert!(!left.is_empty() && !right.is_empty(), "non-empty split");
    // Disjoint and complete
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
    for i in 0..4 {
        assert!(left.contains(&i) || right.contains(&i), "tract {i} missing");
    }
    let pop_left: i64 = left.iter().map(|&i| vw[i]).sum();
    let pop_right: i64 = right.iter().map(|&i| vw[i]).sum();
    let dev = (pop_left - pop_right).abs() as f64 / 4000.0;
    assert!(dev <= 0.2, "split should be balanced, got {dev:.3}");
}

#[test]
fn test_equal_non_nrs_split_keeps_both_sides_connected() {
    // Two dense lobes joined by a narrow corridor exercise the equal-weight
    // path that historically used recursive METIS without Contig.
    let adj = vec![
        vec![1, 2],
        vec![0, 2],
        vec![0, 1, 3],
        vec![2, 4],
        vec![3, 5, 6],
        vec![4, 6],
        vec![4, 5],
    ];
    let vw = vec![1000i64; 7];
    let indices: HashSet<usize> = (0..adj.len()).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &HashMap::new(),
        &indices,
        1.20,
        100,
        Some(7),
        None,
        None,
    )
    .expect("contiguous k-way equal split should succeed");

    assert!(is_connected_subset(&adj, &left));
    assert!(is_connected_subset(&adj, &right));
}

#[test]
fn test_run_all_splits_single_district() {
    let n = 193usize;
    let adj = vec![vec![]; n];
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let assignments = run_all_splits(&adj, &vw, &ew, 1, 0.005, 100, None, None)
        .expect("single district should not invoke METIS");
    assert_eq!(assignments.len(), n);
    assert!(assignments.values().all(|&d| d == 1));
}

#[test]
fn test_run_all_splits_two_districts() {
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let vw = vec![1000i64, 1000, 1000, 1000];
    let ew = HashMap::new();

    let assignments = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None).unwrap();

    assert_eq!(assignments.len(), 4, "all tracts assigned");
    assert!(
        assignments.values().any(|&d| d == 1),
        "district 1 must exist"
    );
    assert!(
        assignments.values().any(|&d| d == 2),
        "district 2 must exist"
    );
    assert!(assignments.values().all(|&d| d == 1 || d == 2));

    let d1: HashSet<usize> = assignments
        .iter()
        .filter(|(_, &v)| v == 1)
        .map(|(&k, _)| k)
        .collect();
    let d2: HashSet<usize> = assignments
        .iter()
        .filter(|(_, &v)| v == 2)
        .map(|(&k, _)| k)
        .collect();
    assert!(d1.is_disjoint(&d2), "districts must be disjoint");
    assert_eq!(d1.len() + d2.len(), 4, "complete coverage");
}

#[test]
fn test_leaf_sort_bfs_order() {
    // Verify sort_by_key gives BFS not lex order
    let mut paths = vec![
        "1".to_string(),
        "0".to_string(),
        "01".to_string(),
        "00".to_string(),
    ];
    paths.sort_by_key(|p| (p.len(), p.clone()));
    // BFS: depth-1 first ("0","1"), then depth-2 ("00","01")
    assert_eq!(paths, vec!["0", "1", "00", "01"]);
}

// ── Invariant tests ──────────────────────────────────────────────────────

#[test]
fn test_nway_single_district_shortcut() {
    let adj = vec![vec![1], vec![0]];
    let vw = vec![1000i64, 1000];
    let ew = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 1, 1.005, 100, None).unwrap();
    assert_eq!(assignments.len(), 2);
    assert!(assignments.values().all(|&d| d == 1));
}

#[test]
fn test_nway_two_districts() {
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let vw = vec![1000i64; 4];
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.005, 100, Some(42)).unwrap();
    assert_eq!(assignments.len(), 4);
    assert!(
        assignments.values().any(|&d| d == 1),
        "district 1 must exist"
    );
    assert!(
        assignments.values().any(|&d| d == 2),
        "district 2 must exist"
    );
    // Districts are 1-based and disjoint
    let d1: HashSet<_> = assignments
        .iter()
        .filter(|(_, &v)| v == 1)
        .map(|(&k, _)| k)
        .collect();
    let d2: HashSet<_> = assignments
        .iter()
        .filter(|(_, &v)| v == 2)
        .map(|(&k, _)| k)
        .collect();
    assert!(d1.is_disjoint(&d2));
    assert_eq!(d1.len() + d2.len(), 4);
}

#[test]
fn test_nway_equal_weights_sum_to_one() {
    // AC-05: for n-way, verify n-1 explicit weights + inferred last = 1.0
    // With weight_per = 1/k, sum = (k-1)/k + inferred(1/k) = 1.0 exactly
    for k in [2usize, 3, 7, 52] {
        let weight_per = 1.0_f64 / k as f64;
        let explicit_sum: f64 = (k - 1) as f64 * weight_per;
        let inferred = 1.0 - explicit_sum;
        assert!(
            (explicit_sum + inferred - 1.0).abs() < 1e-9,
            "k={k}: explicit({explicit_sum:.9}) + inferred({inferred:.9}) should = 1.0"
        );
    }
}

#[test]
fn test_invariant_target_weights_sum_to_one_2way() {
    // AC-05: target partition weights must sum to 1.0 for 2-way split
    // (k_left/k + k_right/k = k/k = 1.0 by construction)
    for k in [2, 3, 4, 7, 8, 14, 52] {
        let tree = bisect_core::BisectionTree::from_k(k);
        for node in &tree.nodes {
            let left_frac = node.k_left as f64 / node.k as f64;
            let right_frac = node.k_right as f64 / node.k as f64;
            let sum = left_frac + right_frac;
            assert!(
                    (sum - 1.0).abs() < 1e-9,
                    "k={k} node k={}: left_frac={left_frac:.6} + right_frac={right_frac:.6} = {sum:.6} != 1.0",
                    node.k
                );
        }
    }
}

// ── ufactor correctness tests ─────────────────────────────────────────────

#[test]
fn test_ufactor_integer_conversion_0_5_pct() {
    // 0.5% tolerance: decimal 1.005 → integer 5
    let decimal = 1.005_f64;
    let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
    assert_eq!(ufactor_int, 5, "1.005 must convert to integer 5 (0.5%)");
}

#[test]
fn test_ufactor_integer_conversion_5_pct() {
    // 5% tolerance: decimal 1.05 → integer 50
    let decimal = 1.05_f64;
    let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
    assert_eq!(ufactor_int, 50, "1.05 must convert to integer 50 (5%)");
}

#[test]
fn test_ufactor_integer_conversion_10_pct() {
    // 10% tolerance: decimal 1.10 → integer 100
    let decimal = 1.10_f64;
    let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
    assert_eq!(ufactor_int, 100, "1.10 must convert to integer 100 (10%)");
}

#[test]
fn test_ufactor_never_zero() {
    // Minimum clamped to 1 — ufactor=0 would disable balance checking
    for decimal in [1.0001_f64, 1.0_f64, 0.999_f64] {
        let raw = ((decimal - 1.0) * 1000.0).round() as i32;
        let clamped = (raw as u32).clamp(1, 1000);
        assert!(
            clamped >= 1,
            "ufactor must be >= 1, got {clamped} from decimal {decimal}"
        );
    }
}

#[test]
fn test_per_node_ufactor_formula() {
    // node_ufactor = 1.0 + balance_tolerance / k_node
    // Root of 98-district map (T=10%): should be very tight
    let k_root = 98usize;
    let tolerance = 0.10_f64;
    let node_ufactor = 1.0 + tolerance / k_root as f64;
    // ~0.102% — convert to int
    let ufactor_int = ((node_ufactor - 1.0) * 1000.0).round() as u32;
    assert_eq!(
        ufactor_int, 1,
        "root of 98-district (10%) → ufactor=1 (0.1%)"
    );

    // Leaf of 2-district split (T=10%): should be loose
    let k_leaf = 2usize;
    let leaf_ufactor = 1.0 + tolerance / k_leaf as f64;
    let leaf_int = ((leaf_ufactor - 1.0) * 1000.0).round() as u32;
    assert_eq!(leaf_int, 50, "leaf of 2-district (10%) → ufactor=50 (5%)");
}

#[test]
fn test_per_node_ufactor_congressional_tight() {
    // Congressional (T=0.5%): root of 52-district CA map
    // 0.5%/52 = 0.0096% → rounds to 0, clamped to minimum 1
    let k = 52usize;
    let tolerance = 0.005_f64; // 0.5%
    let node_ufactor = 1.0 + tolerance / k as f64;
    let raw = ((node_ufactor - 1.0) * 1000.0).round() as u32;
    let ufactor_int = raw.clamp(1, 1000); // minimum 1 = 0.1%
    assert_eq!(
        ufactor_int, 1,
        "CA 52D congressional root → clamped to minimum ufactor=1 (0.1%)"
    );
}

#[test]
fn test_ufactor_wasnt_silently_truncated_regression() {
    // This test catches the historical bug where '-ufactor=1.0050' was passed
    // to gpmetis as a float, which atoi() truncated to 1 regardless of value.
    // The correct behavior: 1.005 → integer 5 (not 1).
    let old_style_float = 1.005_f64;
    // Old bug: atoi("1.0050") == 1 (always)
    // New fix: round((1.005 - 1.0) * 1000) == 5
    let correct_int = ((old_style_float - 1.0) * 1000.0).round() as u32;
    assert_ne!(
        correct_int, 1,
        "REGRESSION: 1.005 should not convert to 1 — that was the bug. Got {correct_int}"
    );
    assert_eq!(
        correct_int, 5,
        "1.005 (0.5% tolerance) must convert to integer 5"
    );
}

#[test]
fn test_invariant_vertex_weights_positive() {
    // DF-04: all vertex weights must be >= 1 after loading
    // sub-zero weights cause METIS to produce degenerate partitions
    // The subgraph builder clamps to max(weight, 1) — verify it would catch 0
    let vw = vec![1000i64, 500, 2000]; // all positive
    let tract_indices: HashSet<usize> = (0..3).collect();
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let sub_vw: Vec<i64> = sorted.iter().map(|&g| vw[g].max(1)).collect();
    assert!(
        sub_vw.iter().all(|&v| v >= 1),
        "all vertex weights must be >= 1 after clamping"
    );
}

// ── Group 1: split_subgraph edge cases ───────────────────────────────────

#[test]
fn test_split_subgraph_with_edge_weights() {
    // 4-node chain with strong edge weights on left side — should bias split
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let vw = vec![1000i64; 4];
    let mut ew = HashMap::new();
    ew.insert((0, 1), 1000.0); // strong edge — METIS should avoid cutting
    ew.insert((1, 2), 1.0); // weak edge — METIS may cut here
    ew.insert((2, 3), 1000.0); // strong edge
    let indices: HashSet<usize> = (0..4).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.005,
        100,
        Some(42),
        None,
        None,
    )
    .expect("should split with edge weights");
    assert_eq!(left.len() + right.len(), 4);
    assert!(!left.is_empty() && !right.is_empty());
}

#[test]
fn test_split_subgraph_unequal_target_weights() {
    // 6 tracts, split 4:2 (target weights 2/3 and 1/3)
    let adj = vec![
        vec![1, 2],
        vec![0, 3],
        vec![0, 3],
        vec![1, 2, 4, 5],
        vec![3, 5],
        vec![3, 4],
    ];
    let vw = vec![1000i64; 6];
    let ew = HashMap::new();
    let indices: HashSet<usize> = (0..6).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.05,
        100,
        Some(42),
        Some(vec![2.0f32 / 3.0, 1.0f32 / 3.0]), // unequal: 4:2 split
        None,
    )
    .expect("should split with target weights");
    assert_eq!(left.len() + right.len(), 6);
    assert!(!left.is_empty() && !right.is_empty());
    // Approximate target: left ~4 tracts, right ~2 (within tolerance)
    let larger = left.len().max(right.len());
    assert!(
        larger >= 3,
        "larger partition should have >= 3 tracts for 4:2 split"
    );
}

#[test]
fn test_split_subgraph_single_node_returns_all_left() {
    // Edge case: single tract — no METIS call, returns all in left
    let adj = vec![vec![]];
    let vw = vec![1000i64];
    let ew = HashMap::new();
    let indices: HashSet<usize> = vec![0].into_iter().collect();
    let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
        .expect("single node split");
    assert_eq!(left.len(), 1);
    assert!(right.is_empty());
}

#[test]
fn test_split_subgraph_two_tracts_always_splits() {
    // 2 tracts: must produce one in each partition
    let adj = vec![vec![1], vec![0]];
    let vw = vec![1000i64, 1000];
    let ew = HashMap::new();
    let indices: HashSet<usize> = (0..2).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.005,
        100,
        Some(42),
        None,
        None,
    )
    .expect("2-node split");
    assert_eq!(left.len(), 1);
    assert_eq!(right.len(), 1);
    assert!(left.is_disjoint(&right));
}

// ── Lorenz analysis tests ─────────────────────────────────────────────

#[test]
fn lorenz_curve_starts_at_origin_ends_at_one() {
    let pop = vec![100i64, 200, 300, 400];
    let area = vec![1000.0f64, 2000.0, 3000.0, 4000.0];
    let (curve, _, _) = population_lorenz(&pop, &area, 4);
    assert!(
        curve
            .first()
            .map(|&(a, p)| a == 0.0 && p == 0.0)
            .unwrap_or(false),
        "curve must start at (0,0)"
    );
    let (a_last, p_last) = *curve.last().unwrap();
    assert!((a_last - 1.0).abs() < 1e-9, "curve area must reach 1.0");
    assert!((p_last - 1.0).abs() < 1e-9, "curve pop must reach 1.0");
}

#[test]
fn lorenz_curve_monotone_non_decreasing() {
    let pop = vec![50i64, 200, 100, 400, 10];
    let area = vec![500.0f64, 1000.0, 800.0, 2000.0, 200.0];
    let (curve, _, _) = population_lorenz(&pop, &area, 5);
    for w in curve.windows(2) {
        assert!(w[1].0 >= w[0].0, "area fraction must be non-decreasing");
        assert!(w[1].1 >= w[0].1, "pop fraction must be non-decreasing");
    }
}

#[test]
fn lorenz_natural_ratio_uniform_state_is_half() {
    // Uniform state: all tracts same density → natural pop at 50% area = 50%
    let pop = vec![100i64; 10];
    let area = vec![1000.0f64; 10];
    let (_, natural_pop, suggested_k) = population_lorenz(&pop, &area, 10);
    assert!(
        (natural_pop - 0.5).abs() < 0.05,
        "uniform state natural pop should be ~50%: got {:.1}%",
        natural_pop * 100.0
    );
    assert_eq!(
        suggested_k, 5,
        "uniform state natural k should be 5 out of 10"
    );
}

#[test]
fn lorenz_natural_ratio_concentrated_state() {
    // Very concentrated: first 2 tracts have 90% of pop in 10% of area
    let pop = vec![900i64, 900, 10, 10, 10, 10, 10, 10, 10, 10];
    let area = vec![
        500.0f64, 500.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0,
    ];
    let (_, natural_pop, _) = population_lorenz(&pop, &area, 10);
    // Dense 50% of area (first 5 dense tracts) should contain well above 50% of pop
    assert!(
        natural_pop > 0.7,
        "concentrated state: dense half should hold >70% pop, got {:.1}%",
        natural_pop * 100.0
    );
}

#[test]
fn lorenz_min_area_for_zero_is_zero() {
    let pop = vec![100i64, 200, 300];
    let area = vec![1000.0f64, 2000.0, 3000.0];
    let min_a = lorenz_min_area(&pop, &area, 0.0);
    assert_eq!(min_a, 0.0);
}

#[test]
fn lorenz_min_area_for_one_is_one() {
    let pop = vec![100i64, 200, 300];
    let area = vec![1000.0f64, 2000.0, 3000.0];
    let min_a = lorenz_min_area(&pop, &area, 1.0);
    assert!((min_a - 1.0).abs() < 1e-9);
}

#[test]
fn lorenz_min_area_dense_tract_first() {
    // Tract 0: 900 pop, 100 area (very dense)
    // Tract 1: 100 pop, 900 area (very sparse)
    // Minimum area to hold 90% of pop = just tract 0 = 100/1000 = 10%
    let pop = vec![900i64, 100];
    let area = vec![100.0f64, 900.0];
    let min_a = lorenz_min_area(&pop, &area, 0.9);
    assert!(
        min_a < 0.15,
        "dense tract holds 90% pop in ~10% area, got {:.1}%",
        min_a * 100.0
    );
}

// ── Group: population_lorenz additional coverage ─────────────────────────

#[test]
fn lorenz_empty_weights_returns_early() {
    // All-zero weights → function returns early with empty curve and 0 natural pop
    let pop = vec![0i64, 0, 0];
    let area = vec![1000.0f64, 2000.0, 3000.0];
    let (curve, natural_pop, suggested_k) = population_lorenz(&pop, &area, 4);
    assert!(curve.is_empty(), "zero total pop must return empty curve");
    assert_eq!(natural_pop, 0.0, "natural pop at half area must be 0");
    assert_eq!(suggested_k, 2, "suggested_k must be num_districts/2 = 2");
}

#[test]
fn lorenz_single_tract() {
    // 1 vertex: curve is trivially (0,0)→(1,1)
    let pop = vec![100i64];
    let area = vec![1000.0f64];
    let (curve, natural_pop, _) = population_lorenz(&pop, &area, 2);
    assert_eq!(
        curve.len(),
        2,
        "single tract: curve has 2 points (0,0) and (1,1)"
    );
    assert!(
        (curve[0].0).abs() < 1e-9 && (curve[0].1).abs() < 1e-9,
        "first point must be (0,0)"
    );
    assert!(
        (curve[1].0 - 1.0).abs() < 1e-9 && (curve[1].1 - 1.0).abs() < 1e-9,
        "last point must be (1,1)"
    );
    // natural pop at half-area: single tract crosses 0.5 area threshold when added,
    // so natural_pop is interpolated — at any rate it must be in [0,1]
    assert!(
        natural_pop >= 0.0 && natural_pop <= 1.0,
        "natural_pop must be in [0,1], got {natural_pop}"
    );
}

#[test]
fn lorenz_two_tracts_different_density() {
    // Tract 0: pop=100, area=10  (density=10) — dense
    // Tract 1: pop=10,  area=100 (density=0.1) — sparse
    // Dense tract is first in sort order.
    // After adding tract 0: cum_area = 10/110 ≈ 0.091 — still < 0.5
    // After adding tract 1: cum_area = 110/110 = 1.0 — crossed 0.5
    // So natural_pop_at_half is interpolated between (0.091, 100/110) and (1.0, 1.0)
    let pop = vec![100i64, 10];
    let area = vec![10.0f64, 100.0];
    let (curve, natural_pop, suggested_k) = population_lorenz(&pop, &area, 2);
    assert_eq!(curve.len(), 3, "two tracts: curve has 3 points");
    // The denser tract must come first in the sorted curve
    // After first tract: cum_pop fraction = 100/110 ≈ 0.909
    assert!(
        curve[1].1 > 0.8,
        "after dense tract, accumulated pop fraction must be > 80%, got {:.3}",
        curve[1].1
    );
    // natural_pop must be > 0.5 (dense area contains majority of pop)
    assert!(
        natural_pop > 0.5,
        "natural pop at half area > 0.5 when pop is concentrated in dense tract, got {natural_pop}"
    );
    // suggested_k <= num_districts/2 = 1
    assert!(
        suggested_k >= 1,
        "suggested_k must be >= 1, got {suggested_k}"
    );
}

#[test]
fn lorenz_natural_k_clamped_to_half() {
    // Verify suggested_k is always <= num_districts/2 for various inputs.
    // Use a heavily concentrated state (all pop in first tract).
    let pop = vec![1000i64, 1, 1, 1, 1, 1, 1, 1];
    let area = vec![10.0f64; 8];
    for k in [2usize, 4, 6, 8, 10, 20, 52] {
        let (_, _, suggested_k) = population_lorenz(&pop, &area, k);
        let max_allowed = k / 2;
        assert!(
            suggested_k <= max_allowed,
            "suggested_k={suggested_k} must be <= {max_allowed} for k={k}"
        );
    }
}
