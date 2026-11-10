use serde::{Deserialize, Serialize};

use crate::master::Column;
use crate::solver::ColumnError;

pub const MAX_PRICING_UNITS: usize = 20;

#[derive(Debug, Clone)]
pub struct PricingInput<'a> {
    pub adjacency: &'a [Vec<usize>],
    pub weights: &'a [i64],
    pub k: usize,
    pub tolerance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricingResult {
    pub generated_columns: Vec<Column>,
    pub considered_subsets: usize,
    pub feasible_subsets: usize,
}

pub fn price_columns(input: PricingInput<'_>) -> Result<PricingResult, ColumnError> {
    validate_pricing_input(&input)?;
    let n = input.weights.len();
    let total: i64 = input.weights.iter().sum();
    let ideal = total as f64 / input.k as f64;
    let lower = ideal * (1.0 - input.tolerance);
    let upper = ideal * (1.0 + input.tolerance);
    let mut generated = Vec::new();
    let mut considered = 0usize;
    let mut feasible = 0usize;

    for mask in 1usize..(1usize << n) {
        considered += 1;
        let units: Vec<usize> = (0..n)
            .filter(|unit| (mask & (1usize << unit)) != 0)
            .collect();
        let population: i64 = units.iter().map(|&unit| input.weights[unit]).sum();
        if (population as f64) < lower || (population as f64) > upper {
            continue;
        }
        if !subset_connected(input.adjacency, &units) {
            continue;
        }
        feasible += 1;
        generated.push(Column {
            id: generated.len(),
            edge_cut: subset_edge_cut(input.adjacency, mask),
            units,
            population,
        });
    }

    Ok(PricingResult {
        generated_columns: generated,
        considered_subsets: considered,
        feasible_subsets: feasible,
    })
}

fn subset_connected(adjacency: &[Vec<usize>], units: &[usize]) -> bool {
    if units.is_empty() {
        return false;
    }
    rgraph_core::node_subset_connected(adjacency, units)
        .expect("validated column-pricing adjacency and subset")
}

fn subset_edge_cut(adjacency: &[Vec<usize>], mask: usize) -> usize {
    rgraph_core::undirected_edge_cut_by(adjacency, |unit| (mask & (1usize << unit)) != 0)
        .expect("validated column-pricing adjacency")
}

/// Validate before enumeration so malformed inputs never panic or masquerade as infeasible.
pub(crate) fn validate_pricing_input(input: &PricingInput<'_>) -> Result<(), ColumnError> {
    let invalid = |message: &str| ColumnError::InvalidInput(message.into());
    let n = input.weights.len();
    if n == 0 || n > MAX_PRICING_UNITS {
        return Err(invalid("complete column pricing supports 1 through 20 units"));
    }
    if input.adjacency.len() != n || input.k == 0 || input.k > n {
        return Err(invalid("adjacency length must match weights and k must be in 1..=unit_count"));
    }
    if !input.tolerance.is_finite() || !(0.0..=1.0).contains(&input.tolerance) {
        return Err(invalid("tolerance must be finite and in 0..=1"));
    }
    let mut total = 0i64;
    for &weight in input.weights {
        if weight <= 0 { return Err(invalid("weights must be positive")); }
        total = total.checked_add(weight).ok_or_else(|| invalid("total population exceeds i64"))?;
    }
    for (unit, neighbors) in input.adjacency.iter().enumerate() {
        let mut seen = std::collections::BTreeSet::new();
        for &neighbor in neighbors {
            if neighbor >= n || neighbor == unit || !seen.insert(neighbor) {
                return Err(invalid("graph contains an out-of-range neighbor, self edge or duplicate edge"));
            }
            if !input.adjacency[neighbor].contains(&unit) {
                return Err(invalid("graph must be symmetric and undirected"));
            }
        }
    }
    Ok(())
}
