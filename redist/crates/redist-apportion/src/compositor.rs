//! `PfrCompositor` — the Prime-Factor Redistricting algorithm.
//!
//! Given a census-tract graph and seat count `n`, hierarchically applies a
//! `SplitStrategy` following the canonical prime factorization sequence of `n`.
//!
//! The output is a global assignment vector: `assignment[tract_index]` ∈ [0, n).

use std::collections::HashMap;
use crate::{prime_factor_sequence, SplitError, SplitStrategy, SubGraph};

/// Result of a PFR run.
pub struct PfrResult {
    /// `assignment[global_tract_index]` = district index ∈ [0, n_districts).
    pub assignment: Vec<u32>,
    /// Number of districts (= seat count n).
    pub n_districts: u32,
    /// The prime factorization sequence used, e.g. [2, 2, 13] for n=52.
    pub factor_sequence: Vec<u32>,
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
pub struct PfrCompositor<S: SplitStrategy> {
    pub strategy: S,
}

impl<S: SplitStrategy> PfrCompositor<S> {
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }

    /// Run PFR on a state graph for `n_districts` seats.
    ///
    /// `adjacency[i]` = global neighbours of tract i.
    /// `vertex_weights[i]` = population of tract i.
    /// `edge_weights` maps canonical (min, max) → boundary length in metres.
    /// `base_seed` drives METIS RNG; use a content-derived seed for reproducibility.
    pub fn compose(
        &self,
        adjacency: &[Vec<usize>],
        vertex_weights: &[i64],
        edge_weights: &HashMap<(usize, usize), f64>,
        n_districts: u32,
        base_seed: Option<u64>,
    ) -> Result<PfrResult, SplitError> {
        let n_tracts = adjacency.len();
        let factor_sequence = prime_factor_sequence(n_districts);

        // n=1: trivial — single at-large district.
        if factor_sequence.is_empty() {
            return Ok(PfrResult {
                assignment: vec![0u32; n_tracts],
                n_districts: 1,
                factor_sequence,
                total_edge_cut: 0,
            });
        }

        // regions: each element is the list of global tract indices in that region.
        let mut regions: Vec<Vec<usize>> = vec![(0..n_tracts).collect()];
        let mut total_edge_cut: i64 = 0;

        for (level, &p) in factor_sequence.iter().enumerate() {
            // Derive a per-level seed to avoid correlated METIS results across levels.
            let level_seed = base_seed.map(|s| {
                // Mix level into seed: FNV-style multiply+xor
                s.wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    .wrapping_add(level as u64)
            });

            let mut next_regions: Vec<Vec<usize>> = Vec::with_capacity(regions.len() * p as usize);

            for region_verts in &regions {
                let subgraph = SubGraph::build(
                    region_verts,
                    adjacency,
                    vertex_weights,
                    edge_weights,
                )?;

                let assignment = self.strategy.split(&subgraph, p, level_seed)?;

                // Accumulate edge cut: count edges crossing parts.
                for (local, neighbors) in subgraph.xadj.windows(2).enumerate() {
                    let (start, end) = (neighbors[0] as usize, neighbors[1] as usize);
                    for &nb_local in &subgraph.adjncy[start..end] {
                        if assignment[local] != assignment[nb_local as usize] {
                            if let Some(ref ew) = subgraph.adjwgt {
                                total_edge_cut += ew[start] as i64;
                            } else {
                                total_edge_cut += 1;
                            }
                        }
                    }
                }
                // Divide by 2: each edge counted from both endpoints.
                total_edge_cut /= 2;

                // Group global tract indices by their assigned sub-region.
                let mut sub_regions: Vec<Vec<usize>> = vec![Vec::new(); p as usize];
                for (local, &part_id) in assignment.iter().enumerate() {
                    sub_regions[part_id as usize].push(subgraph.global_ids[local]);
                }
                next_regions.extend(sub_regions);
            }

            regions = next_regions;
        }

        debug_assert_eq!(regions.len(), n_districts as usize,
            "PFR produced {} regions, expected {}", regions.len(), n_districts);

        // Build global assignment vector.
        let mut assignment = vec![0u32; n_tracts];
        for (district_id, verts) in regions.iter().enumerate() {
            for &g in verts {
                assignment[g] = district_id as u32;
            }
        }

        Ok(PfrResult { assignment, n_districts, factor_sequence, total_edge_cut })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split::MetisKwaySplit;

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
        let c = PfrCompositor::new(MetisKwaySplit::default());
        let r = c.compose(&adj, &vw, &ew, 1, Some(42)).unwrap();
        assert_eq!(r.n_districts, 1);
        assert!(r.assignment.iter().all(|&a| a == 0));
    }

    #[test]
    fn test_n4_four_districts() {
        let (adj, vw, ew) = line_graph(8);
        let c = PfrCompositor::new(MetisKwaySplit::default());
        let r = c.compose(&adj, &vw, &ew, 4, Some(42)).unwrap();
        assert_eq!(r.n_districts, 4);
        assert_eq!(r.factor_sequence, vec![2, 2]);
        let assigned: std::collections::HashSet<u32> = r.assignment.iter().cloned().collect();
        assert_eq!(assigned.len(), 4, "should have exactly 4 distinct district IDs");
        let total: u32 = r.assignment.iter().map(|_| 1).sum();
        assert_eq!(total, 8);
    }

    #[test]
    fn test_n6_two_levels() {
        let (adj, vw, ew) = line_graph(12);
        let c = PfrCompositor::new(MetisKwaySplit::default());
        let r = c.compose(&adj, &vw, &ew, 6, Some(1)).unwrap();
        assert_eq!(r.n_districts, 6);
        assert_eq!(r.factor_sequence, vec![2, 3]);
    }
}
