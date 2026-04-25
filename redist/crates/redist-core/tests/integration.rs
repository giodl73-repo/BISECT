/// L1 integration tests: Graph + Partition + VRA working together.
///
/// These test the full data flow through redist-core modules, not individual
/// functions in isolation. A bug that requires two modules to interact would
/// only be caught here, not in L0 unit tests.
use std::collections::HashMap;
use redist_core::{Graph, Partition, build_vra_edge_weights};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

/// A 7-vertex graph representing a simplified Alabama-like topology.
/// Vertices 0-1 are minority tracts (>40%), vertices 2-6 are not.
///
///  [0]--[1]--[2]--[3]
///   |              |
///  [4]--[5]--[6]--+
///
/// Populations sum to 700_000 (ideal per district = 350_000 for k=2).
fn alabama_toy() -> (Graph, Vec<f64>) {
    let adjacency = vec![
        vec![1, 4],     // 0: minority tract, pop 120_000
        vec![0, 2],     // 1: minority tract, pop 130_000
        vec![1, 3],     // 2: non-minority,   pop 90_000
        vec![2, 6],     // 3: non-minority,   pop 80_000
        vec![0, 5],     // 4: non-minority,   pop 100_000
        vec![4, 6],     // 5: non-minority,   pop 90_000
        vec![5, 3],     // 6: non-minority,   pop 90_000
    ];
    let vertex_weights = vec![120_000i64, 130_000, 90_000, 80_000, 100_000, 90_000, 90_000];
    let minority_fracs = vec![0.55, 0.60, 0.25, 0.20, 0.30, 0.20, 0.25];
    let graph = Graph::new(adjacency, vertex_weights).expect("valid graph");
    (graph, minority_fracs)
}

// ---------------------------------------------------------------------------
// Graph + VRA
// ---------------------------------------------------------------------------

#[test]
fn test_vra_weights_only_boost_minority_minority_edges() {
    let (graph, minority_fracs) = alabama_toy();

    // Collect all edges from the graph
    let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
        .flat_map(|(u, nbrs)| nbrs.iter().filter(move |&&v| v > u).map(move |&v| (u, v)))
        .collect();

    let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);

    // Only edge (0,1) connects two minority tracts — that's the only one that should be boosted
    assert!(weights.contains_key(&(0, 1)), "edge (0,1) both minority → must be boosted");
    assert!(!weights.contains_key(&(0, 4)), "edge (0,4): tract 4 not minority → not boosted");
    assert!(!weights.contains_key(&(1, 2)), "edge (1,2): tract 2 not minority → not boosted");

    // Total boosted edges = 1
    assert_eq!(weights.len(), 1);
}

#[test]
fn test_vra_alpha_correct_for_toy_state() {
    let (graph, minority_fracs) = alabama_toy();
    let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
        .flat_map(|(u, nbrs)| nbrs.iter().filter(move |&&v| v > u).map(move |&v| (u, v)))
        .collect();

    let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);

    // f_minority = 2/7 ≈ 0.2857 → α = max(3.0, 10*(1-0.7*0.2857)) = max(3.0, 10*0.800) = 8.0
    let expected_alpha = 3.0_f64.max(10.0 * (1.0 - 0.7 * (2.0 / 7.0)));
    let actual = weights[&(0, 1)];
    assert!((actual - expected_alpha).abs() < 1e-9,
        "alpha={actual:.4}, expected={expected_alpha:.4}");
}

// ---------------------------------------------------------------------------
// Graph + Partition
// ---------------------------------------------------------------------------

#[test]
fn test_partition_balance_on_real_graph() {
    let (graph, _) = alabama_toy();

    // Assign vertices 0,1,4 to district 0 (pop = 350_000) and 2,3,5,6 to district 1 (pop = 350_000)
    let assignments: HashMap<usize, usize> = [
        (0, 0), (1, 0), (4, 0),
        (2, 1), (3, 1), (5, 1), (6, 1),
    ].into_iter().collect();
    let p = Partition::from_assignments(assignments);

    // Perfectly balanced — assert_balanced must pass
    assert!(
        p.assert_balanced(&graph.vertex_weights, 2, 0.005).is_ok(),
        "50/50 split should be within ±0.5%"
    );
    assert!(p.population_balance(&graph.vertex_weights, 2) < 1e-9,
        "perfect split → deviation ~0");
}

#[test]
fn test_partition_imbalance_detected_on_real_graph() {
    let (graph, _) = alabama_toy();

    // All tracts to district 0 except one small one — extreme imbalance
    let assignments: HashMap<usize, usize> = [
        (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0),
        (6, 1),  // only tract 6 (pop=90_000) in district 1
    ].into_iter().collect();
    let p = Partition::from_assignments(assignments);

    let result = p.assert_balanced(&graph.vertex_weights, 2, 0.005);
    assert!(result.is_err(), "610k vs 90k split should fail balance check");
}

// ---------------------------------------------------------------------------
// Full flow: Graph → VRA weights → Partition → balance check
// ---------------------------------------------------------------------------

#[test]
fn test_full_flow_graph_vra_partition_balance() {
    let (graph, minority_fracs) = alabama_toy();

    // Step 1: build VRA edge weights
    let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
        .flat_map(|(u, nbrs)| nbrs.iter().filter(move |&&v| v > u).map(move |&v| (u, v)))
        .collect();
    let vra_weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);

    // Step 2: verify the weights are non-empty and edge (0,1) is boosted
    assert!(!vra_weights.is_empty());
    assert!(vra_weights.contains_key(&(0, 1)));

    // Step 3: create a balanced partition (simulating what METIS would produce)
    let assignments: HashMap<usize, usize> = [
        (0, 0), (1, 0), (4, 0),
        (2, 1), (3, 1), (5, 1), (6, 1),
    ].into_iter().collect();
    let partition = Partition::from_assignments(assignments);

    // Step 4: assert constitutional balance
    partition.assert_balanced(&graph.vertex_weights, 2, 0.005)
        .expect("well-balanced partition should pass ±0.5% check");

    // Step 5: verify district 0 is the higher-minority district.
    // This hand-crafted balanced partition groups the two highest-minority tracts
    // (0: 55%, 1: 60%) into D0 along with tract 4 (30%), giving ~49.7% minority.
    // Achieving >50% requires METIS edge-weighted optimization — not the point of
    // this test. We verify that D0 has higher minority than D1.
    let minority_pops: Vec<f64> = (0..7)
        .map(|i| graph.vertex_weights[i] as f64 * minority_fracs[i])
        .collect();
    let d0_pop: i64 = [0, 1, 4].iter().map(|&i| graph.vertex_weights[i]).sum();
    let d0_minority: f64 = [0, 1, 4].iter().map(|&i| minority_pops[i]).sum();
    let d1_pop: i64 = [2, 3, 5, 6].iter().map(|&i| graph.vertex_weights[i]).sum();
    let d1_minority: f64 = [2, 3, 5, 6].iter().map(|&i| minority_pops[i]).sum();
    let d0_pct = d0_minority / d0_pop as f64;
    let d1_pct = d1_minority / d1_pop as f64;
    assert!(d0_pct > d1_pct,
        "district 0 ({d0_pct:.3}) should have more minority than district 1 ({d1_pct:.3})");
}
