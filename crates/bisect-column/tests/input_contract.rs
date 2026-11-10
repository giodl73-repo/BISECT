use bisect_column::{price_columns, solve_branch_price, BranchPriceConfig, PricingInput};

fn rejected(adjacency: &[Vec<usize>], weights: &[i64], k: usize, tolerance: f64) {
    assert!(price_columns(PricingInput { adjacency, weights, k, tolerance }).is_err());
    assert!(solve_branch_price(adjacency, weights, BranchPriceConfig {
        k, tolerance, formulation_only: false, exact_fixture_limit: 30,
    }).is_err());
}

#[test]
fn invalid_graphs_parameters_and_overflow_are_errors() {
    let edge = vec![vec![1], vec![0]];
    rejected(&[], &[], 1, 0.01);
    rejected(&edge, &[1], 1, 0.01);
    rejected(&edge, &[1, 1], 0, 0.01);
    rejected(&edge, &[1, 1], 3, 0.01);
    rejected(&edge, &[0, 1], 1, 0.01);
    rejected(&edge, &[-1, 1], 1, 0.01);
    rejected(&edge, &[i64::MAX, 1], 1, 0.01);
    for tolerance in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        rejected(&edge, &[1, 1], 1, tolerance);
    }
    for graph in [vec![vec![2], vec![0]], vec![vec![0], vec![]],
                  vec![vec![1, 1], vec![0]], vec![vec![1], vec![]]] {
        rejected(&graph, &[1, 1], 1, 0.01);
    }
}

#[test]
fn unsearched_inputs_cannot_be_reported_infeasible() {
    rejected(&vec![vec![]; 21], &vec![1; 21], 1, 0.01);
}

#[test]
fn largest_valid_i64_population_is_preserved() {
    let result = price_columns(PricingInput {
        adjacency: &[vec![]], weights: &[i64::MAX], k: 1, tolerance: 0.0,
    }).unwrap();
    assert_eq!(result.generated_columns[0].population, i64::MAX);
}
