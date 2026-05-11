use bisect_clustering::assign::{capacity_cluster, ClusterConfig, ClusterStatus};
use bisect_clustering::fixtures;
use bisect_clustering::CLUSTER_SUMMARY_SCHEMA_VERSION;

#[test]
fn grid_3x3_k3_produces_complete_stage1_summary() {
    let fixture = fixtures::grid_3x3_k3();
    let result = capacity_cluster(
        &fixture.adjacency,
        &fixture.weights,
        ClusterConfig {
            k: fixture.k,
            tolerance: fixture.tolerance,
        },
    )
    .unwrap();

    assert_eq!(result.assignment.len(), fixture.weights.len());
    assert_eq!(result.seeds.len(), fixture.k);
    assert_ne!(result.status, ClusterStatus::InfeasibleCapacity);
    assert_eq!(
        result.summary.schema_version,
        CLUSTER_SUMMARY_SCHEMA_VERSION
    );
    assert_eq!(result.summary.method, "capacity-clustering");
    assert_eq!(result.summary.seed_method, "farthest");
    assert_eq!(result.summary.repair_method, "none");
    assert!(result.summary.population_deviation.is_finite());
    assert!(result.summary.parameter_hash.starts_with("sha256:"));
}
