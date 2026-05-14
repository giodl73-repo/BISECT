use serde::{Deserialize, Serialize};

use crate::master::Column;

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

pub fn price_columns(input: PricingInput<'_>) -> PricingResult {
    let n = input.weights.len();
    let total: i64 = input.weights.iter().sum();
    let ideal = total as f64 / input.k as f64;
    let lower = ideal * (1.0 - input.tolerance);
    let upper = ideal * (1.0 + input.tolerance);
    let mut generated = Vec::new();
    let mut considered = 0usize;
    let mut feasible = 0usize;

    if n == 0 || n > 20 {
        return PricingResult {
            generated_columns: generated,
            considered_subsets: considered,
            feasible_subsets: feasible,
        };
    }

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

    PricingResult {
        generated_columns: generated,
        considered_subsets: considered,
        feasible_subsets: feasible,
    }
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
