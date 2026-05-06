//! PartialPlan: a partially-assigned redistricting plan used during SMC construction.
//!
//! At stage t, districts 1..=t are fully assigned. Remaining tracts are unassigned (None).

use std::collections::VecDeque;

/// A partially-assigned redistricting plan.
///
/// During SMC, districts are added one at a time. After stage t, tracts are either
/// assigned to a district in 1..=t or unassigned (None).
#[derive(Clone)]
pub struct PartialPlan {
    /// assignment[tract] = Some(district_id 1-based) or None (unassigned)
    pub assignment: Vec<Option<u32>>,
    /// Population per tract (shared reference — not cloned)
    pub n_tracts: usize,
    /// Number of unassigned tracts remaining
    pub unassigned_count: usize,
}

impl PartialPlan {
    /// Create an empty plan: all tracts unassigned.
    pub fn empty(n_tracts: usize) -> Self {
        Self {
            assignment: vec![None; n_tracts],
            n_tracts,
            unassigned_count: n_tracts,
        }
    }

    /// Assign a set of tracts to a new district. Panics if any tract is already assigned.
    pub fn assign_district(&mut self, tracts: &[usize], district: u32) {
        for &t in tracts {
            debug_assert!(self.assignment[t].is_none(), "tract {t} already assigned");
            self.assignment[t] = Some(district);
            self.unassigned_count -= 1;
        }
    }

    /// Assign all remaining unassigned tracts to the final district k.
    pub fn assign_remaining(&mut self, k: u32) {
        for a in self.assignment.iter_mut() {
            if a.is_none() {
                *a = Some(k);
            }
        }
        self.unassigned_count = 0;
    }

    /// Return the indices of all unassigned tracts.
    pub fn unassigned_tracts(&self) -> Vec<usize> {
        self.assignment.iter().enumerate()
            .filter_map(|(i, a)| if a.is_none() { Some(i) } else { None })
            .collect()
    }

    /// Find connected components of the unassigned subgraph.
    ///
    /// Returns a Vec of components, each being a Vec of tract indices.
    /// Uses BFS over the unassigned tracts only.
    pub fn unassigned_components(&self, adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let mut visited = vec![false; self.n_tracts];
        let mut components = Vec::new();

        for start in 0..self.n_tracts {
            if self.assignment[start].is_some() || visited[start] {
                continue;
            }
            // BFS from `start` over unassigned tracts
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;
            while let Some(v) = queue.pop_front() {
                component.push(v);
                for &nb in &adj[v] {
                    if !visited[nb] && self.assignment[nb].is_none() {
                        visited[nb] = true;
                        queue.push_back(nb);
                    }
                }
            }
            components.push(component);
        }
        components
    }

    /// Find the largest connected component of unassigned tracts by population.
    ///
    /// Returns the component (list of tract indices) with the highest total population.
    /// Per spec §2.2: the spanning tree is built over this component, ensuring
    /// the connectivity invariant (remaining unassigned tracts stay connected).
    pub fn largest_unassigned_component<'a>(
        &self,
        adj: &[Vec<usize>],
        pop: &[i64],
    ) -> Vec<usize> {
        let components = self.unassigned_components(adj);
        components.into_iter()
            .max_by_key(|c| c.iter().map(|&t| pop[t]).sum::<i64>())
            .unwrap_or_default()
    }

    /// Extract the final complete assignment as a Vec<u32> (1-based district IDs).
    /// Panics if any tract is still unassigned.
    pub fn finalise(&self) -> Vec<u32> {
        self.assignment.iter()
            .map(|a| a.expect("finalise called with unassigned tracts"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj(n: usize) -> Vec<Vec<usize>> {
        (0..n).map(|i| {
            let mut nb = Vec::new();
            if i > 0 { nb.push(i - 1); }
            if i < n - 1 { nb.push(i + 1); }
            nb
        }).collect()
    }

    #[test]
    fn empty_plan_all_unassigned() {
        let p = PartialPlan::empty(4);
        assert_eq!(p.unassigned_count, 4);
        assert!(p.assignment.iter().all(|a| a.is_none()));
    }

    #[test]
    fn assign_district_reduces_count() {
        let mut p = PartialPlan::empty(4);
        p.assign_district(&[0, 1], 1);
        assert_eq!(p.unassigned_count, 2);
        assert_eq!(p.assignment[0], Some(1));
        assert_eq!(p.assignment[2], None);
    }

    #[test]
    fn unassigned_components_path_graph() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let mut p = PartialPlan::empty(4);
        // Assign tract 1 — splits path into {0} and {2,3}
        p.assign_district(&[1], 1);
        let comps = p.unassigned_components(&adj);
        assert_eq!(comps.len(), 2);
        let largest = p.largest_unassigned_component(&adj, &pop);
        assert_eq!(largest.len(), 2); // {2,3} is larger
    }

    #[test]
    fn assign_remaining_fills_all() {
        let mut p = PartialPlan::empty(4);
        p.assign_district(&[0, 1], 1);
        p.assign_remaining(2);
        assert_eq!(p.unassigned_count, 0);
        assert!(p.assignment.iter().all(|a| a.is_some()));
        let final_plan = p.finalise();
        assert_eq!(final_plan[2], 2);
        assert_eq!(final_plan[3], 2);
    }
}
