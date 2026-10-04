use super::*;

#[test]
fn multi_selects_best_feasible_weighted_cut_and_is_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew: HashMap<_, _> = adj
        .iter()
        .enumerate()
        .flat_map(|(u, neighbors)| {
            neighbors
                .iter()
                .filter(move |&&v| u < v)
                .map(move |&v| ((u, v), if u % 4 == 1 { 100.0 } else { 1.0 }))
        })
        .collect();
    let vertices: HashSet<_> = (0..pop.len()).collect();
    let total: i64 = pop.iter().sum();
    let target = total as f64 / 2.0;
    let mut expected = None;
    for index in 0..4 {
        let (left, right) = split_subgraph(
            &adj,
            &pop,
            1,
            &ew,
            &vertices,
            1.125,
            10,
            Some(42 + index),
            None,
            None,
        )
        .unwrap();
        let left_pop: i64 = left.iter().map(|&v| pop[v]).sum();
        if (left_pop as f64 - target).abs() <= target * 0.125 + 1.0
            && is_connected_subset(&adj, &left)
            && is_connected_subset(&adj, &right)
        {
            let cut = weighted_edge_cut(&ew, &left);
            if expected.as_ref().map_or(true, |(old, _, _)| cut < *old) {
                expected = Some((cut, left, right));
            }
        }
    }
    let (_, left, right) = expected.expect("fixture needs at least one feasible seed");
    let actual = run_all_splits_multi(&adj, &pop, &ew, 2, 0.25, 10, 42, 4, None).unwrap();
    for v in left {
        assert_eq!(actual[&v], 1);
    }
    for v in right {
        assert_eq!(actual[&v], 2);
    }
    assert_eq!(
        actual,
        run_all_splits_multi(&adj, &pop, &ew, 2, 0.25, 10, 42, 4, None).unwrap()
    );
}

#[test]
fn multi_reports_impossible_balance_and_zero_budget() {
    let (adj, mut pop) = small_grid(4, 4);
    pop[0] = 1_000_000;
    assert!(
        run_all_splits_multi(&adj, &pop, &HashMap::new(), 2, 0.05, 10, 42, 4, None)
            .unwrap_err()
            .contains("no balanced contiguous cut")
    );
    assert!(run_all_splits_multi(&adj, &pop, &HashMap::new(), 2, 0.05, 10, 42, 0, None).is_err());
}

#[test]
fn percentile_rejects_invalid_budget_and_propagates_failed_splits() {
    let (adj, pop) = small_grid(4, 4);
    assert!(
        run_all_splits_percentile(&adj, &pop, &HashMap::new(), 2, 0.05, 10, 42, 0, 0.5, None)
            .is_err()
    );
    assert!(run_all_splits_percentile(
        &adj,
        &pop,
        &HashMap::new(),
        2,
        0.05,
        10,
        42,
        2,
        f64::NAN,
        None
    )
    .is_err());
    let disconnected = vec![vec![1], vec![0], vec![3], vec![2]];
    // A rejected graph must not become an invented zero-cut one-district plan.
    assert!(run_all_splits_percentile(
        &disconnected,
        &vec![100; 4],
        &HashMap::new(),
        2,
        0.05,
        10,
        42,
        2,
        0.5,
        None
    )
    .is_err());
}

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
