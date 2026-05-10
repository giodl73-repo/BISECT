//! Solver-neutral separation rounds for U.16 branch-and-cut workflows.
//!
//! A real solver integration can call this after each incumbent solution. The
//! result is intentionally plain JSON-friendly metadata plus the violated cuts
//! that should be added before the next solve.

use crate::connectivity_cuts::{separate_connectivity_cuts, ConnectivityCut};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SEPARATION_ROUND_SCHEMA_VERSION: &str = "branch-and-cut-separation-round-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeparationRoundStatus {
    NoViolatedCuts,
    ViolatedConnectivityCuts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeparationRound {
    pub schema_version: String,
    pub round_index: usize,
    pub k: usize,
    pub n_vertices: usize,
    pub status: SeparationRoundStatus,
    pub cut_count: usize,
    pub disconnected_districts: Vec<usize>,
    pub cuts: Vec<ConnectivityCut>,
}

pub fn separate_round(
    adjacency: &[Vec<usize>],
    assignment: &[usize],
    k: usize,
    round_index: usize,
) -> SeparationRound {
    let cuts = separate_connectivity_cuts(adjacency, assignment, k);
    let disconnected_districts = cuts
        .iter()
        .map(|cut| cut.district_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let status = if cuts.is_empty() {
        SeparationRoundStatus::NoViolatedCuts
    } else {
        SeparationRoundStatus::ViolatedConnectivityCuts
    };
    SeparationRound {
        schema_version: SEPARATION_ROUND_SCHEMA_VERSION.to_string(),
        round_index,
        k,
        n_vertices: adjacency.len(),
        status,
        cut_count: cuts.len(),
        disconnected_districts,
        cuts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_5() -> Vec<Vec<usize>> {
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]]
    }

    #[test]
    fn connected_round_has_no_violated_cuts() {
        let round = separate_round(&path_5(), &[0, 0, 0, 1, 1], 2, 7);
        assert_eq!(round.schema_version, SEPARATION_ROUND_SCHEMA_VERSION);
        assert_eq!(round.round_index, 7);
        assert_eq!(round.status, SeparationRoundStatus::NoViolatedCuts);
        assert_eq!(round.cut_count, 0);
    }

    #[test]
    fn disconnected_round_reports_districts_and_cuts() {
        let round = separate_round(&path_5(), &[0, 1, 0, 1, 1], 2, 0);
        assert_eq!(
            round.status,
            SeparationRoundStatus::ViolatedConnectivityCuts
        );
        assert_eq!(round.cut_count, 4);
        assert_eq!(round.disconnected_districts, vec![0, 1]);
    }

    #[test]
    fn separation_round_json_round_trips() {
        let round = separate_round(&path_5(), &[0, 1, 0, 1, 1], 2, 2);
        let json = serde_json::to_string(&round).unwrap();
        assert!(json.contains("violated-connectivity-cuts"));
        let decoded: SeparationRound = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, round);
    }
}
