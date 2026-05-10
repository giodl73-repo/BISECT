//! Connectivity-cut separation for branch-and-cut ILP workflows.
//!
//! This module does not require a solver callback API. It inspects an incumbent
//! assignment and returns disconnected district witnesses that an iterative
//! solve-separate-resolve loop can turn into connectivity cuts.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictComponents {
    pub district_id: usize,
    pub components: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectivityCut {
    pub district_id: usize,
    pub component: Vec<usize>,
    pub outside_neighbors: Vec<usize>,
}

pub fn separate_connectivity_cuts(
    adjacency: &[Vec<usize>],
    assignment: &[usize],
    k: usize,
) -> Vec<ConnectivityCut> {
    disconnected_components(adjacency, assignment, k)
        .into_iter()
        .flat_map(|district| cuts_for_district(adjacency, district))
        .collect()
}

pub fn disconnected_components(
    adjacency: &[Vec<usize>],
    assignment: &[usize],
    k: usize,
) -> Vec<DistrictComponents> {
    let mut disconnected = Vec::new();
    for district_id in 0..k {
        let members: Vec<usize> = assignment
            .iter()
            .enumerate()
            .filter_map(|(idx, &assigned)| (assigned == district_id).then_some(idx))
            .collect();
        let components = components_for_members(adjacency, &members);
        if components.len() > 1 {
            disconnected.push(DistrictComponents {
                district_id,
                components,
            });
        }
    }
    disconnected
}

fn cuts_for_district(
    adjacency: &[Vec<usize>],
    district: DistrictComponents,
) -> Vec<ConnectivityCut> {
    let mut cuts = Vec::new();
    for component in district.components {
        let component_set: BTreeSet<usize> = component.iter().copied().collect();
        let mut outside_neighbors = BTreeSet::new();
        for &node in &component {
            for &neighbor in &adjacency[node] {
                if !component_set.contains(&neighbor) {
                    outside_neighbors.insert(neighbor);
                }
            }
        }
        cuts.push(ConnectivityCut {
            district_id: district.district_id,
            component,
            outside_neighbors: outside_neighbors.into_iter().collect(),
        });
    }
    cuts
}

fn components_for_members(adjacency: &[Vec<usize>], members: &[usize]) -> Vec<Vec<usize>> {
    let member_set: BTreeSet<usize> = members.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut components = Vec::new();
    for &start in members {
        if seen.contains(&start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::new();
        seen.insert(start);
        while let Some(node) = queue.pop_front() {
            component.push(node);
            for &neighbor in &adjacency[node] {
                if member_set.contains(&neighbor) && seen.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
        components.push(component);
    }
    components
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_5() -> Vec<Vec<usize>> {
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]]
    }

    #[test]
    fn connected_solution_emits_zero_cuts() {
        let cuts = separate_connectivity_cuts(&path_5(), &[0, 0, 0, 1, 1], 2);
        assert!(cuts.is_empty());
    }

    #[test]
    fn disconnected_incumbent_triggers_cuts() {
        let cuts = separate_connectivity_cuts(&path_5(), &[0, 1, 0, 1, 1], 2);
        assert_eq!(cuts.len(), 4);
        assert!(cuts.iter().any(|cut| cut.component == vec![0]));
        assert!(cuts.iter().any(|cut| cut.component == vec![2]));
        assert!(cuts
            .iter()
            .any(|cut| cut.district_id == 1 && cut.component == vec![1]));
        assert!(cuts
            .iter()
            .any(|cut| cut.district_id == 1 && cut.component == vec![3, 4]));
    }

    #[test]
    fn outside_neighbors_identify_boundary_for_component() {
        let cuts = separate_connectivity_cuts(&path_5(), &[0, 1, 0, 1, 1], 2);
        let component_zero = cuts.iter().find(|cut| cut.component == vec![0]).unwrap();
        assert_eq!(component_zero.outside_neighbors, vec![1]);
    }
}
