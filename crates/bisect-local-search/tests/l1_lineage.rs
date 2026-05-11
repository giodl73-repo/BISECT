use bisect_local_search::{improve_one_move, LocalSearchConfig};

#[test]
fn one_move_summary_builds_algorithm_lineage() {
    let adjacency = vec![
        vec![1, 2],
        vec![0],
        vec![0, 3, 4, 5],
        vec![2, 4],
        vec![2, 3, 5],
        vec![2, 4],
    ];
    let weights = vec![100; 6];
    let assignment = vec![0, 0, 0, 1, 1, 1];
    let result = improve_one_move(
        &adjacency,
        &weights,
        &assignment,
        LocalSearchConfig::one_move(2, 0.34),
    )
    .unwrap();

    let lineage = result
        .summary
        .algorithm_lineage("0.1.0", vec!["sha256:parent".to_string()])
        .unwrap();

    assert_eq!(lineage.producer_crate, "bisect-local-search");
    assert_eq!(lineage.method, "one-move");
    assert_eq!(lineage.parent_plan_hashes, vec!["sha256:parent"]);
    assert_eq!(lineage.extra["status"], "improved");
    assert_eq!(lineage.extra["moves_accepted"], 1);
    assert!(lineage.parameters_hash.starts_with("sha256:"));
}
