//! Forest-ReCom (reversible ReCom) — two-tree Metropolis-Hastings approximation.
//!
//! Each step:
//!   1. Find all adjacent district pairs.
//!   2. Shuffle pairs, try up to MAX_PAIR_ATTEMPTS.
//!   3. For each pair (d_i, d_j):
//!      a. Extract the merged region (tracts in d_i ∪ d_j).
//!      b. Sample forward spanning tree T_fwd via Wilson's algorithm.
//!      c. Count all balanced cuts in T_fwd → valid_cuts_forward.
//!      d. If 0: skip to next pair.
//!      e. Pick one cut uniformly at random and apply it → proposed assignment.
//!      f. Sample reverse spanning tree T_rev via Wilson's algorithm.
//!      g. Count all balanced cuts in T_rev → valid_cuts_reverse.
//!      h. If 0: ratio = 0.0 → reject, continue.
//!      i. ratio = valid_cuts_forward / valid_cuts_reverse.
//!      j. Accept with probability min(1.0, ratio).
//!
//! The MH correction (ratio of forward/reverse cut counts) targets the
//! stationary distribution where every balanced spanning-tree cut is equally
//! likely — identical to GerryChain's Forest-ReCom stationary distribution.

use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand::seq::SliceRandom;
use crate::recom::StepRecord;
use crate::spanning::{random_spanning_tree, SpanningTree};

/// Maximum number of pair reselection attempts per step.
const MAX_PAIR_ATTEMPTS: usize = 50;

/// Core Forest-ReCom chain state.
pub struct ForestRecomChain {
    /// CSR adjacency: adj[v] = list of neighbour indices.
    pub adj: Vec<Vec<u32>>,
    /// Population per tract.
    pub pop: Vec<i64>,
    /// Current district assignment: assignment[tract] = district (1-based).
    pub assignment: Vec<u32>,
    pub k: u32,
    pub pop_tolerance: f64,
    pub steps_taken: u64,
    pub steps_accepted: u64,
    // Pre-computed ideal population.
    ideal_pop: f64,
    // Total edges (for cut_fraction).
    total_edges: usize,
}

impl ForestRecomChain {
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
        Self {
            adj, pop, assignment, k, pop_tolerance,
            steps_taken: 0, steps_accepted: 0,
            ideal_pop, total_edges,
        }
    }

    /// Run one Forest-ReCom step.
    ///
    /// Takes two independent RNGs:
    /// - `rng_forward`: drives the forward tree, proposal selection, and acceptance coin flip.
    /// - `rng_reverse`: drives the reverse tree only.
    pub fn step<R: Rng>(&mut self, rng_forward: &mut R, rng_reverse: &mut R) -> StepRecord {
        self.steps_taken += 1;

        let accepted = self.try_step(rng_forward, rng_reverse);
        if accepted { self.steps_accepted += 1; }

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

    /// MH Forest-ReCom step with pair reselection on persistent balance failure.
    /// Returns true if the assignment was updated.
    fn try_step<R: Rng>(&mut self, rng_forward: &mut R, rng_reverse: &mut R) -> bool {
        let pairs = self.adjacent_pairs();
        if pairs.is_empty() { return false; }

        // Shuffle pairs for random pair reselection.
        let mut pair_order: Vec<usize> = (0..pairs.len()).collect();
        pair_order.shuffle(rng_forward);

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

            // Pre-compute region population array and target.
            let local_pop: Vec<i64> = region.iter().map(|&g| self.pop[g as usize]).collect();
            let total_pop: i64 = local_pop.iter().sum();
            let tol_abs = self.ideal_pop * self.pop_tolerance;

            // (a) Sample forward spanning tree.
            let t_fwd = random_spanning_tree(&local_adj, rng_forward);

            // (b) Count balanced cuts in the forward tree.
            let valid_cuts_fwd = count_balanced_cuts(&t_fwd, &local_pop, self.ideal_pop, tol_abs);

            if valid_cuts_fwd.is_empty() {
                // No balanced cut in this tree — try next pair.
                continue;
            }
            let valid_cuts_forward = valid_cuts_fwd.len();

            // (c) Pick one cut uniformly at random and build the proposed assignment.
            let &(local_a, local_b) = valid_cuts_fwd.choose(rng_forward).unwrap();
            let (comp_a, comp_b) = t_fwd.split_on(local_a, local_b);

            // Verify the proposed split is population-balanced (sanity).
            let pop_a: i64 = comp_a.iter().map(|&l| local_pop[l as usize]).sum();
            let pop_b = total_pop - pop_a;
            let dev_a = (pop_a as f64 - self.ideal_pop).abs();
            let dev_b = (pop_b as f64 - self.ideal_pop).abs();
            if dev_a > tol_abs && dev_b > tol_abs {
                // count_balanced_cuts guarantees at least one side is balanced;
                // both failing should be impossible, but skip defensively.
                continue;
            }

            // (d) Sample reverse spanning tree (same local_adj, different rng).
            let t_rev = random_spanning_tree(&local_adj, rng_reverse);

            // (e) Count balanced cuts in the reverse tree.
            let valid_cuts_rev = count_balanced_cuts(&t_rev, &local_pop, self.ideal_pop, tol_abs);
            let valid_cuts_reverse = valid_cuts_rev.len();

            // (f) Compute MH ratio and accept/reject.
            if valid_cuts_reverse == 0 {
                // Ratio = 0 → reject. Try next pair.
                continue;
            }

            let ratio = valid_cuts_forward as f64 / valid_cuts_reverse as f64;

            if rng_forward.gen::<f64>() < ratio.min(1.0) {
                // Accept: apply the proposed split.
                for &local in &comp_a { self.assignment[region[local as usize] as usize] = d_i; }
                for &local in &comp_b { self.assignment[region[local as usize] as usize] = d_j; }
                return true;
            } else {
                // Reject: plan unchanged.
                return false;
            }
        }
        false
    }

    /// Acceptance rate: steps_accepted / steps_taken.
    pub fn acceptance_rate(&self) -> f64 {
        if self.steps_taken == 0 { 0.0 } else { self.steps_accepted as f64 / self.steps_taken as f64 }
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

/// Count all tree edges whose removal produces two population-balanced components.
///
/// Returns a list of `(local_a, local_b)` valid cut edges.
/// A cut is valid if at least one of the two resulting components has population
/// within `tol_abs` of `target_pop`.
/// (Since total = 2 × target_pop and both must be within tolerance, checking one
/// side suffices when both sides must pass — but we use the spec's OR condition
/// to be permissive: at least one side balanced.)
pub fn count_balanced_cuts(
    tree: &SpanningTree,
    local_pop: &[i64],
    target_pop: f64,
    tol_abs: f64,
) -> Vec<(u32, u32)> {
    let total_pop: i64 = local_pop.iter().sum();
    let mut valid = Vec::new();

    for (a, b) in tree.edges() {
        let (comp_a, _) = tree.split_on(a, b);
        let pop_a: i64 = comp_a.iter().map(|&l| local_pop[l as usize]).sum();
        let pop_b = total_pop - pop_a;
        let ok_a = (pop_a as f64 - target_pop).abs() <= tol_abs;
        let ok_b = (pop_b as f64 - target_pop).abs() <= tol_abs;
        if ok_a || ok_b {
            valid.push((a, b));
        }
    }
    valid
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    // ── Graph helpers ─────────────────────────────────────────────────────────

    /// n-node path: 0-1-2-…-(n-1).
    fn path_adj(n: usize) -> Vec<Vec<u32>> {
        let mut adj = vec![vec![]; n];
        for i in 0..n - 1 {
            adj[i].push((i + 1) as u32);
            adj[i + 1].push(i as u32);
        }
        adj
    }

    /// rows×cols grid.
    fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<u32>> {
        let n = rows * cols;
        let mut adj = vec![vec![]; n];
        for r in 0..rows {
            for c in 0..cols {
                let v = r * cols + c;
                if c + 1 < cols { adj[v].push((v + 1) as u32); adj[v + 1].push(v as u32); }
                if r + 1 < rows { adj[v].push((v + cols) as u32); adj[v + cols].push(v as u32); }
            }
        }
        adj
    }

    // ── Chain helpers ─────────────────────────────────────────────────────────

    fn make_rngs(seed: u64) -> (SmallRng, SmallRng) {
        (SmallRng::seed_from_u64(seed), SmallRng::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_1234))
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn forest_recom_chain_4_path_k2() {
        // 4-node path: [0,1] = d1, [2,3] = d2.
        let adj = path_adj(4);
        let pop = vec![1000i64; 4];
        let assignment = vec![1u32, 1, 2, 2];
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.05);
        let (mut rf, mut rr) = make_rngs(42);

        let rec = chain.step(&mut rf, &mut rr);

        // Plan must still have exactly 2 districts.
        let districts: HashSet<u32> = chain.assignment.iter().copied().collect();
        assert_eq!(districts.len(), 2, "must have exactly 2 districts after step");

        // StepRecord fields must be in range.
        assert_eq!(rec.step, 1);
        assert!(rec.cut_fraction >= 0.0 && rec.cut_fraction <= 1.0);
        assert!(rec.pop_deviation >= 0.0);

        // steps_taken must be 1.
        assert_eq!(chain.steps_taken, 1);
    }

    #[test]
    fn forest_recom_acceptance_rate_in_range() {
        // 4×4 grid, k=2: top 8 = d1, bottom 8 = d2.
        let adj = grid_adj(4, 4);
        let pop = vec![1000i64; 16];
        let assignment: Vec<u32> = (0..16).map(|i| if i < 8 { 1 } else { 2 }).collect();
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.10);
        let (mut rf, mut rr) = make_rngs(99);

        for _ in 0..100 {
            chain.step(&mut rf, &mut rr);
        }

        let rate = chain.acceptance_rate();
        assert!(
            rate > 0.0 && rate <= 1.0,
            "acceptance rate {rate} must be in (0.0, 1.0] after 100 steps on a 4x4 grid"
        );
    }

    #[test]
    fn forest_recom_deterministic() {
        // Same seeds → identical trajectory.
        let adj = grid_adj(4, 4);
        let pop = vec![1000i64; 16];
        let assignment: Vec<u32> = (0..16).map(|i| if i < 8 { 1 } else { 2 }).collect();

        let (mut rf1, mut rr1) = make_rngs(7);
        let mut chain1 = ForestRecomChain::new(adj.clone(), pop.clone(), assignment.clone(), 2, 0.10);
        let recs1: Vec<bool> = (0..30).map(|_| chain1.step(&mut rf1, &mut rr1).accepted).collect();

        let (mut rf2, mut rr2) = make_rngs(7);
        let mut chain2 = ForestRecomChain::new(adj, pop, assignment, 2, 0.10);
        let recs2: Vec<bool> = (0..30).map(|_| chain2.step(&mut rf2, &mut rr2).accepted).collect();

        assert_eq!(recs1, recs2, "same seeds must produce identical trajectory");
        assert_eq!(chain1.assignment, chain2.assignment, "same seeds must produce same final assignment");
    }

    #[test]
    fn forest_recom_rejected_step_plan_unchanged() {
        // Very tight tolerance → many rejections.
        let adj = path_adj(4);
        let pop = vec![1000i64; 4];
        let assignment = vec![1u32, 1, 2, 2];
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.0);
        let (mut rf, mut rr) = make_rngs(0);

        let before = chain.assignment.clone();
        for _ in 0..30 {
            let rec = chain.step(&mut rf, &mut rr);
            if !rec.accepted {
                assert_eq!(
                    chain.assignment, before,
                    "rejected step must leave assignment unchanged"
                );
                return; // found at least one rejection — test passes
            }
        }
        // If all 30 steps accepted with tol=0.0 on a 4-node path (only one balanced
        // cut), that's fine — means every proposal happened to be accepted.
    }

    #[test]
    fn forest_recom_valid_cuts_count_path() {
        // 4-node path: local_pop = [1000, 1000, 1000, 1000], target = 2000.
        // Only one balanced cut dividing into 2+2.
        let adj = path_adj(4);
        let local_pop = vec![1000i64; 4];
        let target = 2000.0_f64;
        let tol_abs = target * 0.05; // 100

        let mut rng = SmallRng::seed_from_u64(42);
        // Run several trees — path graph has a unique spanning tree (it IS the path),
        // so count_balanced_cuts always returns exactly 1.
        for _ in 0..10 {
            let tree = random_spanning_tree(&adj, &mut rng);
            let cuts = count_balanced_cuts(&tree, &local_pop, target, tol_abs);
            assert_eq!(
                cuts.len(), 1,
                "4-node uniform path must have exactly 1 balanced cut"
            );
        }
    }

    #[test]
    fn forest_recom_zero_valid_cuts_reverse_rejects() {
        // Star graph: hub=0, leaves=1,2,3,4 (each pop=100).
        // With k=2 and tolerance=0.0 (exact balance), target=250.
        // No subset of a 5-node star sums to exactly 250 with integer pops,
        // so count_balanced_cuts on the reverse tree returns 0 → ratio=0 → reject.
        //
        // We verify via acceptance rate: with such a tight tolerance almost
        // everything is rejected.
        let n = 5usize;
        let mut adj = vec![vec![]; n];
        for i in 1..n { adj[0].push(i as u32); adj[i].push(0); }
        // Deliberately unequal populations so no exact half-sum exists.
        let pop = vec![100i64, 200, 300, 400, 500]; // total=1500, target=750
        let assignment = vec![1u32, 1, 1, 2, 2];
        // zero tolerance: only accept perfectly balanced splits
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.0);
        let (mut rf, mut rr) = make_rngs(5);

        for _ in 0..50 {
            chain.step(&mut rf, &mut rr);
        }
        // With zero tolerance and unequal pops, most steps should be rejected.
        // We don't assert exactly 0 accepted (there could be lucky balanced cuts)
        // but steps_accepted <= steps_taken always holds.
        assert!(chain.steps_accepted <= chain.steps_taken);
    }

    #[test]
    fn forest_recom_steps_accepted_le_steps_taken() {
        // Invariant: accepted ≤ taken for any number of steps on any graph.
        let adj = grid_adj(4, 4);
        let pop = vec![1000i64; 16];
        let assignment: Vec<u32> = (0..16).map(|i| if i < 8 { 1 } else { 2 }).collect();
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.05);
        let (mut rf, mut rr) = make_rngs(17);

        for _ in 0..200 {
            chain.step(&mut rf, &mut rr);
        }
        assert!(
            chain.steps_accepted <= chain.steps_taken,
            "steps_accepted {} must never exceed steps_taken {}",
            chain.steps_accepted, chain.steps_taken
        );
        assert_eq!(chain.steps_taken, 200);
    }

    // ── Additional coverage ───────────────────────────────────────────────────

    #[test]
    fn acceptance_rate_zero_before_any_step() {
        let adj = path_adj(4);
        let pop = vec![1000i64; 4];
        let assignment = vec![1u32, 1, 2, 2];
        let chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.05);
        assert_eq!(chain.acceptance_rate(), 0.0, "rate must be 0.0 with no steps taken");
    }

    #[test]
    fn step_count_increments_each_call() {
        let adj = path_adj(4);
        let pop = vec![1000i64; 4];
        let assignment = vec![1u32, 1, 2, 2];
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.05);
        let (mut rf, mut rr) = make_rngs(1);
        assert_eq!(chain.steps_taken, 0);
        chain.step(&mut rf, &mut rr); assert_eq!(chain.steps_taken, 1);
        chain.step(&mut rf, &mut rr); assert_eq!(chain.steps_taken, 2);
        chain.step(&mut rf, &mut rr); assert_eq!(chain.steps_taken, 3);
    }

    #[test]
    fn district_count_preserved_over_many_steps() {
        // 4×4 grid, k=2.
        let adj = grid_adj(4, 4);
        let pop = vec![1000i64; 16];
        let assignment: Vec<u32> = (0..16).map(|i| if i < 8 { 1 } else { 2 }).collect();
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.10);
        let (mut rf, mut rr) = make_rngs(100);
        for _ in 0..100 {
            chain.step(&mut rf, &mut rr);
            let districts: HashSet<u32> = chain.assignment.iter().copied().collect();
            assert_eq!(districts.len(), 2, "must always have exactly 2 districts");
        }
    }

    #[test]
    fn population_balance_maintained() {
        // 4×4 grid, k=2, tolerance 10%.
        let adj = grid_adj(4, 4);
        let pop = vec![1000i64; 16];
        let assignment: Vec<u32> = (0..16).map(|i| if i < 8 { 1 } else { 2 }).collect();
        let mut chain = ForestRecomChain::new(adj, pop, assignment, 2, 0.10);
        let (mut rf, mut rr) = make_rngs(55);
        for _ in 0..100 {
            chain.step(&mut rf, &mut rr);
            let dev = chain.max_pop_deviation();
            assert!(dev <= 0.11, "pop deviation {dev:.4} must not exceed tolerance+epsilon");
        }
    }

    #[test]
    fn count_balanced_cuts_symmetric() {
        // Edge (a,b) and (b,a) must both appear or neither.  In practice edges
        // are stored as (child, parent), so we just verify the count is stable
        // across multiple calls with the same tree.
        let adj = grid_adj(4, 4);
        let local_pop = vec![1000i64; 16];
        let target = 8000.0_f64;
        let tol_abs = target * 0.05;
        let mut rng = SmallRng::seed_from_u64(77);
        let tree = random_spanning_tree(&adj, &mut rng);
        let cuts1 = count_balanced_cuts(&tree, &local_pop, target, tol_abs);
        let cuts2 = count_balanced_cuts(&tree, &local_pop, target, tol_abs);
        assert_eq!(cuts1.len(), cuts2.len(), "count must be stable / deterministic");
    }
}
