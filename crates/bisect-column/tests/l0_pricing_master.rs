use bisect_column::{solve_branch_price, BranchPriceConfig, BranchPriceStatus, PricingInput};

fn path_adj(n: usize) -> Vec<Vec<usize>> {
    (0..n)
        .map(|idx| {
            let mut neighbors = Vec::new();
            if idx > 0 {
                neighbors.push(idx - 1);
            }
            if idx + 1 < n {
                neighbors.push(idx + 1);
            }
            neighbors
        })
        .collect()
}

#[test]
fn pricing_generates_connected_balanced_columns_on_path() {
    let adjacency = path_adj(4);
    let weights = vec![100, 100, 100, 100];
    let pricing = bisect_column::price_columns(PricingInput {
        adjacency: &adjacency,
        weights: &weights,
        k: 2,
        tolerance: 0.01,
    });

    let units: Vec<Vec<usize>> = pricing
        .generated_columns
        .iter()
        .map(|column| column.units.clone())
        .collect();
    assert_eq!(units, vec![vec![0, 1], vec![1, 2], vec![2, 3]]);
    assert_eq!(pricing.feasible_subsets, 3);
}

#[test]
fn formulation_only_report_has_no_bounds() {
    let adjacency = path_adj(4);
    let report = solve_branch_price(
        &adjacency,
        &[100, 100, 100, 100],
        BranchPriceConfig {
            k: 2,
            tolerance: 0.01,
            formulation_only: true,
            exact_fixture_limit: 8,
        },
    )
    .unwrap();

    assert_eq!(report.status, BranchPriceStatus::FormulationOnly);
    assert!(report.solution.is_none());
    assert_eq!(report.lower_bound, None);
    assert_eq!(report.upper_bound, None);
    assert!(report.parameter_hash.starts_with("sha256:"));
}

#[test]
fn small_exact_master_solves_path_fixture() {
    let adjacency = path_adj(4);
    let report = solve_branch_price(
        &adjacency,
        &[100, 100, 100, 100],
        BranchPriceConfig {
            k: 2,
            tolerance: 0.01,
            formulation_only: false,
            exact_fixture_limit: 8,
        },
    )
    .unwrap();

    assert_eq!(report.status, BranchPriceStatus::ExactFixtureOptimal);
    let solution = report.solution.unwrap();
    assert_eq!(solution.assignment, vec![0, 0, 1, 1]);
    assert_eq!(solution.objective, 1);
    assert_eq!(report.lower_bound, Some(1.0));
    assert_eq!(report.upper_bound, Some(1.0));
    assert_eq!(report.gap, Some(0.0));
}

#[test]
fn report_builds_algorithm_lineage() {
    let adjacency = path_adj(4);
    let report = solve_branch_price(
        &adjacency,
        &[100, 100, 100, 100],
        BranchPriceConfig {
            k: 2,
            tolerance: 0.01,
            formulation_only: false,
            exact_fixture_limit: 8,
        },
    )
    .unwrap();

    let lineage = report.algorithm_lineage("0.1.0", Vec::new()).unwrap();
    assert_eq!(lineage.producer_crate, "bisect-column");
    assert_eq!(lineage.method, "branch-and-price");
    assert_eq!(lineage.extra["status"], "exact-fixture-optimal");
    assert!(lineage.parameters_hash.starts_with("sha256:"));
}
