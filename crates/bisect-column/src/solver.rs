use thiserror::Error;

use crate::master::{Column, MasterProblem, MasterSolution};
use crate::output::{BranchPriceReport, BranchPriceStatus};
use crate::pricing::{price_columns, PricingInput};

#[derive(Debug, Clone, Copy)]
pub struct BranchPriceConfig {
    pub k: usize,
    pub tolerance: f64,
    pub formulation_only: bool,
    pub exact_fixture_limit: usize,
}

#[derive(Debug, Error, PartialEq)]
pub enum ColumnError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub fn solve_branch_price(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: BranchPriceConfig,
) -> Result<BranchPriceReport, ColumnError> {
    validate(adjacency, weights, config)?;
    let pricing = price_columns(PricingInput {
        adjacency,
        weights,
        k: config.k,
        tolerance: config.tolerance,
    });
    let master = MasterProblem::new(weights.len(), config.k, pricing.generated_columns);
    if config.formulation_only || weights.len() > config.exact_fixture_limit {
        return Ok(BranchPriceReport::new(
            BranchPriceStatus::FormulationOnly,
            1,
            master.columns.len(),
            master.formulation_size(),
            None,
        ));
    }

    let solution = solve_master_exact(&master);
    let status = if solution.is_some() {
        BranchPriceStatus::ExactFixtureOptimal
    } else {
        BranchPriceStatus::Infeasible
    };
    Ok(BranchPriceReport::new(
        status,
        1,
        master.columns.len(),
        master.formulation_size(),
        solution,
    ))
}

fn solve_master_exact(master: &MasterProblem) -> Option<MasterSolution> {
    let mut best: Option<MasterSolution> = None;
    let mut selected = Vec::new();
    let mut covered = vec![false; master.unit_count];
    search_columns(master, 0, &mut selected, &mut covered, &mut best);
    best
}

fn search_columns(
    master: &MasterProblem,
    start: usize,
    selected: &mut Vec<Column>,
    covered: &mut [bool],
    best: &mut Option<MasterSolution>,
) {
    if selected.len() == master.k {
        if covered.iter().all(|&is_covered| is_covered) {
            let boundary_sum: usize = selected.iter().map(|column| column.edge_cut).sum();
            let objective = boundary_sum / 2;
            let selected_column_ids = selected.iter().map(|column| column.id).collect();
            let assignment = assignment_from_columns(master.unit_count, selected);
            let candidate = MasterSolution {
                selected_column_ids,
                assignment,
                objective,
            };
            if best.as_ref().map_or(true, |existing| {
                solution_key(&candidate) < solution_key(existing)
            }) {
                *best = Some(candidate);
            }
        }
        return;
    }
    if start >= master.columns.len() {
        return;
    }

    for idx in start..master.columns.len() {
        let column = &master.columns[idx];
        if column.units.iter().any(|&unit| covered[unit]) {
            continue;
        }
        for &unit in &column.units {
            covered[unit] = true;
        }
        selected.push(column.clone());
        search_columns(master, idx + 1, selected, covered, best);
        selected.pop();
        for &unit in &column.units {
            covered[unit] = false;
        }
    }
}

fn solution_key(solution: &MasterSolution) -> (usize, Vec<usize>) {
    (solution.objective, solution.selected_column_ids.clone())
}

fn assignment_from_columns(unit_count: usize, selected: &[Column]) -> Vec<usize> {
    let mut assignment = vec![usize::MAX; unit_count];
    for (district, column) in selected.iter().enumerate() {
        for &unit in &column.units {
            assignment[unit] = district;
        }
    }
    assignment
}

fn validate(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: BranchPriceConfig,
) -> Result<(), ColumnError> {
    if config.k == 0 {
        return Err(ColumnError::InvalidInput(
            "k must be greater than zero".to_string(),
        ));
    }
    if adjacency.len() != weights.len() {
        return Err(ColumnError::InvalidInput(format!(
            "adjacency has {} units but weights has {}",
            adjacency.len(),
            weights.len()
        )));
    }
    if config.k > weights.len() {
        return Err(ColumnError::InvalidInput(format!(
            "k={} exceeds unit count {}",
            config.k,
            weights.len()
        )));
    }
    if weights.iter().any(|&weight| weight <= 0) {
        return Err(ColumnError::InvalidInput(
            "weights must be positive".to_string(),
        ));
    }
    Ok(())
}
