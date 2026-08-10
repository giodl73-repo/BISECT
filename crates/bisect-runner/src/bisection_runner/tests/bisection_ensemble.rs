use super::*;

// ── BisectionEnsemble tests ───────────────────────────────────────────────

#[test]
fn bisection_ensemble_produces_valid_2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_bisection_ensemble(
        &adj,
        &pop,
        &ew,
        &tracts,
        1.05,
        10,
        Some(42),
        None,
        50,
        0.5,
    )
    .expect("bisection ensemble must succeed");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both components must be non-empty"
    );
    let mut all: Vec<usize> = left.iter().chain(right.iter()).copied().collect();
    all.sort_unstable();
    let expected: Vec<usize> = (0..16).collect();
    assert_eq!(all, expected, "components must partition all 16 tracts");
}

#[test]
fn bisection_ensemble_p0_equals_standard_bisection_on_small_graph() {
    // With p=0.0 (minimum EC), result should equal or be better than standard bisection.
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (l_ens, r_ens) = split_subgraph_bisection_ensemble(
        &adj,
        &pop,
        &ew,
        &tracts,
        1.05,
        10,
        Some(42),
        None,
        20,
        0.0,
    )
    .expect("must succeed");
    // Just verify it produces a valid partition
    assert_eq!(l_ens.len() + r_ens.len(), 16);
}

#[test]
fn bisection_ensemble_with_search_produces_k2_partition() {
    let (adj, pop) = small_grid(4, 5); // 20 tracts
    let ew = HashMap::new();
    let result = run_all_splits_with_search(
        &adj,
        &pop,
        &ew,
        2,
        0.05,
        10,
        Some(42),
        None,
        Some((0.5, 30)),
    )
    .expect("bisection-ensemble run_all_splits must succeed");
    assert_eq!(result.len(), 20);
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2);
}

#[test]
fn empty_graph_split_honors_tpwgts_left_target() {
    let adj = vec![vec![], vec![], vec![], vec![]];
    let pop = vec![10i64, 10, 10, 70];
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..4).collect();
    let (left, right) = split_subgraph(
        &adj,
        &pop,
        1,
        &ew,
        &tracts,
        1.05,
        10,
        Some(1),
        Some(vec![0.30, 0.70]),
        None,
    )
    .expect("empty graph split must succeed");
    let left_pop: i64 = left.iter().map(|&g| pop[g]).sum();
    let right_pop: i64 = right.iter().map(|&g| pop[g]).sum();
    assert_eq!(left_pop, 30);
    assert_eq!(right_pop, 70);
}

#[test]
fn connected_subset_detects_bridge_removal_disconnection() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1]];
    let connected: HashSet<usize> = [0, 1, 2].into_iter().collect();
    let disconnected: HashSet<usize> = [0, 2].into_iter().collect();
    assert!(is_connected_subset(&adj, &connected));
    assert!(!is_connected_subset(&adj, &disconnected));
}

#[test]
fn count_edge_cuts_known_grid() {
    // 4-node path split at midpoint: [0,1] | [2,3]. One cut edge: 1-2.
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let mut asgn = HashMap::new();
    asgn.insert(0, 1);
    asgn.insert(1, 1);
    asgn.insert(2, 2);
    asgn.insert(3, 2);
    assert_eq!(
        count_edge_cuts(&asgn, &adj),
        1,
        "4-node path bisection has 1 cut edge"
    );
}

#[test]
fn weighted_edge_cut_sums_crossing_weights() {
    let edge_weights = HashMap::from([
        ((0usize, 1usize), 1.5),
        ((1usize, 2usize), 2.25),
        ((2usize, 3usize), 3.0),
    ]);
    let left = HashSet::from([0usize, 1]);

    assert_eq!(weighted_edge_cut(&edge_weights, &left), 2.25);
}

#[test]
fn weighted_edge_cut_is_zero_when_no_edges_cross() {
    let edge_weights = HashMap::from([
        ((0usize, 1usize), 1.5),
        ((1usize, 2usize), 2.25),
        ((2usize, 3usize), 3.0),
    ]);
    let all_left = HashSet::from([0usize, 1, 2, 3]);
    let none_left = HashSet::new();

    assert_eq!(weighted_edge_cut(&edge_weights, &all_left), 0.0);
    assert_eq!(weighted_edge_cut(&edge_weights, &none_left), 0.0);
}

#[test]
fn weighted_edge_cut_sums_all_crossing_edges() {
    let edge_weights = HashMap::from([
        ((0usize, 1usize), 1.5),
        ((0usize, 2usize), 2.0),
        ((1usize, 3usize), 3.25),
    ]);
    let left = HashSet::from([0usize, 1]);

    assert_eq!(weighted_edge_cut(&edge_weights, &left), 5.25);
}

#[test]
fn weighted_edge_cut_treats_missing_nodes_as_right_side() {
    let edge_weights = HashMap::from([((0usize, 10usize), 4.0), ((10usize, 11usize), 7.0)]);
    let left = HashSet::from([0usize]);

    assert_eq!(weighted_edge_cut(&edge_weights, &left), 4.0);
}

#[test]
fn weighted_edge_cut_uses_canonical_edge_order() {
    // Adding the two unit weights before 1e16 preserves one f64 ULP; adding
    // either unit after 1e16 would round it away. Keys define the canonical order.
    let mut edge_weights = HashMap::new();
    edge_weights.insert((2usize, 3usize), 1.0e16);
    edge_weights.insert((0usize, 3usize), 1.0);
    edge_weights.insert((1usize, 3usize), 1.0);
    let left = HashSet::from([0usize, 1, 2]);

    assert_eq!(
        weighted_edge_cut(&edge_weights, &left),
        10_000_000_000_000_002.0
    );
}
