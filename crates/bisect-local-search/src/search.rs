use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::metrics::{all_districts_connected, edge_cut, population_deviation};
use crate::output::LocalSearchSummary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalSearchMethod {
    OneMove,
    Tabu {
        max_iters: usize,
        tenure: usize,
    },
    Lns {
        max_iters: usize,
        destroy_size: usize,
    },
}

impl LocalSearchMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::OneMove => "one-move",
            Self::Tabu { .. } => "tabu",
            Self::Lns { .. } => "lns",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalSearchConfig {
    pub k: usize,
    pub tolerance: f64,
    pub method: LocalSearchMethod,
}

impl LocalSearchConfig {
    pub fn one_move(k: usize, tolerance: f64) -> Self {
        Self {
            k,
            tolerance,
            method: LocalSearchMethod::OneMove,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImproveStatus {
    Improved,
    NoImprovement,
}

impl ImproveStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Improved => "improved",
            Self::NoImprovement => "no-improvement",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImproveResult {
    pub assignment: Vec<usize>,
    pub status: ImproveStatus,
    pub moved_vertex: Option<usize>,
    pub moved_from: Option<usize>,
    pub moved_to: Option<usize>,
    pub summary: LocalSearchSummary,
}

#[derive(Debug, Error, PartialEq)]
pub enum ImproveError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("method is staged: {0}")]
    StagedMethod(String),
}

pub fn improve_one_move(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    assignment: &[usize],
    config: LocalSearchConfig,
) -> Result<ImproveResult, ImproveError> {
    validate_inputs(adjacency, weights, assignment, &config)?;
    if !matches!(config.method, LocalSearchMethod::OneMove) {
        return Err(ImproveError::StagedMethod(
            config.method.label().to_string(),
        ));
    }

    let initial_cut = edge_cut(adjacency, assignment);
    let initial_deviation = population_deviation(weights, assignment, config.k);
    if initial_deviation > config.tolerance
        || !all_districts_connected(adjacency, assignment, config.k)
    {
        return Err(ImproveError::InvalidInput(
            "initial assignment must satisfy population tolerance and contiguity".to_string(),
        ));
    }

    let mut moves_evaluated = 0usize;
    let mut best: Option<(usize, usize, usize, usize, f64, Vec<usize>)> = None;
    for vertex in 0..assignment.len() {
        let from = assignment[vertex];
        let mut targets: Vec<usize> = adjacency[vertex]
            .iter()
            .filter_map(|&neighbor| assignment.get(neighbor).copied())
            .filter(|&district| district != from)
            .collect();
        targets.sort_unstable();
        targets.dedup();
        for to in targets {
            moves_evaluated += 1;
            let mut candidate = assignment.to_vec();
            candidate[vertex] = to;
            let deviation = population_deviation(weights, &candidate, config.k);
            if deviation > config.tolerance {
                continue;
            }
            if !all_districts_connected(adjacency, &candidate, config.k) {
                continue;
            }
            let candidate_cut = edge_cut(adjacency, &candidate);
            if candidate_cut >= initial_cut {
                continue;
            }
            let candidate_key = (candidate_cut, vertex, from, to);
            let replace = best
                .as_ref()
                .map(|(best_cut, best_vertex, best_from, best_to, _, _)| {
                    candidate_key < (*best_cut, *best_vertex, *best_from, *best_to)
                })
                .unwrap_or(true);
            if replace {
                best = Some((candidate_cut, vertex, from, to, deviation, candidate));
            }
        }
    }

    let (status, moved_vertex, moved_from, moved_to, final_cut, final_deviation, final_assignment) =
        if let Some((cut, vertex, from, to, deviation, candidate)) = best {
            (
                ImproveStatus::Improved,
                Some(vertex),
                Some(from),
                Some(to),
                cut,
                deviation,
                candidate,
            )
        } else {
            (
                ImproveStatus::NoImprovement,
                None,
                None,
                None,
                initial_cut,
                initial_deviation,
                assignment.to_vec(),
            )
        };

    let summary = LocalSearchSummary::new(
        config.method.label(),
        status.label(),
        moves_evaluated,
        usize::from(status == ImproveStatus::Improved),
        initial_cut,
        final_cut,
        initial_deviation,
        final_deviation,
        config.tolerance,
    );

    Ok(ImproveResult {
        assignment: final_assignment,
        status,
        moved_vertex,
        moved_from,
        moved_to,
        summary,
    })
}

fn validate_inputs(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    assignment: &[usize],
    config: &LocalSearchConfig,
) -> Result<(), ImproveError> {
    if config.k == 0 {
        return Err(ImproveError::InvalidInput(
            "k must be greater than zero".to_string(),
        ));
    }
    if adjacency.len() != weights.len() || adjacency.len() != assignment.len() {
        return Err(ImproveError::InvalidInput(
            "adjacency, weights, and assignment lengths must match".to_string(),
        ));
    }
    if assignment.iter().any(|&district| district >= config.k) {
        return Err(ImproveError::InvalidInput(
            "assignment contains district id outside 0..k".to_string(),
        ));
    }
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            if neighbor >= adjacency.len() {
                return Err(ImproveError::InvalidInput(format!(
                    "neighbor index {neighbor} out of bounds for node {node}"
                )));
            }
        }
    }
    Ok(())
}
