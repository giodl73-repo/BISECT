use bisect_clustering::assign::{ClusterConfig, ClusterStatus};
use bisect_clustering::fixtures;
use bisect_clustering::metrics::{all_clusters_connected, population_deviation};
use bisect_clustering::output::REGIONALIZATION_SUMMARY_SCHEMA_VERSION;
use bisect_clustering::regionalization::regionalize;

#[test]
fn path_regionalization_merges_adjacent_pairs() {
    let fixture = fixtures::path_6_k2();
    let result = regionalize(
        &fixture.adjacency,
        &fixture.weights,
        ClusterConfig {
            k: 3,
            tolerance: fixture.tolerance,
        },
    )
    .unwrap();

    assert_eq!(result.status, ClusterStatus::Valid);
    assert_eq!(result.assignment, vec![0, 0, 1, 1, 2, 2]);
    assert_eq!(result.merge_log.len(), 3);
    assert!(result
        .merge_log
        .iter()
        .all(|witness| witness.cut_edges_between > 0));
}

#[test]
fn regionalization_detects_infeasible_capacity() {
    let fixture = fixtures::impossible_capacity_k3();
    let result = regionalize(
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
    assert!(result.assignment.is_empty());
}

#[test]
fn regionalization_summary_is_deterministic() {
    let fixture = fixtures::grid_3x3_k3();
    let config = ClusterConfig {
        k: fixture.k,
        tolerance: fixture.tolerance,
    };
    let first = regionalize(&fixture.adjacency, &fixture.weights, config.clone()).unwrap();
    let second = regionalize(&fixture.adjacency, &fixture.weights, config).unwrap();

    assert_eq!(first.assignment, second.assignment);
    assert_eq!(first.merge_log, second.merge_log);
    assert_eq!(first.summary, second.summary);
    assert_eq!(
        first.summary.schema_version,
        REGIONALIZATION_SUMMARY_SCHEMA_VERSION
    );
    assert!(first.summary.parameter_hash.starts_with("sha256:"));
}

#[test]
fn two_clique_regionalization_preserves_contiguity_and_capacity() {
    let fixture = fixtures::two_clique_bridge_k2();
    let result = regionalize(
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
    assert_eq!(
        result.summary.merge_count,
        fixture.weights.len() - fixture.k
    );
}
