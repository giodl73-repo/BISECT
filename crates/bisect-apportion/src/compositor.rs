//! `PfrCompositor` — the Prime-Factor Redistricting algorithm with partition cache.
//!
//! Largest-prime-first split prescription means k=34 and k=51 both start with
//! the same 17-region top-level partition of the state. The cache makes this
//! literal: that partition is computed once and reused on the second call.
//!
//! Cache key: `(region_hash, n_parts, seed)`. Two `compose` calls on the same
//! compositor share the cache, so computing k=34 first makes k=51 faster.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::Mutex;
use crate::{split_prescription, SplitStep, Partitioner, SplitError, SubGraph};

/// Stable hash of a sorted slice of global tract indices.
fn hash_region(sorted_verts: &[usize]) -> u64 {
    let mut h = DefaultHasher::new();
    sorted_verts.hash(&mut h);
    h.finish()
}

/// Cache key: the geographic region (by hash of its sorted tract indices),
/// the number of parts we're splitting it into, and the METIS seed.
type CacheKey = (u64, u32, u64);

/// Result of a PFR run.
pub struct PfrResult {
    /// `assignment[global_tract_index]` = district index ∈ [0, n_districts).
    pub assignment: Vec<u32>,
    /// Number of districts (= seat count n).
    pub n_districts: u32,
    /// Total edge cut across all levels (raw integer units from METIS).
    pub total_edge_cut: i64,
    /// Number of cache hits during this run (informational).
    pub cache_hits: u32,
}

/// Hierarchically applies a `Partitioner` using the prime factorization of `n`,
/// largest prime first, with a shared partition cache.
///
/// The cache persists across calls to `compose`, so:
/// ```text
///   compositor.compose(graph, 34, seed)   # computes 17-partition, stores it
///   compositor.compose(graph, 51, seed)   # reuses 17-partition, free
/// ```
///
/// Cache key: `(region_hash, n_parts, seed)`. Regions with the same tract set
/// and the same target part count and seed always get the same partition.
pub struct PfrCompositor<P: Partitioner> {
    pub strategy: P,
    cache: Mutex<HashMap<CacheKey, Vec<u32>>>,
}

impl<P: Partitioner> PfrCompositor<P> {
    pub fn new(strategy: P) -> Self {
        Self { strategy, cache: Mutex::new(HashMap::new()) }
    }

    /// Discard all cached partitions (e.g. after a seed change).
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }

    /// Number of partitions currently in the cache.
    pub fn cache_size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    /// Run PFR on a state graph for `n_districts` seats.
    ///
    /// `adjacency[i]`    = global neighbours of tract i.
    /// `vertex_weights[i]` = population of tract i (i64, ≥ 0).
    /// `edge_weights`    = canonical (min, max) → boundary length in metres.
    /// `base_seed`       = content-derived METIS seed for reproducibility.
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

        if n_districts == 1 {
            return Ok(PfrResult {
                assignment: vec![0u32; n_tracts],
                n_districts: 1,
                total_edge_cut: 0,
                cache_hits: 0,
            });
        }

        let all_verts: Vec<usize> = (0..n_tracts).collect();
        let mut total_edge_cut: i64 = 0;
        let mut cache_hits: u32 = 0;
        let mut leaves: Vec<(usize, Vec<usize>)> = Vec::new();

        self.recurse(
            &all_verts, n_districts, 0,
            adjacency, vertex_weights, edge_weights,
            base_seed, 0,
            &mut leaves, &mut total_edge_cut, &mut cache_hits,
        )?;

        leaves.sort_by_key(|(id, _)| *id);

        let mut assignment = vec![0u32; n_tracts];
        for (district_id, verts) in &leaves {
            for &g in verts {
                assignment[g] = *district_id as u32;
            }
        }

        debug_assert_eq!(leaves.len(), n_districts as usize);
        Ok(PfrResult { assignment, n_districts, total_edge_cut, cache_hits })
    }

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
        cache_hits: &mut u32,
    ) -> Result<(), SplitError> {
        if k == 1 {
            leaves.push((base_district_id, region_verts.to_vec()));
            return Ok(());
        }

        let step = split_prescription(k);
        let n_parts = step.parts();

        // Seed for this depth level — same depth always produces the same seed
        // regardless of k, so k=34 and k=51 at depth=0 see the same METIS seed.
        let level_seed = base_seed.map(|s| {
            s.wrapping_mul(0x9e37_79b9_7f4a_7c15).wrapping_add(depth as u64)
        });
        let seed_key = level_seed.unwrap_or(0);

        // Sort region verts for stable hashing (SubGraph::build also sorts).
        let mut sorted_verts = region_verts.to_vec();
        sorted_verts.sort_unstable();

        let region_hash = hash_region(&sorted_verts);
        let cache_key: CacheKey = (region_hash, n_parts, seed_key);

        // Check cache.
        let cached = self.cache.lock().unwrap().get(&cache_key).cloned();

        let part_assignment = if let Some(hit) = cached {
            *cache_hits += 1;
            hit
        } else {
            // Build subgraph and call partitioner.
            let subgraph = SubGraph::build(
                region_verts, adjacency, vertex_weights, edge_weights,
            )?;

            let assignment = match &step {
                SplitStep::Uniform { parts, .. } => {
                    self.strategy.split(&subgraph, *parts, level_seed)?
                }
                SplitStep::Binary { k_left, k_right } => {
                    let total = (*k_left + *k_right) as f32;
                    let fracs = [*k_left as f32 / total, *k_right as f32 / total];
                    self.strategy.split_weighted(&subgraph, &fracs, level_seed)?
                }
            };

            // Accumulate edge cut.
            let mut ec: i64 = 0;
            for local in 0..subgraph.n_vertices() {
                let start = subgraph.xadj[local] as usize;
                let end   = subgraph.xadj[local + 1] as usize;
                for (edge_idx, &nb) in subgraph.adjncy[start..end].iter().enumerate() {
                    if assignment[local] != assignment[nb as usize] {
                        ec += subgraph.adjwgt.as_ref()
                            .map(|ew| ew[start + edge_idx] as i64)
                            .unwrap_or(1);
                    }
                }
            }
            *total_ec += ec / 2;

            // Store in cache.
            self.cache.lock().unwrap().insert(cache_key, assignment.clone());
            assignment
        };

        // Group sorted verts by their assigned part.
        let mut sub_regions: Vec<Vec<usize>> = vec![Vec::new(); n_parts as usize];
        for (local_idx, &global_v) in sorted_verts.iter().enumerate() {
            sub_regions[part_assignment[local_idx] as usize].push(global_v);
        }

        // Recurse into each sub-region.
        let mut next_base = base_district_id;
        for (i, sub_verts) in sub_regions.into_iter().enumerate() {
            let sub_k = step.sub_k(i);
            self.recurse(
                &sub_verts, sub_k, next_base,
                adjacency, vertex_weights, edge_weights,
                base_seed, depth + 1,
                leaves, total_ec, cache_hits,
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
        assert_eq!(r.cache_hits, 0);
    }

    #[test]
    fn test_n4_four_districts() {
        let (adj, vw, ew) = line_graph(8);
        let c = PfrCompositor::new(MetisPartitioner::default());
        let r = c.compose(&adj, &vw, &ew, 4, Some(42)).unwrap();
        assert_eq!(r.n_districts, 4);
        let assigned: std::collections::HashSet<u32> = r.assignment.iter().cloned().collect();
        assert_eq!(assigned.len(), 4);
    }

    #[test]
    fn test_n6_two_levels() {
        let (adj, vw, ew) = line_graph(12);
        let c = PfrCompositor::new(MetisPartitioner::default());
        let r = c.compose(&adj, &vw, &ew, 6, Some(1)).unwrap();
        assert_eq!(r.n_districts, 6);
    }

    #[test]
    fn test_cache_reuse_across_seat_counts() {
        // k=34 and k=51 share factor 17: same top-level 17-partition.
        // After composing k=34, composing k=51 should get cache_hits > 0.
        let (adj, vw, ew) = line_graph(34); // 34 tracts for clean splits
        let c = PfrCompositor::new(MetisPartitioner::default());

        let r34 = c.compose(&adj, &vw, &ew, 34, Some(7)).unwrap();
        assert_eq!(r34.n_districts, 34);
        assert_eq!(r34.cache_hits, 0); // cold cache

        let cache_after_34 = c.cache_size();
        assert!(cache_after_34 > 0, "cache should have entries after k=34");

        let r51 = c.compose(&adj, &vw, &ew, 51, Some(7)).unwrap();
        assert_eq!(r51.n_districts, 51);
        // The top-level 17-partition (and deeper 17-internal splits) should be
        // reused from k=34's run.
        assert!(r51.cache_hits > 0,
            "k=51 should reuse cached partitions from k=34 (hits={})", r51.cache_hits);
    }

    #[test]
    fn test_cache_clear() {
        let (adj, vw, ew) = line_graph(12);
        let c = PfrCompositor::new(MetisPartitioner::default());
        c.compose(&adj, &vw, &ew, 6, Some(1)).unwrap();
        assert!(c.cache_size() > 0);
        c.clear_cache();
        assert_eq!(c.cache_size(), 0);
    }

    #[test]
    fn test_same_result_second_call() {
        // Calling compose twice with same k and seed should produce identical results.
        let (adj, vw, ew) = line_graph(12);
        let c = PfrCompositor::new(MetisPartitioner::default());
        let r1 = c.compose(&adj, &vw, &ew, 6, Some(99)).unwrap();
        let r2 = c.compose(&adj, &vw, &ew, 6, Some(99)).unwrap();
        assert_eq!(r1.assignment, r2.assignment);
        assert!(r2.cache_hits > 0, "second identical call should be fully cached");
    }
}
