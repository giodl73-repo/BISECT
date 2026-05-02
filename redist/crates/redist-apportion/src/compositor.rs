//! `PfrCompositor` — the Prime-Factor Redistricting algorithm.
//!
//! Given a census-tract graph and seat count `n`, hierarchically applies a
//! `SplitStrategy` following the canonical prime factorization sequence of `n`.
//!
//! The output is a global assignment vector: `assignment[tract_index]` ∈ [0, n).

use std::collections::HashMap;
use crate::{split_prescription, SplitStep, Partitioner, SplitError, SubGraph};

/// Result of a PFR run.
pub struct PfrResult {
    /// `assignment[global_tract_index]` = district index ∈ [0, n_districts).
    pub assignment: Vec<u32>,
    /// Number of districts (= seat count n).
    pub n_districts: u32,
    /// Total METIS edge cut summed across all levels (in raw integer units).
    pub total_edge_cut: i64,
}

/// Hierarchically applies a `SplitStrategy` using the prime factorization of `n`.
///
/// ## Algorithm
///
/// 1. Compute `F(n) = [p1, p2, ..., pd]` — the canonical prime factorization
///    sequence (smallest prime first, with repetition).
/// 2. Start with one region = the whole state.
/// 3. At each level `l`, split every current region into `F(n)[l]` sub-regions
///    using the `SplitStrategy`.
/// 4. After `d` levels, there are exactly `n = p1 * p2 * ... * pd` regions.
///
/// Each level receives a deterministic seed derived from `base_seed` and the
/// level index, so runs are reproducible.
pub struct PfrCompositor<P: Partitioner> {
    pub strategy: P,
}

impl<P: Partitioner> PfrCompositor<P> {
    pub fn new(strategy: P) -> Self {
        Self { strategy }
    }

    /// Run PFR on a state graph for `n_districts` seats.
    ///
    /// `adjacency[i]` = global neighbours of tract i.
    /// `vertex_weights[i]` = population of tract i.
    /// `edge_weights` maps canonical (min, max) → boundary length in metres.
    /// `base_seed` drives METIS RNG; use a content-derived seed for reproducibility.
    /// Run PFR on a state graph for `n_districts` seats.
    pub fn compose(
        &self,
        adjacency: &[Vec<usize>],
        vertex_weights: &[i64],
        edge_weights: &HashMap<(usize, usize), f64>,
        n_districts: u32,
        base_seed: Option<u64>,
    ) -> Result<PfrResult, SplitError> {
        let n_tracts = adjacency.len();
        if n_districts == 0 { return Err(SplitError::ZeroParts); }

        let mut total_edge_cut: i64 = 0;
        let all_verts: Vec<usize> = (0..n_tracts).collect();

        // Collect leaf regions: (district_index, global_tract_indices)
        let mut leaves: Vec<(usize, Vec<usize>)> = Vec::new();

        self.recurse(
            &all_verts, n_districts, 0,
            adjacency, vertex_weights, edge_weights,
            base_seed, 0,
            &mut leaves, &mut total_edge_cut,
        )?;

        // Sort by district index (recursion fills in BFS order, already correct)
        leaves.sort_by_key(|(id, _)| *id);

        let mut assignment = vec![0u32; n_tracts];
        for (district_id, verts) in &leaves {
            for &g in verts {
                assignment[g] = *district_id as u32;
            }
        }

        debug_assert_eq!(leaves.len(), n_districts as usize);
        Ok(PfrResult { assignment, n_districts, total_edge_cut })
    }

    /// Recursive helper: split `region_verts` into `k` districts.
    /// `base_district_id` is the first district ID to assign to this region's leaves.
    /// Appends (district_id, tract_list) to `leaves`.
    fn recurse(
        &self,
        region_verts: &[usize],
        k: u32,
        base_district_id: usize,
        adjacency: &[Vec<usize>],
        vertex_weights: &[i64],
        edge_weights: &HashMap<(usize, usize), f64>,
        base_seed: Option<u64>,
        depth: u32,
        leaves: &mut Vec<(usize, Vec<usize>)>,
        total_ec: &mut i64,
    ) -> Result<(), SplitError> {
        if k == 1 {
            leaves.push((base_district_id, region_verts.to_vec()));
            return Ok(());
        }

        let step = split_prescription(k);

        // Per-depth seed to avoid correlated METIS calls across levels.
        let level_seed = base_seed.map(|s| {
            s.wrapping_mul(0x9e37_79b9_7f4a_7c15).wrapping_add(depth as u64)
        });

        let subgraph = SubGraph::build(region_verts, adjacency, vertex_weights, edge_weights)?;

        let part_assignment = match &step {
            SplitStep::Uniform { parts, .. } => {
                self.strategy.split(&subgraph, *parts, level_seed)?
            }
            SplitStep::Binary { k_left, k_right } => {
                let total = (k_left + k_right) as f32;
                let fracs = [*k_left as f32 / total, *k_right as f32 / total];
                self.strategy.split_weighted(&subgraph, &fracs, level_seed)?
            }
        };

        // Accumulate edge cut for this level.
        let mut ec: i64 = 0;
        for local in 0..subgraph.n_vertices() {
            let start = subgraph.xadj[local] as usize;
            let end   = subgraph.xadj[local + 1] as usize;
            for (edge_idx, &nb) in subgraph.adjncy[start..end].iter().enumerate() {
                if part_assignment[local] != part_assignment[nb as usize] {
                    ec += subgraph.adjwgt.as_ref()
                        .map(|ew| ew[start + edge_idx] as i64)
                        .unwrap_or(1);
                }
            }
        }
        *total_ec += ec / 2; // each edge counted twice

        // Group global tract indices by sub-region.
        let parts = step.parts() as usize;
        let mut sub_regions: Vec<Vec<usize>> = vec![Vec::new(); parts];
        for (local, &part_id) in part_assignment.iter().enumerate() {
            sub_regions[part_id as usize].push(subgraph.global_ids[local]);
        }

        // Recurse: district IDs are laid out in BFS order.
        let mut next_base = base_district_id;
        for (i, sub_verts) in sub_regions.into_iter().enumerate() {
            let sub_k = step.sub_k(i);
            self.recurse(
                &sub_verts, sub_k, next_base,
                adjacency, vertex_weights, edge_weights,
                base_seed, depth + 1,
                leaves, total_ec,
            )?;
            next_base += sub_k as usize;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split::MetisPartitioner;

    fn line_graph(n: usize) -> (Vec<Vec<usize>>, Vec<i64>, HashMap<(usize, usize), f64>) {
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nb = Vec::new();
            if i > 0 { nb.push(i - 1); }
            if i + 1 < n { nb.push(i + 1); }
            nb
        }).collect();
        let vw = vec![100i64; n];
        let ew: HashMap<_, _> = (0..n-1).map(|i| ((i, i+1), 1000.0)).collect();
        (adj, vw, ew)
    }

    #[test]
    fn test_n1_trivial() {
        let (adj, vw, ew) = line_graph(5);
        let c = PfrCompositor::new(MetisPartitioner::default());
        let r = c.compose(&adj, &vw, &ew, 1, Some(42)).unwrap();
        assert_eq!(r.n_districts, 1);
        assert!(r.assignment.iter().all(|&a| a == 0));
    }

    #[test]
    fn test_n4_four_districts() {
        let (adj, vw, ew) = line_graph(8);
        let c = PfrCompositor::new(MetisPartitioner::default());
        let r = c.compose(&adj, &vw, &ew, 4, Some(42)).unwrap();
        assert_eq!(r.n_districts, 4);
        let assigned: std::collections::HashSet<u32> = r.assignment.iter().cloned().collect();
        assert_eq!(assigned.len(), 4, "should have exactly 4 distinct district IDs");
        let total: u32 = r.assignment.iter().map(|_| 1).sum();
        assert_eq!(total, 8);
    }

    #[test]
    fn test_n6_two_levels() {
        let (adj, vw, ew) = line_graph(12);
        let c = PfrCompositor::new(MetisPartitioner::default());
        let r = c.compose(&adj, &vw, &ew, 6, Some(1)).unwrap();
        assert_eq!(r.n_districts, 6);
    }
}
