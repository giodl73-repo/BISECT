//! U.16 branch-and-cut certificate metadata.
//!
//! This is intentionally solver-neutral. It records whether a candidate
//! incumbent has violated connectivity cuts and distinguishes true lazy
//! callbacks from iterative solve-separate-resolve separation.

use crate::connectivity_cuts::{separate_connectivity_cuts, ConnectivityCut};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BranchAndCutMode {
    LazyCallback,
    IterativeSeparation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BranchAndCutSeparationStatus {
    NoViolatedCuts,
    ViolatedConnectivityCuts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchAndCutCertificate {
    pub schema_version: String,
    pub mode: BranchAndCutMode,
    pub status: BranchAndCutSeparationStatus,
    pub k: usize,
    pub n_vertices: usize,
    pub cut_count: usize,
    pub disconnected_districts: Vec<usize>,
    pub cuts: Vec<ConnectivityCut>,
    pub lower_bound: Option<f64>,
    pub incumbent_objective: Option<f64>,
    pub optimality_gap: Option<f64>,
    pub solver_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_search: Option<ExactSearchStats>,
    pub solver_mode_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactSearchStats {
    pub search_strategy: String,
    pub search_limit_vertices: usize,
    pub nodes_visited: usize,
    pub pruned_by_bound: usize,
    pub pruned_by_population: usize,
    pub complete_assignments_checked: usize,
    pub connected_feasible_assignments: usize,
    pub incumbent_updates: usize,
}

pub fn branch_and_cut_certificate(
    adjacency: &[Vec<usize>],
    assignment: &[usize],
    k: usize,
    mode: BranchAndCutMode,
) -> BranchAndCutCertificate {
    let cuts = separate_connectivity_cuts(adjacency, assignment, k);
    let disconnected_districts = cuts
        .iter()
        .map(|cut| cut.district_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let status = if cuts.is_empty() {
        BranchAndCutSeparationStatus::NoViolatedCuts
    } else {
        BranchAndCutSeparationStatus::ViolatedConnectivityCuts
    };
    BranchAndCutCertificate {
        schema_version: "branch-and-cut-certificate-v1".to_string(),
        mode,
        status,
        k,
        n_vertices: adjacency.len(),
        cut_count: cuts.len(),
        disconnected_districts,
        cuts,
        lower_bound: None,
        incumbent_objective: None,
        optimality_gap: None,
        solver_name: None,
        exact_search: None,
        solver_mode_note: match mode {
            BranchAndCutMode::LazyCallback => {
                "solver supports lazy connectivity-cut callbacks".to_string()
            }
            BranchAndCutMode::IterativeSeparation => {
                "iterative solve-separate-resolve mode; not a lazy-callback proof".to_string()
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_5() -> Vec<Vec<usize>> {
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]]
    }

    #[test]
    fn connected_solution_certificate_has_zero_violated_cuts() {
        let cert = branch_and_cut_certificate(
            &path_5(),
            &[0, 0, 0, 1, 1],
            2,
            BranchAndCutMode::IterativeSeparation,
        );
        assert_eq!(cert.status, BranchAndCutSeparationStatus::NoViolatedCuts);
        assert_eq!(cert.cut_count, 0);
        assert!(cert.disconnected_districts.is_empty());
    }

    #[test]
    fn disconnected_solution_certificate_reports_cut_count() {
        let cert = branch_and_cut_certificate(
            &path_5(),
            &[0, 1, 0, 1, 1],
            2,
            BranchAndCutMode::IterativeSeparation,
        );
        assert_eq!(
            cert.status,
            BranchAndCutSeparationStatus::ViolatedConnectivityCuts
        );
        assert_eq!(cert.cut_count, 4);
        assert_eq!(cert.disconnected_districts, vec![0, 1]);
        assert!(cert.solver_mode_note.contains("iterative"));
    }

    #[test]
    fn certificate_json_round_trips() {
        let cert = branch_and_cut_certificate(
            &path_5(),
            &[0, 0, 0, 1, 1],
            2,
            BranchAndCutMode::LazyCallback,
        );
        let json = serde_json::to_string(&cert).unwrap();
        let decoded: BranchAndCutCertificate = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.schema_version, "branch-and-cut-certificate-v1");
        assert_eq!(decoded.mode, BranchAndCutMode::LazyCallback);
    }
}
