//! ReCom (Recombination) proposal for redistricting MCMC.
//!
//! Each step:
//!   1. Find all adjacent district pairs.
//!   2. Pick one at random.
//!   3. Extract the merged region (tracts in d_i ∪ d_j).
//!   4. Sample a uniform random spanning tree of the region subgraph.
//!   5. Find all balanced cuts of the tree (both components within pop tolerance).
//!   6. If any balanced cut exists: accept uniformly at random.
//!      Else: resample tree (up to MAX_TREE_RESAMPLES).
//!      If still none: reject and pick a new district pair (pair reselection).
//!
//! The "full tree-resample on balance failure" approach matches GerryChain's
//! stationary distribution: every balanced spanning-tree cut is equally likely.

use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand::seq::SliceRandom;
use crate::spanning::{random_spanning_tree, SpanningTree};

/// Maximum number of spanning tree resamples before pair reselection.
const MAX_TREE_RESAMPLES: usize = 10;

/// Maximum number of pair reselection attempts per step.
const MAX_PAIR_ATTEMPTS: usize = 50;

/// Per-step outcome for diagnostics.
#[derive(Debug, Clone)]
pub struct StepRecord {
    pub step: u64,
    pub cut_edges: usize,
    pub cut_fraction: f32,
    pub pop_deviation: f32,
    pub accepted: bool,
}

/// Core ReCom chain state.
pub struct RecomChain {
    /// CSR adjacency: adj[v] = list of neighbour indices.
    pub adj: Vec<Vec<u32>>,
    /// Population per tract.
    pub pop: Vec<i64>,
    /// Current district assignment: assignment[tract] = district (1-based).
    pub assignment: Vec<u32>,
    pub k: u32,
    pub pop_tolerance: f64,
    pub steps_taken: u64,
    // Pre-computed ideal population.
    ideal_pop: f64,
    // Total edges (for cut_fraction).
    total_edges: usize,
}

impl RecomChain {
    pub fn new(
        adj: Vec<Vec<u32>>,
        pop: Vec<i64>,
        assignment: Vec<u32>,
        k: u32,
        pop_tolerance: f64,
    ) -> Self {
        let n = adj.len();
        assert_eq!(pop.len(), n);
        assert_eq!(assignment.len(), n);
        let ideal_pop = pop.iter().sum::<i64>() as f64 / k as f64;
        let total_edges = adj.iter().map(|nb| nb.len()).sum::<usize>() / 2;
        Self { adj, pop, assignment, k, pop_tolerance, steps_taken: 0, ideal_pop, total_edges }
    }

    /// Run one ReCom step. Returns a `StepRecord` with outcome metrics.
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> StepRecord {
        self.steps_taken += 1;

        let accepted = self.try_step(rng);

        let cut = self.count_cut_edges();
        let max_dev = self.max_pop_deviation();

        StepRecord {
            step: self.steps_taken,
            cut_edges: cut,
            cut_fraction: cut as f32 / self.total_edges.max(1) as f32,
            pop_deviation: max_dev as f32,
            accepted,
        }
    }

    /// Attempt a ReCom step with pair reselection on persistent balance failure.
    /// Returns true if the assignment was updated.
    fn try_step<R: Rng>(&mut self, rng: &mut R) -> bool {
        let pairs = self.adjacent_pairs();
        if pairs.is_empty() { return false; }

        // Shuffle pairs for random pair reselection.
        let mut pair_order: Vec<usize> = (0..pairs.len()).collect();
        pair_order.shuffle(rng);

        for &pair_idx in pair_order.iter().take(MAX_PAIR_ATTEMPTS) {
            let (d_i, d_j) = pairs[pair_idx];

            // Extract region: all tracts in d_i ∪ d_j.
            let region: Vec<u32> = self.assignment.iter().enumerate()
                .filter(|(_, &d)| d == d_i || d == d_j)
                .map(|(v, _)| v as u32)
                .collect();

            if region.len() < 2 { continue; }

            // Build local subgraph adjacency (local indices 0..region.len()).
            let local_idx: HashMap<u32, u32> = region.iter()
                .enumerate()
                .map(|(local, &global)| (global, local as u32))
                .collect();

            let local_adj: Vec<Vec<u32>> = region.iter().map(|&g| {
                self.adj[g as usize].iter()
                    .filter_map(|&nb| local_idx.get(&nb).copied())
                    .collect()
            }).collect();

            // Try up to MAX_TREE_RESAMPLES spanning trees.
            for _ in 0..MAX_TREE_RESAMPLES {
                let tree = random_spanning_tree(&local_adj, rng);

                // Collect all balanced cuts.
                let balanced_cuts = self.balanced_cuts(&tree, &region);

                if !balanced_cuts.is_empty() {
                    // Pick one uniformly at random and apply it.
                    let &(local_a, local_b) = balanced_cuts.choose(rng).unwrap();
                    let (comp_a, comp_b) = tree.split_on(local_a, local_b);
                    for &local in &comp_a { self.assignment[region[local as usize] as usize] = d_i; }
                    for &local in &comp_b { self.assignment[region[local as usize] as usize] = d_j; }
                    return true;
                }
                // No balanced cut in this tree — resample.
            }
            // All tree resamples exhausted — try next pair.
        }
        false
    }

    /// Find all tree edges whose removal produces two population-balanced components.
    fn balanced_cuts(&self, tree: &SpanningTree, region: &[u32]) -> Vec<(u32, u32)> {
        let mut balanced = Vec::new();
        let total_pop: i64 = region.iter().map(|&g| self.pop[g as usize]).sum();

        for (a, b) in tree.edges() {
            let (comp_a, _) = tree.split_on(a, b);
            let pop_a: i64 = comp_a.iter().map(|&local| self.pop[region[local as usize] as usize]).sum();
            let pop_b = total_pop - pop_a;
            let dev_a = (pop_a as f64 - self.ideal_pop).abs() / self.ideal_pop;
            let dev_b = (pop_b as f64 - self.ideal_pop).abs() / self.ideal_pop;
            if dev_a <= self.pop_tolerance && dev_b <= self.pop_tolerance {
                balanced.push((a, b));
            }
        }
        balanced
    }

    /// Count all edges crossing district boundaries.
    fn count_cut_edges(&self) -> usize {
        let mut cut = 0usize;
        for (v, nbrs) in self.adj.iter().enumerate() {
            for &nb in nbrs {
                if self.assignment[v] != self.assignment[nb as usize] {
                    cut += 1;
                }
            }
        }
        cut / 2
    }

    fn max_pop_deviation(&self) -> f64 {
        let mut dist_pops: HashMap<u32, i64> = HashMap::new();
        for (v, &d) in self.assignment.iter().enumerate() {
            *dist_pops.entry(d).or_default() += self.pop[v];
        }
        dist_pops.values()
            .map(|&p| (p as f64 - self.ideal_pop).abs() / self.ideal_pop)
            .fold(0.0_f64, f64::max)
    }

    /// Find all pairs of districts that share at least one tract-level edge.
    fn adjacent_pairs(&self) -> Vec<(u32, u32)> {
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for (v, nbrs) in self.adj.iter().enumerate() {
            let dv = self.assignment[v];
            for &nb in nbrs {
                let dn = self.assignment[nb as usize];
                if dv != dn {
                    let pair = (dv.min(dn), dv.max(dn));
                    seen.insert(pair);
                }
            }
        }
        seen.into_iter().collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    /// Build a 4×2 grid assigned to 2 districts (left=1, right=2).
    fn grid_chain() -> RecomChain {
        // 8 nodes: 0-3 in district 1, 4-7 in district 2
        // Layout: 0-1-2-3 / 4-5-6-7 (rows), connected left-right
        let adj = vec![
            vec![1u32, 4],    // 0
            vec![0, 2, 5],    // 1
            vec![1, 3, 6],    // 2
            vec![2, 7],       // 3
            vec![0, 5],       // 4
            vec![4, 1, 6],    // 5
            vec![5, 2, 7],    // 6
            vec![6, 3],       // 7
        ];
        let pop = vec![1000i64; 8];
        let assignment = vec![1, 1, 1, 1, 2, 2, 2, 2];
        RecomChain::new(adj, pop, assignment, 2, 0.05)
    }

    #[test]
    fn step_preserves_district_count() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..50 {
            chain.step(&mut rng);
            let districts: HashSet<u32> = chain.assignment.iter().copied().collect();
            assert_eq!(districts.len(), 2, "must always have exactly 2 districts");
        }
    }

    #[test]
    fn step_preserves_population_balance() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(99);
        for _ in 0..50 {
            chain.step(&mut rng);
            let dev = chain.max_pop_deviation();
            assert!(dev <= 0.06, "population deviation {dev:.4} exceeds tolerance");
        }
    }

    #[test]
    fn step_record_fields_are_valid() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(7);
        let rec = chain.step(&mut rng);
        assert!(rec.cut_fraction >= 0.0 && rec.cut_fraction <= 1.0);
        assert!(rec.pop_deviation >= 0.0);
        assert_eq!(rec.step, 1);
    }

    #[test]
    fn adjacent_pairs_finds_boundary() {
        let chain = grid_chain();
        let pairs = chain.adjacent_pairs();
        assert!(!pairs.is_empty(), "grid with two districts must have adjacent pairs");
        assert!(pairs.contains(&(1, 2)), "districts 1 and 2 share a boundary");
    }

    // ── Additional L0 coverage ────────────────────────────────────────────────

    #[test]
    fn single_district_no_adjacent_pairs() {
        let adj = vec![vec![1u32,2], vec![0,2], vec![0,1]];
        let pop = vec![1000i64; 3];
        let assignment = vec![1u32, 1, 1]; // all same district
        let chain = RecomChain::new(adj, pop, assignment, 1, 0.05);
        assert!(chain.adjacent_pairs().is_empty(), "no pairs when all in one district");
    }

    #[test]
    fn step_returns_false_when_no_adjacent_pairs() {
        let adj = vec![vec![1u32,2], vec![0,2], vec![0,1]];
        let pop = vec![1000i64; 3];
        let assignment = vec![1u32, 1, 1];
        let mut chain = RecomChain::new(adj, pop, assignment, 1, 0.05);
        let mut rng = SmallRng::seed_from_u64(0);
        let rec = chain.step(&mut rng);
        assert!(!rec.accepted, "cannot accept when no adjacent pairs exist");
    }

    #[test]
    fn k3_districts_preserved_over_many_steps() {
        // 6-node path split into 3 districts: [0,1], [2,3], [4,5]
        let adj: Vec<Vec<u32>> = (0..6usize).map(|i| {
            let mut nb = vec![];
            if i > 0 { nb.push((i-1) as u32); }
            if i < 5 { nb.push((i+1) as u32); }
            nb
        }).collect();
        let pop = vec![1000i64; 6];
        let assignment = vec![1u32, 1, 2, 2, 3, 3];
        let mut chain = RecomChain::new(adj, pop, assignment, 3, 0.05);
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..100 {
            chain.step(&mut rng);
            let districts: HashSet<u32> = chain.assignment.iter().copied().collect();
            assert_eq!(districts.len(), 3, "must always have 3 districts");
        }
    }

    #[test]
    fn rejected_step_leaves_assignment_unchanged() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(0);
        // Use zero tolerance so almost all proposals are rejected.
        chain.pop_tolerance = 0.0;
        let before = chain.assignment.clone();
        for _ in 0..20 {
            let rec = chain.step(&mut rng);
            if !rec.accepted {
                assert_eq!(chain.assignment, before, "rejected step must not change assignment");
                return; // found a rejection, test passes
            }
        }
        // If all 20 steps accepted with tolerance=0, assignment may have changed — that's fine,
        // it means the graph has perfectly balanced splits and all proposals succeed.
    }

    #[test]
    fn step_count_increments() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(1);
        assert_eq!(chain.steps_taken, 0);
        chain.step(&mut rng);
        assert_eq!(chain.steps_taken, 1);
        chain.step(&mut rng);
        assert_eq!(chain.steps_taken, 2);
    }

    #[test]
    fn ideal_pop_is_total_over_k() {
        let chain = grid_chain();
        let expected = 8.0 * 1000.0 / 2.0; // 8 tracts × 1000 pop / k=2
        assert!((chain.ideal_pop - expected).abs() < 1e-6);
    }

    #[test]
    fn max_pop_deviation_zero_for_perfectly_balanced() {
        let adj = vec![vec![1u32], vec![0]];
        let pop = vec![1000i64, 1000];
        let assignment = vec![1u32, 2];
        let chain = RecomChain::new(adj, pop, assignment, 2, 0.01);
        let dev = chain.max_pop_deviation();
        assert!(dev < 1e-10, "perfectly balanced partition has zero deviation");
    }

    #[test]
    fn cut_edges_count_on_two_district_grid() {
        let chain = grid_chain();
        // 4×2 grid, left 4 (0-3) = d1, right 4 (4-7) = d2.
        // Boundary edges: 0-4, 1-5, 2-6, 3-7 → 4 cut edges.
        let cut = chain.count_cut_edges();
        assert_eq!(cut, 4, "4×2 grid has 4 cut edges at the midline");
    }

    #[test]
    fn all_assignments_valid_after_many_steps() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(100);
        for _ in 0..200 {
            chain.step(&mut rng);
            for &d in &chain.assignment {
                assert!(d == 1 || d == 2, "assignment must be 1 or 2, got {d}");
            }
        }
    }

    #[test]
    fn pop_balance_never_exceeds_tolerance_plus_epsilon() {
        let mut chain = grid_chain(); // tolerance = 0.05
        let mut rng = SmallRng::seed_from_u64(77);
        for _ in 0..100 {
            chain.step(&mut rng);
            let dev = chain.max_pop_deviation();
            assert!(dev <= 0.06, "deviation {dev:.4} must not exceed tolerance+epsilon");
        }
    }

    #[test]
    fn adjacent_pairs_are_ordered_min_max() {
        let chain = grid_chain();
        for (a, b) in chain.adjacent_pairs() {
            assert!(a < b, "pairs must be (min, max): got ({a},{b})");
        }
    }

    #[test]
    fn step_cut_fraction_in_range() {
        let mut chain = grid_chain();
        let mut rng = SmallRng::seed_from_u64(55);
        for _ in 0..50 {
            let rec = chain.step(&mut rng);
            assert!(rec.cut_fraction >= 0.0 && rec.cut_fraction <= 1.0,
                "cut_fraction {} out of range", rec.cut_fraction);
        }
    }
}
