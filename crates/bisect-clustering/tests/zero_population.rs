use bisect_clustering::{capacity_cluster_repaired, regionalize, ClusterConfig, ClusterStatus};

fn path(n: usize) -> Vec<Vec<usize>> {
    (0..n).map(|i| [i.checked_sub(1), (i + 1 < n).then_some(i + 1)].into_iter().flatten().collect()).collect()
}

#[test]
fn zero_population_units_are_preserved_by_both_constructors() {
    let population = [100, 0, 100, 100, 0, 100];
    let adjacency = path(population.len());
    let config = ClusterConfig { k: 2, tolerance: 0.01 };
    let cluster = capacity_cluster_repaired(&adjacency, &population, config.clone()).unwrap();
    let regional = regionalize(&adjacency, &population, config).unwrap();
    for (assignment, status) in [(cluster.assignment, cluster.status), (regional.assignment, regional.status)] {
        assert_eq!(status, ClusterStatus::Valid);
        assert_eq!(assignment.len(), population.len());
        assert!(assignment.iter().all(|&district| district < 2));
        assert!(bisect_clustering::metrics::all_clusters_connected(&adjacency, &assignment, 2));
        assert_eq!(bisect_clustering::metrics::population_deviation(&population, &assignment, 2), 0.0);
    }
}

#[test]
fn invalid_population_and_tolerance_are_rejected_by_both_constructors() {
    for weights in [[0, 0], [-1, 2], [i64::MAX, 1]] {
        let config = ClusterConfig { k: 2, tolerance: 0.01 };
        assert!(capacity_cluster_repaired(&path(2), &weights, config.clone()).is_err());
        assert!(regionalize(&path(2), &weights, config).is_err());
    }
    for tolerance in [f64::NAN, f64::INFINITY, -0.01, 1.0] {
        let config = ClusterConfig { k: 2, tolerance };
        assert!(capacity_cluster_repaired(&path(2), &[100, 100], config.clone()).is_err());
        assert!(regionalize(&path(2), &[100, 100], config).is_err());
    }
}
