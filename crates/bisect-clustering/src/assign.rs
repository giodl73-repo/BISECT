use thiserror::Error;

use crate::metrics::{all_clusters_connected, edge_cut, population_deviation};
use crate::output::ClusterSummary;
use crate::repair::RepairStatus;
use crate::seeds::{distances_from, farthest_point_seeds};

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub k: usize,
    pub tolerance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClusterStatus {
    Valid,
    NeedsRepair,
    InfeasibleCapacity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterResult {
    pub assignment: Vec<usize>,
    pub seeds: Vec<usize>,
    pub status: ClusterStatus,
    pub summary: ClusterSummary,
}

#[derive(Debug, Error, PartialEq)]
pub enum ClusterError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub fn capacity_cluster(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: ClusterConfig,
) -> Result<ClusterResult, ClusterError> {
    validate_inputs(adjacency, weights, &config)?;
    if !capacity_feasible(weights, config.k, config.tolerance) {
        return Ok(empty_result(
            ClusterStatus::InfeasibleCapacity,
            f64::INFINITY,
            0,
        ));
    }

    let seeds = farthest_point_seeds(adjacency, config.k).map_err(ClusterError::InvalidInput)?;
    let assignment = assign_to_nearest_seed_with_capacity(adjacency, weights, &seeds, &config);
    let deviation = population_deviation(weights, &assignment, config.k);
    let cut = edge_cut(adjacency, &assignment);
    let connected = all_clusters_connected(adjacency, &assignment, config.k);
    let status = if deviation <= config.tolerance && connected {
        ClusterStatus::Valid
    } else {
        ClusterStatus::NeedsRepair
    };
    let summary = ClusterSummary::new(
        "capacity-clustering",
        "farthest",
        "none",
        status_label(status),
        if connected {
            RepairStatus::NotNeeded
        } else {
            RepairStatus::Needed
        },
        deviation,
        cut,
    );

    Ok(ClusterResult {
        assignment,
        seeds,
        status,
        summary,
    })
}

fn validate_inputs(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: &ClusterConfig,
) -> Result<(), ClusterError> {
    if config.k == 0 {
        return Err(ClusterError::InvalidInput(
            "k must be greater than zero".to_string(),
        ));
    }
    if adjacency.len() != weights.len() {
        return Err(ClusterError::InvalidInput(format!(
            "adjacency has {} units but weights has {}",
            adjacency.len(),
            weights.len()
        )));
    }
    if config.k > weights.len() {
        return Err(ClusterError::InvalidInput(format!(
            "k={} exceeds unit count {}",
            config.k,
            weights.len()
        )));
    }
    if weights.iter().any(|&weight| weight <= 0) {
        return Err(ClusterError::InvalidInput(
            "weights must be positive".to_string(),
        ));
    }
    Ok(())
}

fn capacity_feasible(weights: &[i64], k: usize, tolerance: f64) -> bool {
    let total: i64 = weights.iter().sum();
    let ideal = total as f64 / k as f64;
    let lower = ideal * (1.0 - tolerance);
    let upper = ideal * (1.0 + tolerance);
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
            .all(|&pop| (pop as f64) >= lower && (pop as f64) <= upper);
    }

    let weight = weights[idx];
    for district in 0..bins.len() {
        if bins[district] + weight > upper.ceil() as i64 {
            continue;
        }
        bins[district] += weight;
        if exact_capacity_feasible(weights, idx + 1, bins, lower, upper) {
            return true;
        }
        bins[district] -= weight;
        if bins[district] == 0 {
            break;
        }
    }
    false
}

fn assign_to_nearest_seed_with_capacity(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    seeds: &[usize],
    config: &ClusterConfig,
) -> Vec<usize> {
    let distances_by_seed: Vec<Vec<Option<usize>>> = seeds
        .iter()
        .map(|&seed| distances_from(adjacency, seed))
        .collect();
    let total: i64 = weights.iter().sum();
    let ideal = total as f64 / config.k as f64;
    let upper = ideal * (1.0 + config.tolerance);
    let mut district_pop = vec![0i64; config.k];
    let mut assignment = vec![usize::MAX; weights.len()];

    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by_key(|&idx| {
        let nearest = distances_by_seed
            .iter()
            .filter_map(|distances| distances[idx])
            .min()
            .unwrap_or(usize::MAX);
        (nearest, idx)
    });

    for unit in order {
        let mut ranked: Vec<(usize, usize)> = distances_by_seed
            .iter()
            .enumerate()
            .map(|(district, distances)| (distances[unit].unwrap_or(usize::MAX), district))
            .collect();
        ranked.sort();
        let chosen = ranked
            .iter()
            .find(|(_, district)| district_pop[*district] + weights[unit] <= upper.ceil() as i64)
            .or_else(|| ranked.first())
            .map(|(_, district)| *district)
            .unwrap_or(0);
        assignment[unit] = chosen;
        district_pop[chosen] += weights[unit];
    }

    assignment
}

fn empty_result(
    status: ClusterStatus,
    population_deviation: f64,
    edge_cut: usize,
) -> ClusterResult {
    ClusterResult {
        assignment: Vec::new(),
        seeds: Vec::new(),
        status,
        summary: ClusterSummary::new(
            "capacity-clustering",
            "farthest",
            "none",
            status_label(status),
            RepairStatus::NotAttempted,
            population_deviation,
            edge_cut,
        ),
    }
}

fn status_label(status: ClusterStatus) -> &'static str {
    match status {
        ClusterStatus::Valid => "valid",
        ClusterStatus::NeedsRepair => "needs-repair",
        ClusterStatus::InfeasibleCapacity => "infeasible-capacity",
    }
}
