use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::assign::{ClusterConfig, ClusterError, ClusterStatus};
use crate::metrics::{all_clusters_connected, edge_cut, population_deviation};
use crate::output::RegionalizationSummary;
use crate::repair::{repair_to_valid_small, RepairStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeWitness {
    pub step: usize,
    pub left_region: usize,
    pub right_region: usize,
    pub merged_region: usize,
    pub merged_population: i64,
    pub cut_edges_between: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalizationResult {
    pub assignment: Vec<usize>,
    pub status: ClusterStatus,
    pub merge_log: Vec<MergeWitness>,
    pub summary: RegionalizationSummary,
}

#[derive(Debug, Clone)]
struct Region {
    members: BTreeSet<usize>,
    population: i64,
    depth: usize,
}

pub fn regionalize(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: ClusterConfig,
) -> Result<RegionalizationResult, ClusterError> {
    validate_inputs(adjacency, weights, &config)?;
    if !capacity_feasible(weights, config.k, config.tolerance) {
        return Ok(empty_result(
            ClusterStatus::InfeasibleCapacity,
            f64::INFINITY,
            0,
        ));
    }

    let mut regions: BTreeMap<usize, Region> = weights
        .iter()
        .enumerate()
        .map(|(idx, &population)| {
            (
                idx,
                Region {
                    members: BTreeSet::from([idx]),
                    population,
                    depth: 0,
                },
            )
        })
        .collect();
    let mut merge_log = Vec::new();

    while regions.len() > config.k {
        let (left_id, right_id, cut_edges) =
            choose_merge_pair(adjacency, &regions, weights, config.k, config.tolerance)
                .ok_or_else(|| {
                    ClusterError::InvalidInput(
                        "regionalization requires at least one adjacent region pair".to_string(),
                    )
                })?;
        let right = regions
            .remove(&right_id)
            .expect("chosen right region exists");
        let left = regions
            .get_mut(&left_id)
            .expect("chosen left region exists");
        let right_population = right.population;
        let right_depth = right.depth;
        left.members.extend(right.members);
        left.population += right_population;
        left.depth = left.depth.max(right_depth) + 1;
        merge_log.push(MergeWitness {
            step: merge_log.len(),
            left_region: left_id,
            right_region: right_id,
            merged_region: left_id,
            merged_population: left.population,
            cut_edges_between: cut_edges,
        });
    }

    let mut assignment = assignment_from_regions(weights.len(), &regions);
    let mut deviation = population_deviation(weights, &assignment, config.k);
    let mut connected = all_clusters_connected(adjacency, &assignment, config.k);
    let mut status = if deviation <= config.tolerance && connected {
        ClusterStatus::Valid
    } else {
        ClusterStatus::NeedsRepair
    };
    let mut repair_status = if connected {
        RepairStatus::NotNeeded
    } else {
        RepairStatus::Needed
    };
    let mut repair_method = "none";

    if status == ClusterStatus::NeedsRepair {
        if let Some(repaired) =
            repair_to_valid_small(adjacency, weights, &assignment, config.k, config.tolerance)
        {
            assignment = repaired.assignment;
            deviation = population_deviation(weights, &assignment, config.k);
            connected = all_clusters_connected(adjacency, &assignment, config.k);
            if deviation <= config.tolerance && connected {
                status = ClusterStatus::Valid;
                repair_status = RepairStatus::Repaired;
                repair_method = "exhaustive-small";
            }
        }
    }

    let cut = edge_cut(adjacency, &assignment);
    let hierarchy_depth = regions
        .values()
        .map(|region| region.depth)
        .max()
        .unwrap_or(0);
    Ok(RegionalizationResult {
        assignment,
        status,
        merge_log,
        summary: RegionalizationSummary::new(
            "adjacent-balanced-agglomerative",
            repair_method,
            status_label(status),
            repair_status,
            deviation,
            cut,
            weights.len().saturating_sub(config.k),
            hierarchy_depth,
        ),
    })
}

fn choose_merge_pair(
    adjacency: &[Vec<usize>],
    regions: &BTreeMap<usize, Region>,
    weights: &[i64],
    k: usize,
    tolerance: f64,
) -> Option<(usize, usize, usize)> {
    let total: i64 = weights.iter().sum();
    let ideal = total as f64 / k as f64;
    let upper = ideal * (1.0 + tolerance);
    let unit_to_region = unit_region_index(weights.len(), regions);
    let mut pair_edges: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for (unit, neighbors) in adjacency.iter().enumerate() {
        let left = unit_to_region[unit];
        for &neighbor in neighbors {
            if neighbor <= unit || neighbor >= unit_to_region.len() {
                continue;
            }
            let right = unit_to_region[neighbor];
            if left != right {
                pair_edges
                    .entry((left.min(right), left.max(right)))
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
    }

    pair_edges
        .into_iter()
        .map(|((left, right), cut_edges)| {
            let merged_pop = regions[&left].population + regions[&right].population;
            let over_capacity = merged_pop as f64 > upper;
            let distance = ((merged_pop as f64) - ideal).abs();
            (
                over_capacity,
                ordered_f64(distance),
                merged_pop,
                left,
                right,
                cut_edges,
            )
        })
        .min()
        .map(|(_, _, _, left, right, cut_edges)| (left, right, cut_edges))
}

fn assignment_from_regions(n_units: usize, regions: &BTreeMap<usize, Region>) -> Vec<usize> {
    let mut assignment = vec![usize::MAX; n_units];
    for (district, region) in regions.values().enumerate() {
        for &unit in &region.members {
            assignment[unit] = district;
        }
    }
    assignment
}

fn unit_region_index(n_units: usize, regions: &BTreeMap<usize, Region>) -> Vec<usize> {
    let mut index = vec![usize::MAX; n_units];
    for (&region_id, region) in regions {
        for &unit in &region.members {
            index[unit] = region_id;
        }
    }
    index
}

fn ordered_f64(value: f64) -> u64 {
    value.to_bits()
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

fn empty_result(
    status: ClusterStatus,
    population_deviation: f64,
    edge_cut: usize,
) -> RegionalizationResult {
    RegionalizationResult {
        assignment: Vec::new(),
        status,
        merge_log: Vec::new(),
        summary: RegionalizationSummary::new(
            "adjacent-balanced-agglomerative",
            "none",
            status_label(status),
            RepairStatus::NotAttempted,
            population_deviation,
            edge_cut,
            0,
            0,
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
