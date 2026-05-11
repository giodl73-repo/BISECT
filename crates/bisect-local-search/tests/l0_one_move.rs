use bisect_local_search::metrics::{all_districts_connected, edge_cut, population_deviation};
use bisect_local_search::{
    improve_one_move, ImproveError, ImproveStatus, LocalSearchConfig, LocalSearchMethod,
};

#[test]
fn one_move_improves_edge_cut_while_preserving_validity() {
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

    assert_eq!(result.status, ImproveStatus::Improved);
    assert_eq!(result.moved_vertex, Some(2));
    assert_eq!(result.moved_from, Some(0));
    assert_eq!(result.moved_to, Some(1));
    assert_eq!(edge_cut(&adjacency, &assignment), 3);
    assert_eq!(edge_cut(&adjacency, &result.assignment), 1);
    assert!(population_deviation(&weights, &result.assignment, 2) <= 0.34);
    assert!(all_districts_connected(&adjacency, &result.assignment, 2));
    assert_eq!(result.summary.moves_accepted, 1);
    assert!(result.summary.parameter_hash.starts_with("sha256:"));
}

#[test]
fn one_move_is_deterministic_for_fixed_input() {
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
    let config = LocalSearchConfig::one_move(2, 0.34);

    let first = improve_one_move(&adjacency, &weights, &assignment, config.clone()).unwrap();
    let second = improve_one_move(&adjacency, &weights, &assignment, config).unwrap();

    assert_eq!(first.assignment, second.assignment);
    assert_eq!(first.summary, second.summary);
}

#[test]
fn no_improvement_preserves_assignment() {
    let adjacency = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let weights = vec![100; 4];
    let assignment = vec![0, 0, 1, 1];

    let result = improve_one_move(
        &adjacency,
        &weights,
        &assignment,
        LocalSearchConfig::one_move(2, 0.05),
    )
    .unwrap();

    assert_eq!(result.status, ImproveStatus::NoImprovement);
    assert_eq!(result.assignment, assignment);
    assert_eq!(result.summary.moves_accepted, 0);
}

#[test]
fn staged_tabu_config_returns_structured_error() {
    let adjacency = vec![vec![1], vec![0]];
    let weights = vec![100, 100];
    let assignment = vec![0, 1];

    let err = improve_one_move(
        &adjacency,
        &weights,
        &assignment,
        LocalSearchConfig {
            k: 2,
            tolerance: 0.05,
            method: LocalSearchMethod::Tabu {
                max_iters: 10,
                tenure: 3,
            },
        },
    )
    .unwrap_err();

    assert_eq!(err, ImproveError::StagedMethod("tabu".to_string()));
}
