use bisect_clustering::assign::{capacity_cluster, ClusterConfig, ClusterStatus};
use bisect_clustering::fixtures;
use bisect_clustering::metrics::{all_clusters_connected, population_deviation};
use bisect_clustering::seeds::farthest_point_seeds;

#[test]
fn farthest_seeds_are_deterministic_on_path() {
    let fixture = fixtures::path_6_k2();
    let first = farthest_point_seeds(&fixture.adjacency, fixture.k).unwrap();
    let second = farthest_point_seeds(&fixture.adjacency, fixture.k).unwrap();

    assert_eq!(first, second);
    assert_eq!(first, vec![0, 5]);
}

#[test]
fn bridge_fixture_places_seeds_in_opposite_cliques() {
    let fixture = fixtures::two_clique_bridge_k2();
    let seeds = farthest_point_seeds(&fixture.adjacency, fixture.k).unwrap();

    assert_eq!(seeds.len(), 2);
    assert!(seeds.iter().any(|&seed| seed < 3));
    assert!(seeds.iter().any(|&seed| seed >= 3));
}

#[test]
fn impossible_capacity_returns_structured_status() {
    let fixture = fixtures::impossible_capacity_k3();
    let result = capacity_cluster(
        &fixture.adjacency,
        &fixture.weights,
        ClusterConfig {
            k: fixture.k,
            tolerance: fixture.tolerance,
        },
    )
    .unwrap();

    assert_eq!(result.status, ClusterStatus::InfeasibleCapacity);
    assert_eq!(result.summary.capacity_status, "infeasible-capacity");
}

#[test]
fn grid_summary_is_deterministic_for_fixed_input() {
    let fixture = fixtures::grid_3x3_k3();
    let config = ClusterConfig {
        k: fixture.k,
        tolerance: fixture.tolerance,
    };
    let first = capacity_cluster(&fixture.adjacency, &fixture.weights, config.clone()).unwrap();
    let second = capacity_cluster(&fixture.adjacency, &fixture.weights, config).unwrap();

    assert_eq!(first.assignment, second.assignment);
    assert_eq!(first.summary, second.summary);
    assert!(first.summary.parameter_hash.starts_with("sha256:"));
}

#[test]
fn path_cluster_produces_balanced_connected_plan() {
    let fixture = fixtures::path_6_k2();
    let result = capacity_cluster(
        &fixture.adjacency,
        &fixture.weights,
        ClusterConfig {
            k: fixture.k,
            tolerance: fixture.tolerance,
        },
    )
    .unwrap();

    assert_eq!(result.status, ClusterStatus::Valid);
    assert!(all_clusters_connected(
        &fixture.adjacency,
        &result.assignment,
        fixture.k
    ));
    assert!(
        population_deviation(&fixture.weights, &result.assignment, fixture.k) <= fixture.tolerance
    );
}
