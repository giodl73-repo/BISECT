use rmath_core::normalize_centered;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

pub const SPECTRAL_SUMMARY_SCHEMA_VERSION: &str = "bisect-spectral-summary-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectralConfig {
    pub max_iters: usize,
    pub tolerance: f64,
    pub target_fraction: f64,
}

impl Default for SpectralConfig {
    fn default() -> Self {
        Self {
            max_iters: 200,
            tolerance: 0.05,
            target_fraction: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectralSummary {
    pub schema_version: String,
    pub method: String,
    pub sweep: String,
    pub iterations: usize,
    pub converged: bool,
    pub edge_cut: usize,
    pub population_deviation: f64,
    pub tolerance: f64,
    pub target_fraction: f64,
    pub parameter_hash: String,
}

impl SpectralSummary {
    fn new(
        iterations: usize,
        converged: bool,
        edge_cut: usize,
        population_deviation: f64,
        tolerance: f64,
        target_fraction: f64,
    ) -> Self {
        let mut summary = Self {
            schema_version: SPECTRAL_SUMMARY_SCHEMA_VERSION.to_string(),
            method: "spectral".to_string(),
            sweep: "population-balanced-min-cut".to_string(),
            iterations,
            converged,
            edge_cut,
            population_deviation,
            tolerance,
            target_fraction,
            parameter_hash: String::new(),
        };
        summary.parameter_hash = summary.compute_parameter_hash();
        summary
    }

    fn compute_parameter_hash(&self) -> String {
        let payload = serde_json::json!({
            "schema_version": self.schema_version,
            "method": self.method,
            "sweep": self.sweep,
            "iterations": self.iterations,
            "converged": self.converged,
            "edge_cut": self.edge_cut,
            "population_deviation": self.population_deviation,
            "tolerance": self.tolerance,
            "target_fraction": self.target_fraction,
        });
        let bytes = serde_json::to_vec(&payload).expect("spectral summary serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpectralResult {
    pub assignment: Vec<usize>,
    pub vector: Vec<f64>,
    pub summary: SpectralSummary,
}

#[derive(Debug, Error, PartialEq)]
pub enum SpectralError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("no population-balanced spectral sweep cut found")]
    NoBalancedCut,
}

pub fn spectral_bisect(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: SpectralConfig,
) -> Result<SpectralResult, SpectralError> {
    validate_inputs(adjacency, weights, &config)?;
    let (vector, iterations, converged) = smooth_spectral_vector(adjacency, config.max_iters);
    let (assignment, cut, deviation) = balanced_sweep(
        adjacency,
        weights,
        &vector,
        config.tolerance,
        config.target_fraction,
    )?;
    Ok(SpectralResult {
        assignment,
        vector,
        summary: SpectralSummary::new(
            iterations,
            converged,
            cut,
            deviation,
            config.tolerance,
            config.target_fraction,
        ),
    })
}

fn smooth_spectral_vector(adjacency: &[Vec<usize>], max_iters: usize) -> (Vec<f64>, usize, bool) {
    let n = adjacency.len();
    let center = (n.saturating_sub(1)) as f64 / 2.0;
    let anchor: Vec<f64> = (0..n).map(|idx| idx as f64 - center).collect();
    let mut vector = normalize_centered(anchor.clone(), 0.0).expect("valid spectral anchor vector");
    let mut converged = false;
    let mut iterations = 0usize;
    for iter in 0..max_iters {
        iterations = iter + 1;
        let mut next = vec![0.0; n];
        for (node, neighbors) in adjacency.iter().enumerate() {
            let neighbor_mean = if neighbors.is_empty() {
                vector[node]
            } else {
                neighbors
                    .iter()
                    .map(|&neighbor| vector[neighbor])
                    .sum::<f64>()
                    / neighbors.len() as f64
            };
            next[node] = 0.85 * neighbor_mean + 0.15 * anchor[node];
        }
        next = normalize_centered(next, 0.0).expect("valid spectral iteration vector");
        let delta = vector
            .iter()
            .zip(next.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        vector = next;
        if delta < 1.0e-9 {
            converged = true;
            break;
        }
    }
    (vector, iterations, converged)
}

fn balanced_sweep(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    vector: &[f64],
    tolerance: f64,
    target_fraction: f64,
) -> Result<(Vec<usize>, usize, f64), SpectralError> {
    let n = adjacency.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| vector[a].total_cmp(&vector[b]).then_with(|| a.cmp(&b)));

    let mut best: Option<(usize, usize, f64, Vec<usize>)> = None;
    for split in 1..n {
        let left: HashSet<usize> = order[..split].iter().copied().collect();
        let assignment: Vec<usize> = (0..n)
            .map(|idx| if left.contains(&idx) { 0 } else { 1 })
            .collect();
        let deviation = target_population_deviation(weights, &assignment, target_fraction);
        if deviation > tolerance {
            continue;
        }
        let cut = edge_cut(adjacency, &assignment);
        let key = (cut, split);
        let replace = best
            .as_ref()
            .map(|(best_cut, best_split, _, _)| key < (*best_cut, *best_split))
            .unwrap_or(true);
        if replace {
            best = Some((cut, split, deviation, assignment));
        }
    }
    best.map(|(cut, _, deviation, assignment)| (assignment, cut, deviation))
        .ok_or(SpectralError::NoBalancedCut)
}

fn edge_cut(adjacency: &[Vec<usize>], assignment: &[usize]) -> usize {
    rgraph_core::undirected_edge_cut(adjacency, assignment)
        .expect("validated spectral adjacency and assignment")
}

fn target_population_deviation(weights: &[i64], assignment: &[usize], target_fraction: f64) -> f64 {
    let total: i64 = weights.iter().sum();
    let targets = [
        total as f64 * target_fraction,
        total as f64 * (1.0 - target_fraction),
    ];
    let mut district_pop = [0i64; 2];
    for (&district, &weight) in assignment.iter().zip(weights.iter()) {
        district_pop[district] += weight;
    }
    district_pop
        .into_iter()
        .zip(targets)
        .map(|(pop, target)| (pop as f64 - target).abs() / target)
        .fold(0.0, f64::max)
}

fn validate_inputs(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    config: &SpectralConfig,
) -> Result<(), SpectralError> {
    if adjacency.len() < 2 {
        return Err(SpectralError::InvalidInput(
            "at least two vertices are required".to_string(),
        ));
    }
    if adjacency.len() != weights.len() {
        return Err(SpectralError::InvalidInput(
            "adjacency and weights lengths must match".to_string(),
        ));
    }
    if config.max_iters == 0 {
        return Err(SpectralError::InvalidInput(
            "max_iters must be greater than zero".to_string(),
        ));
    }
    if !(0.0..1.0).contains(&config.target_fraction) {
        return Err(SpectralError::InvalidInput(
            "target_fraction must be greater than zero and less than one".to_string(),
        ));
    }
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            if neighbor >= adjacency.len() {
                return Err(SpectralError::InvalidInput(format!(
                    "neighbor index {neighbor} out of bounds for node {node}"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_graph_splits_near_midpoint() {
        let result = spectral_bisect(&path_adj(6), &[100; 6], SpectralConfig::default()).unwrap();

        assert_eq!(result.assignment, vec![0, 0, 0, 1, 1, 1]);
        assert_eq!(result.summary.edge_cut, 1);
        assert_eq!(result.summary.population_deviation, 0.0);
    }

    #[test]
    fn two_clique_bridge_cuts_bridge() {
        let result =
            spectral_bisect(&two_clique_bridge(), &[100; 6], SpectralConfig::default()).unwrap();

        assert_eq!(result.assignment, vec![0, 0, 0, 1, 1, 1]);
        assert_eq!(result.summary.edge_cut, 1);
    }

    #[test]
    fn path_graph_honors_non_half_target_fraction() {
        let result = spectral_bisect(
            &path_adj(6),
            &[100; 6],
            SpectralConfig {
                target_fraction: 1.0 / 3.0,
                tolerance: 0.001,
                ..SpectralConfig::default()
            },
        )
        .unwrap();

        assert_eq!(result.assignment, vec![0, 0, 1, 1, 1, 1]);
        assert_eq!(result.summary.edge_cut, 1);
        assert!(result.summary.population_deviation < 1.0e-12);
        assert_eq!(result.summary.target_fraction, 1.0 / 3.0);
    }

    #[test]
    fn spectral_bisect_is_deterministic() {
        let adjacency = two_clique_bridge();
        let config = SpectralConfig::default();

        let first = spectral_bisect(&adjacency, &[100; 6], config.clone()).unwrap();
        let second = spectral_bisect(&adjacency, &[100; 6], config).unwrap();

        assert_eq!(first.assignment, second.assignment);
        assert_eq!(first.vector, second.vector);
        assert_eq!(first.summary, second.summary);
        assert!(first.summary.parameter_hash.starts_with("sha256:"));
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

    fn two_clique_bridge() -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); 6];
        for clique in [0usize, 3] {
            for a in clique..clique + 3 {
                for b in clique..clique + 3 {
                    if a != b {
                        adjacency[a].push(b);
                    }
                }
            }
        }
        adjacency[2].push(3);
        adjacency[3].push(2);
        adjacency
    }
}

/// Shared recursive spectral schedule used by the native CLI and browser engine.
pub fn run_spectral_recursive(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    k: usize,
    tolerance: f64,
    max_iters: usize,
) -> Result<(Vec<usize>, serde_json::Value), String> {
    if k == 0 || k > adjacency.len() {
        return Err("spectral: k must be between one and the vertex count".to_string());
    }
    if adjacency.len() != weights.len() {
        return Err("spectral: adjacency and weight lengths must match".to_string());
    }
    if max_iters == 0 || !tolerance.is_finite() || !(0.0..=1.0).contains(&tolerance)
        || weights.iter().any(|&weight| weight < 0)
        || weights.iter().try_fold(0i64, |sum, &weight| sum.checked_add(weight)).filter(|&sum| sum > 0).is_none()
        || adjacency.iter().flatten().any(|&neighbor| neighbor >= adjacency.len()) {
        return Err("spectral: invalid iterations, tolerance, weights or adjacency".into());
    }
    let mut assignment = vec![0usize; adjacency.len()];
    let mut node_summaries = Vec::new();
    split_spectral_node(
        adjacency,
        weights,
        &(0..adjacency.len()).collect::<Vec<_>>(),
        k,
        0,
        tolerance,
        max_iters,
        &mut assignment,
        &mut node_summaries,
    )?;
    let edge_cut = count_edge_cuts_zero_based(&assignment, adjacency);
    Ok((
        assignment,
        serde_json::json!({
            "schema_version": "bisect-spectral-run-summary-v1",
            "method": "spectral",
            "max_iters": max_iters,
            "tolerance": tolerance,
            "k": k,
            "edge_cut": edge_cut,
            "nodes": node_summaries,
        }),
    ))
}

#[allow(clippy::too_many_arguments)]
fn split_spectral_node(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    vertices: &[usize],
    k: usize,
    district_offset: usize,
    tolerance: f64,
    max_iters: usize,
    assignment: &mut [usize],
    node_summaries: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    if k == 1 {
        for &vertex in vertices {
            assignment[vertex] = district_offset;
        }
        return Ok(());
    }
    let local_index: HashMap<usize, usize> = vertices
        .iter()
        .enumerate()
        .map(|(local, &global)| (global, local))
        .collect();
    let local_adjacency: Vec<Vec<usize>> = vertices
        .iter()
        .map(|&global| {
            adjacency[global]
                .iter()
                .filter_map(|neighbor| local_index.get(neighbor).copied())
                .collect()
        })
        .collect();
    let local_weights: Vec<i64> = vertices.iter().map(|&global| weights[global]).collect();
    let left_k = k / 2;
    let right_k = k - left_k;
    let result = crate::spectral_bisect(
        &local_adjacency,
        &local_weights,
        crate::SpectralConfig {
            max_iters,
            tolerance,
            target_fraction: left_k as f64 / k as f64,
        },
    )
    .map_err(|e| format!("spectral split failed: {e}"))?;
    node_summaries.push(serde_json::to_value(&result.summary).map_err(|e| e.to_string())?);

    let mut left = Vec::new();
    let mut right = Vec::new();
    for (local, &global) in vertices.iter().enumerate() {
        if result.assignment[local] == 0 {
            left.push(global);
        } else {
            right.push(global);
        }
    }
    split_spectral_node(
        adjacency,
        weights,
        &left,
        left_k,
        district_offset,
        tolerance,
        max_iters,
        assignment,
        node_summaries,
    )?;
    split_spectral_node(
        adjacency,
        weights,
        &right,
        right_k,
        district_offset + left_k,
        tolerance,
        max_iters,
        assignment,
        node_summaries,
    )
}

fn count_edge_cuts_zero_based(assignment: &[usize], adjacency: &[Vec<usize>]) -> usize {
    rgraph_core::undirected_edge_cut(adjacency, assignment)
        .expect("validated zero-based spectral adjacency and assignment")
}


#[cfg(test)]
mod recursive_tests {
    use super::*;
    fn grid() -> (Vec<Vec<usize>>,Vec<i64>) {
        let width=12;let n=width*width;
        let adjacency=(0..n).map(|i| {
            let mut neighbors=Vec::new();
            if i%width>0 {neighbors.push(i-1);} if i%width<width-1 {neighbors.push(i+1);}
            if i>=width {neighbors.push(i-width);} if i<n-width {neighbors.push(i+width);}
            neighbors
        }).collect();
        (adjacency,vec![100;n])
    }
    #[test]
    fn recursive_schedule_preserves_summary_and_every_vertex() {
        let (adjacency,weights)=grid();
        for k in [1,2,3,6] {
            let (assignment,summary)=run_spectral_recursive(&adjacency,&weights,k,0.05,200).unwrap();
            assert_eq!(assignment.len(),weights.len());
            assert!(assignment.iter().all(|&district|district<k));
            assert_eq!(summary["nodes"].as_array().unwrap().len(),k-1);
            assert_eq!(summary["edge_cut"].as_u64().unwrap() as usize,rgraph_core::undirected_edge_cut(&adjacency,&assignment).unwrap());
            assert_eq!(run_spectral_recursive(&adjacency,&weights,k,0.05,200).unwrap(),(assignment,summary));
        }
    }
    #[test]
    fn recursive_rejects_invalid_dimensions_and_numeric_inputs() {
        let (adjacency,weights)=grid();
        for (k,tolerance,iters) in [(0,0.05,200),(145,0.05,200),(2,f64::NAN,200),(2,0.05,0)] {
            assert!(run_spectral_recursive(&adjacency,&weights,k,tolerance,iters).is_err());
        }
        assert!(run_spectral_recursive(&adjacency,&weights[..2],2,0.05,200).is_err());
        assert!(run_spectral_recursive(&[vec![10],vec![0]],&[1,1],2,0.05,200).is_err());
        assert!(run_spectral_recursive(&[vec![1],vec![0]],&[i64::MAX,i64::MAX],2,0.05,200).is_err());
    }
}
