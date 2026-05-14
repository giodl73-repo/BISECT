use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, VecDeque};
use thiserror::Error;

pub const FLOW_SUMMARY_SCHEMA_VERSION: &str = "bisect-flow-summary-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlowSeedMethod {
    Farthest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlowCostMethod {
    EdgeCut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlowRepairMethod {
    None,
    Bfs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlowStatus {
    Valid,
    NeedsRepair,
    InfeasibleCapacity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowConfig {
    pub k: usize,
    pub tolerance: f64,
    pub seed_method: FlowSeedMethod,
    pub cost_method: FlowCostMethod,
    pub repair_method: FlowRepairMethod,
}

impl FlowConfig {
    pub fn new(k: usize, tolerance: f64) -> Self {
        Self {
            k,
            tolerance,
            seed_method: FlowSeedMethod::Farthest,
            cost_method: FlowCostMethod::EdgeCut,
            repair_method: FlowRepairMethod::Bfs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowInfeasibilityWitness {
    pub reason: String,
    pub ideal_population: i64,
    pub lower_population: i64,
    pub upper_population: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowSummary {
    pub schema_version: String,
    pub method: String,
    pub seed_method: FlowSeedMethod,
    pub cost_method: FlowCostMethod,
    pub repair_method: FlowRepairMethod,
    pub status: FlowStatus,
    pub population_deviation: f64,
    pub edge_cut: usize,
    pub seeds: Vec<usize>,
    pub infeasibility_witness: Option<FlowInfeasibilityWitness>,
    pub parameter_hash: String,
}

impl FlowSummary {
    fn new(
        config: &FlowConfig,
        status: FlowStatus,
        population_deviation: f64,
        edge_cut: usize,
        seeds: Vec<usize>,
        infeasibility_witness: Option<FlowInfeasibilityWitness>,
    ) -> Self {
        let mut summary = Self {
            schema_version: FLOW_SUMMARY_SCHEMA_VERSION.to_string(),
            method: "flow-construction".to_string(),
            seed_method: config.seed_method,
            cost_method: config.cost_method,
            repair_method: config.repair_method,
            status,
            population_deviation,
            edge_cut,
            seeds,
            infeasibility_witness,
            parameter_hash: String::new(),
        };
        summary.parameter_hash = summary.compute_parameter_hash();
        summary
    }

    fn compute_parameter_hash(&self) -> String {
        let payload = serde_json::json!({
            "schema_version": self.schema_version,
            "method": self.method,
            "seed_method": self.seed_method,
            "cost_method": self.cost_method,
            "repair_method": self.repair_method,
            "status": self.status,
            "population_deviation": self.population_deviation,
            "edge_cut": self.edge_cut,
            "seeds": self.seeds,
            "infeasibility_witness": self.infeasibility_witness,
        });
        let bytes = serde_json::to_vec(&payload).expect("flow summary payload serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    pub fn algorithm_lineage(
        &self,
        producer_version: impl Into<String>,
        parent_plan_hashes: Vec<String>,
    ) -> Result<rplan_audit::AlgorithmLineage, rplan_audit::AuditError> {
        rplan_audit::AlgorithmLineage::new(
            "bisect-flow",
            producer_version,
            self.method.clone(),
            parent_plan_hashes,
            self.algorithm_lineage_extra(),
        )
    }

    pub fn algorithm_lineage_extra(&self) -> serde_json::Value {
        serde_json::json!({
            "lineage_schema_version": self.schema_version,
            "method": self.method,
            "seed_method": self.seed_method,
            "cost_method": self.cost_method,
            "repair_method": self.repair_method,
            "status": self.status,
            "population_deviation": self.population_deviation,
            "edge_cut": self.edge_cut,
            "seeds": self.seeds,
            "infeasibility_witness": self.infeasibility_witness,
            "parameter_hash": self.parameter_hash,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowResult {
    pub assignment: Vec<usize>,
    pub seeds: Vec<usize>,
    pub status: FlowStatus,
    pub summary: FlowSummary,
}

#[derive(Debug, Error, PartialEq)]
pub enum FlowError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub fn construct_flow(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: FlowConfig,
) -> Result<FlowResult, FlowError> {
    validate_inputs(adjacency, weights, &config)?;
    let (lower, ideal, upper) = capacity_bounds(weights, config.k, config.tolerance);
    if !capacity_feasible(weights, config.k, config.tolerance) {
        let witness = FlowInfeasibilityWitness {
            reason: "no balanced capacity assignment exists for this fixture".to_string(),
            ideal_population: ideal.round() as i64,
            lower_population: lower.floor() as i64,
            upper_population: upper.ceil() as i64,
        };
        let summary = FlowSummary::new(
            &config,
            FlowStatus::InfeasibleCapacity,
            f64::INFINITY,
            0,
            Vec::new(),
            Some(witness),
        );
        return Ok(FlowResult {
            assignment: Vec::new(),
            seeds: Vec::new(),
            status: FlowStatus::InfeasibleCapacity,
            summary,
        });
    }

    let seeds = farthest_point_seeds(adjacency, config.k)?;
    let mut assignment = balanced_frontier_assignment(adjacency, weights, &seeds, upper);
    if assignment.iter().any(|&district| district == usize::MAX) {
        fill_unassigned_by_nearest_seed(adjacency, &seeds, &mut assignment);
    }
    let mut deviation = population_deviation(weights, &assignment, config.k);
    let mut connected = all_districts_connected(adjacency, &assignment, config.k);
    let mut status = if deviation <= config.tolerance && connected {
        FlowStatus::Valid
    } else {
        FlowStatus::NeedsRepair
    };

    if status == FlowStatus::NeedsRepair && config.repair_method == FlowRepairMethod::Bfs {
        if let Some(repaired) = exhaustive_valid_small(adjacency, weights, config.k, lower, upper) {
            assignment = repaired;
            deviation = population_deviation(weights, &assignment, config.k);
            connected = all_districts_connected(adjacency, &assignment, config.k);
            if deviation <= config.tolerance && connected {
                status = FlowStatus::Valid;
            }
        }
    }

    let cut = edge_cut(adjacency, &assignment);
    let summary = FlowSummary::new(&config, status, deviation, cut, seeds.clone(), None);
    Ok(FlowResult {
        assignment,
        seeds,
        status,
        summary,
    })
}

fn balanced_frontier_assignment(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    seeds: &[usize],
    upper: f64,
) -> Vec<usize> {
    let mut assignment = vec![usize::MAX; weights.len()];
    let mut district_pop = vec![0i64; seeds.len()];
    let mut frontier: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); seeds.len()];

    for (district, &seed) in seeds.iter().enumerate() {
        assignment[seed] = district;
        district_pop[district] += weights[seed];
        for &neighbor in &adjacency[seed] {
            if neighbor < assignment.len() && assignment[neighbor] == usize::MAX {
                frontier[district].insert(neighbor);
            }
        }
    }

    while assignment.iter().any(|&district| district == usize::MAX) {
        let Some((district, unit)) = choose_frontier_unit(&frontier, &district_pop, weights, upper)
        else {
            break;
        };
        frontier[district].remove(&unit);
        if assignment[unit] != usize::MAX {
            continue;
        }
        assignment[unit] = district;
        district_pop[district] += weights[unit];
        for &neighbor in &adjacency[unit] {
            if neighbor < assignment.len() && assignment[neighbor] == usize::MAX {
                frontier[district].insert(neighbor);
            }
        }
    }
    assignment
}

fn choose_frontier_unit(
    frontier: &[BTreeSet<usize>],
    district_pop: &[i64],
    weights: &[i64],
    upper: f64,
) -> Option<(usize, usize)> {
    frontier
        .iter()
        .enumerate()
        .filter_map(|(district, units)| {
            units
                .iter()
                .copied()
                .find(|&unit| district_pop[district] + weights[unit] <= upper.ceil() as i64)
                .map(|unit| (district_pop[district], unit, district))
        })
        .min()
        .map(|(_, unit, district)| (district, unit))
}

fn fill_unassigned_by_nearest_seed(
    adjacency: &[Vec<usize>],
    seeds: &[usize],
    assignment: &mut [usize],
) {
    let distances: Vec<Vec<Option<usize>>> = seeds
        .iter()
        .map(|&seed| distances_from(adjacency, seed))
        .collect();
    for unit in 0..assignment.len() {
        if assignment[unit] != usize::MAX {
            continue;
        }
        assignment[unit] = distances
            .iter()
            .enumerate()
            .filter_map(|(district, distances)| {
                distances[unit].map(|distance| (distance, district))
            })
            .min()
            .map(|(_, district)| district)
            .unwrap_or(0);
    }
}

fn farthest_point_seeds(adjacency: &[Vec<usize>], k: usize) -> Result<Vec<usize>, FlowError> {
    let mut seeds = vec![0usize];
    while seeds.len() < k {
        let distance_sets: Vec<Vec<Option<usize>>> = seeds
            .iter()
            .map(|&seed| distances_from(adjacency, seed))
            .collect();
        let next = (0..adjacency.len())
            .filter(|idx| !seeds.contains(idx))
            .map(|idx| {
                let min_distance = distance_sets
                    .iter()
                    .filter_map(|distances| distances[idx])
                    .min()
                    .unwrap_or(usize::MAX);
                (min_distance, idx)
            })
            .max()
            .map(|(_, idx)| idx)
            .ok_or_else(|| {
                FlowError::InvalidInput("not enough distinct units for seeds".to_string())
            })?;
        seeds.push(next);
    }
    seeds.sort_unstable();
    Ok(seeds)
}

fn distances_from(adjacency: &[Vec<usize>], start: usize) -> Vec<Option<usize>> {
    let mut distances = vec![None; adjacency.len()];
    let mut queue = VecDeque::from([start]);
    distances[start] = Some(0);
    while let Some(node) = queue.pop_front() {
        let next_distance = distances[node].unwrap() + 1;
        for &neighbor in &adjacency[node] {
            if neighbor < adjacency.len() && distances[neighbor].is_none() {
                distances[neighbor] = Some(next_distance);
                queue.push_back(neighbor);
            }
        }
    }
    distances
}

fn exhaustive_valid_small(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    k: usize,
    lower: f64,
    upper: f64,
) -> Option<Vec<usize>> {
    if weights.len() > 12 {
        return None;
    }
    let mut assignment = vec![usize::MAX; weights.len()];
    let mut district_pop = vec![0i64; k];
    search_assignment(
        adjacency,
        weights,
        k,
        lower,
        upper,
        0,
        &mut assignment,
        &mut district_pop,
    )
}

#[allow(clippy::too_many_arguments)]
fn search_assignment(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    k: usize,
    lower: f64,
    upper: f64,
    unit: usize,
    assignment: &mut [usize],
    district_pop: &mut [i64],
) -> Option<Vec<usize>> {
    if unit == weights.len() {
        if district_pop
            .iter()
            .all(|&pop| pop as f64 >= lower && pop as f64 <= upper)
            && all_districts_connected(adjacency, assignment, k)
        {
            return Some(assignment.to_vec());
        }
        return None;
    }

    for district in 0..k {
        if district_pop[district] + weights[unit] > upper.ceil() as i64 {
            continue;
        }
        assignment[unit] = district;
        district_pop[district] += weights[unit];
        if let Some(valid) = search_assignment(
            adjacency,
            weights,
            k,
            lower,
            upper,
            unit + 1,
            assignment,
            district_pop,
        ) {
            return Some(valid);
        }
        district_pop[district] -= weights[unit];
        assignment[unit] = usize::MAX;
        if district_pop[district] == 0 {
            break;
        }
    }
    None
}

fn validate_inputs(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: &FlowConfig,
) -> Result<(), FlowError> {
    if config.k == 0 {
        return Err(FlowError::InvalidInput(
            "k must be greater than zero".to_string(),
        ));
    }
    if adjacency.len() != weights.len() {
        return Err(FlowError::InvalidInput(format!(
            "adjacency has {} units but weights has {}",
            adjacency.len(),
            weights.len()
        )));
    }
    if config.k > weights.len() {
        return Err(FlowError::InvalidInput(format!(
            "k={} exceeds unit count {}",
            config.k,
            weights.len()
        )));
    }
    if weights.iter().any(|&weight| weight <= 0) {
        return Err(FlowError::InvalidInput(
            "weights must be positive".to_string(),
        ));
    }
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            if neighbor >= adjacency.len() {
                return Err(FlowError::InvalidInput(format!(
                    "neighbor index {neighbor} out of bounds for node {node}"
                )));
            }
        }
    }
    Ok(())
}

fn capacity_bounds(weights: &[i64], k: usize, tolerance: f64) -> (f64, f64, f64) {
    let total: i64 = weights.iter().sum();
    let ideal = total as f64 / k as f64;
    (ideal * (1.0 - tolerance), ideal, ideal * (1.0 + tolerance))
}

fn capacity_feasible(weights: &[i64], k: usize, tolerance: f64) -> bool {
    let (lower, _, upper) = capacity_bounds(weights, k, tolerance);
    if !weights.iter().all(|&weight| weight as f64 <= upper) {
        return false;
    }
    if weights.len() > 32 {
        return true;
    }
    let mut sorted = weights.to_vec();
    sorted.sort_by(|a, b| b.cmp(a));
    let mut bins = vec![0i64; k];
    exact_capacity_feasible(&sorted, 0, &mut bins, lower, upper)
}

fn exact_capacity_feasible(
    weights: &[i64],
    idx: usize,
    bins: &mut [i64],
    lower: f64,
    upper: f64,
) -> bool {
    if idx == weights.len() {
        return bins
            .iter()
            .all(|&pop| pop as f64 >= lower && pop as f64 <= upper);
    }
    for district in 0..bins.len() {
        if bins[district] + weights[idx] > upper.ceil() as i64 {
            continue;
        }
        bins[district] += weights[idx];
        if exact_capacity_feasible(weights, idx + 1, bins, lower, upper) {
            return true;
        }
        bins[district] -= weights[idx];
        if bins[district] == 0 {
            break;
        }
    }
    false
}

fn population_deviation(weights: &[i64], assignment: &[usize], k: usize) -> f64 {
    let total: i64 = weights.iter().sum();
    let ideal = total as f64 / k as f64;
    let mut district_pop = vec![0i64; k];
    for (&district, &weight) in assignment.iter().zip(weights.iter()) {
        if district < k {
            district_pop[district] += weight;
        }
    }
    district_pop
        .into_iter()
        .map(|pop| (pop as f64 - ideal).abs() / ideal)
        .fold(0.0, f64::max)
}

fn edge_cut(adjacency: &[Vec<usize>], assignment: &[usize]) -> usize {
    rgraph_core::undirected_edge_cut(adjacency, assignment)
        .expect("validated flow adjacency and assignment")
}

fn all_districts_connected(adjacency: &[Vec<usize>], assignment: &[usize], k: usize) -> bool {
    (0..k).all(|district| district_connected(adjacency, assignment, district))
}

fn district_connected(adjacency: &[Vec<usize>], assignment: &[usize], district: usize) -> bool {
    let Some(start) = assignment.iter().position(|&value| value == district) else {
        return false;
    };
    let district_size = assignment
        .iter()
        .filter(|&&value| value == district)
        .count();
    let mut visited = vec![false; assignment.len()];
    let mut queue = VecDeque::from([start]);
    visited[start] = true;
    let mut seen = 0usize;
    while let Some(node) = queue.pop_front() {
        seen += 1;
        for &neighbor in &adjacency[node] {
            if neighbor < assignment.len() && !visited[neighbor] && assignment[neighbor] == district
            {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }
    seen == district_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_construction_balances_path_fixture() {
        let result = construct_flow(&path_adj(6), &[100; 6], FlowConfig::new(2, 0.01)).unwrap();

        assert_eq!(result.status, FlowStatus::Valid);
        assert_eq!(result.assignment, vec![0, 0, 0, 1, 1, 1]);
        assert_eq!(result.summary.edge_cut, 1);
        assert_eq!(result.summary.population_deviation, 0.0);
        assert!(result.summary.parameter_hash.starts_with("sha256:"));
    }

    #[test]
    fn flow_construction_detects_infeasible_capacity() {
        let result = construct_flow(&path_adj(3), &[300, 1, 1], FlowConfig::new(2, 0.01)).unwrap();

        assert_eq!(result.status, FlowStatus::InfeasibleCapacity);
        assert!(result.assignment.is_empty());
        assert!(result.summary.infeasibility_witness.is_some());
    }

    #[test]
    fn flow_construction_is_deterministic() {
        let adjacency = path_adj(8);
        let config = FlowConfig::new(4, 0.01);

        let first = construct_flow(&adjacency, &[100; 8], config.clone()).unwrap();
        let second = construct_flow(&adjacency, &[100; 8], config).unwrap();

        assert_eq!(first.assignment, second.assignment);
        assert_eq!(first.summary, second.summary);
    }

    #[test]
    fn flow_summary_builds_algorithm_lineage() {
        let result = construct_flow(&path_adj(6), &[100; 6], FlowConfig::new(2, 0.01)).unwrap();

        let lineage = result
            .summary
            .algorithm_lineage("0.1.0", Vec::new())
            .unwrap();
        assert_eq!(lineage.producer_crate, "bisect-flow");
        assert_eq!(lineage.method, "flow-construction");
        assert_eq!(lineage.extra["status"], "valid");
    }

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
}
