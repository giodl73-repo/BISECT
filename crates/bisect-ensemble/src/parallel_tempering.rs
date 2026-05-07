//! Parallel Tempering — Multi-Chain Replica Exchange MCMC for Redistricting.
//!
//! Runs `n_replicas` ForestRecomChains simultaneously on a geometric tolerance
//! ladder. Every `swap_interval` steps, adjacent replicas propose to exchange
//! their current plans via a Metropolis criterion. Hot chains mix quickly;
//! cold chains (low tolerance) produce distribution-correct samples.
//!
//! See spec: `docs/specs/2026-05-07-parallel-tempering.md`

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use sha2::{Digest, Sha256};
use crate::forest_recom::ForestRecomChain;

// ── Seed derivation helpers ───────────────────────────────────────────────────

/// Derive the replica step seed for a given replica, step, and base seed.
///
/// `SHA-256("PT_REPLICA_" || replica:u32le || "_" || step:u64le || "_" || base_seed:u64le)`
/// → least-significant 64 bits.
pub fn replica_seed(base_seed: u64, replica: u32, step: u64) -> u64 {
    let mut h = Sha256::new();
    h.update(b"PT_REPLICA_");
    h.update(replica.to_le_bytes());
    h.update(b"_");
    h.update(step.to_le_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Derive the swap RNG seed for a given step, pair index, and base seed.
///
/// `SHA-256("PT_SWAP_" || pair:u32le || "_" || step:u64le || "_" || base_seed:u64le)`
/// → least-significant 64 bits.
pub fn swap_seed(base_seed: u64, step: u64, pair: u32) -> u64 {
    let mut h = Sha256::new();
    h.update(b"PT_SWAP_");
    h.update(pair.to_le_bytes());
    h.update(b"_");
    h.update(step.to_le_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Derive forward/reverse RNGs from a replica seed.
///
/// Uses domain-separated `"PT_FWD_"` / `"PT_REV_"` prefixes, matching the
/// seeding convention in `forest_recom`.
pub fn replica_rngs(rseed: u64) -> (SmallRng, SmallRng) {
    let fwd = {
        let mut h = Sha256::new();
        h.update(b"PT_FWD_");
        h.update(rseed.to_le_bytes());
        let d = h.finalize();
        SmallRng::seed_from_u64(u64::from_le_bytes(d[..8].try_into().unwrap()))
    };
    let rev = {
        let mut h = Sha256::new();
        h.update(b"PT_REV_");
        h.update(rseed.to_le_bytes());
        let d = h.finalize();
        SmallRng::seed_from_u64(u64::from_le_bytes(d[..8].try_into().unwrap()))
    };
    (fwd, rev)
}

// ── Edge-cut helper ───────────────────────────────────────────────────────────

/// Count edges crossing district boundaries (each undirected edge counted once).
fn count_edge_cuts_u32(assignment: &[u32], adj: &[Vec<u32>]) -> usize {
    let mut cut = 0usize;
    for (v, nbrs) in adj.iter().enumerate() {
        let dv = assignment[v];
        for &nb in nbrs {
            if nb as usize > v && assignment[nb as usize] != dv {
                cut += 1;
            }
        }
    }
    cut
}

// ── Main struct ───────────────────────────────────────────────────────────────

/// Parallel Tempering chain: `n_replicas` ForestRecomChains on a geometric
/// tolerance ladder, with periodic Metropolis replica-exchange proposals.
pub struct ParallelTemperingChain {
    /// One ForestRecomChain per replica, ordered cold→hot (index 0 = coldest).
    pub replicas: Vec<ForestRecomChain>,
    /// Balance tolerance for each replica (geometric ladder, tolerances[0] = cold).
    pub tolerances: Vec<f64>,
    pub k: u32,
    pub n_replicas: usize,
    pub swap_interval: usize,
    pub steps_taken: u64,
    pub swap_attempts: u64,
    pub swap_acceptances: u64,
    /// All plans recorded from the cold chain (replica 0), one per step.
    /// Entry: (step_idx, assignment). step_idx 0 = initial plan.
    pub cold_chain_records: Vec<(usize, Vec<u32>)>,
}

impl ParallelTemperingChain {
    /// Build `n_replicas` ForestRecomChains on a geometric tolerance ladder.
    ///
    /// All replicas start from the same `initial_assignment`.
    ///
    /// Geometric ladder: `tolerances[i] = cold * (hot/cold)^(i/(n_replicas-1))`.
    pub fn new(
        adj: Vec<Vec<u32>>,
        pop: Vec<i64>,
        initial_assignment: Vec<u32>,
        k: u32,
        cold_tolerance: f64,
        hot_tolerance: f64,
        n_replicas: usize,
        swap_interval: usize,
    ) -> Self {
        assert!(n_replicas >= 1, "n_replicas must be >= 1");

        let tolerances: Vec<f64> = if n_replicas == 1 {
            vec![cold_tolerance]
        } else {
            (0..n_replicas)
                .map(|i| {
                    cold_tolerance
                        * (hot_tolerance / cold_tolerance)
                            .powf(i as f64 / (n_replicas - 1) as f64)
                })
                .collect()
        };

        let replicas = tolerances
            .iter()
            .map(|&tol| {
                ForestRecomChain::new(
                    adj.clone(),
                    pop.clone(),
                    initial_assignment.clone(),
                    k,
                    tol,
                )
            })
            .collect();

        let initial_record = (0, initial_assignment);

        Self {
            replicas,
            tolerances,
            k,
            n_replicas,
            swap_interval,
            steps_taken: 0,
            swap_attempts: 0,
            swap_acceptances: 0,
            cold_chain_records: vec![initial_record],
        }
    }

    /// Run one step across all replicas, then (if `step % swap_interval == 0`)
    /// propose swaps between adjacent replicas.
    ///
    /// `rng_replicas`: one `(rng_fwd, rng_rev)` pair per replica, derived from
    /// seeds externally (see [`replica_rngs`]).
    /// `rng_swap`: RNG for the swap coin flips.
    pub fn step(
        &mut self,
        rng_replicas: &mut Vec<(SmallRng, SmallRng)>,
        rng_swap: &mut SmallRng,
    ) {
        self.steps_taken += 1;

        // 1. Each replica takes one step.
        for ((rng_fwd, rng_rev), replica) in
            rng_replicas.iter_mut().zip(self.replicas.iter_mut())
        {
            replica.step(rng_fwd, rng_rev);
        }

        // 2. Record cold chain state after the step.
        self.cold_chain_records.push((
            self.steps_taken as usize,
            self.replicas[0].assignment.clone(),
        ));

        // 3. Propose replica swaps every swap_interval steps.
        if self.steps_taken % self.swap_interval as u64 == 0 {
            for i in 0..self.n_replicas.saturating_sub(1) {
                self.swap_attempts += 1;

                let ec_i =
                    count_edge_cuts_u32(&self.replicas[i].assignment, &self.replicas[i].adj);
                let ec_j = count_edge_cuts_u32(
                    &self.replicas[i + 1].assignment,
                    &self.replicas[i + 1].adj,
                );

                let beta_i = 1.0 / self.tolerances[i].max(1e-10);
                let beta_j = 1.0 / self.tolerances[i + 1].max(1e-10);

                // Metropolis criterion for swap.
                let log_ratio = (beta_i - beta_j) * (ec_i as f64 - ec_j as f64);

                if rng_swap.gen::<f64>().ln() < log_ratio.min(0.0) {
                    // Accept swap: exchange assignments between replicas i and i+1.
                    let (left, right) = self.replicas.split_at_mut(i + 1);
                    std::mem::swap(&mut left[i].assignment, &mut right[0].assignment);
                    self.swap_acceptances += 1;
                }
            }
        }
    }

    /// Current assignment of the cold chain (replica 0).
    pub fn cold_assignment(&self) -> &Vec<u32> {
        &self.replicas[0].assignment
    }

    /// Fraction of swap proposals that were accepted.
    pub fn swap_acceptance_rate(&self) -> f64 {
        if self.swap_attempts == 0 {
            0.0
        } else {
            self.swap_acceptances as f64 / self.swap_attempts as f64
        }
    }

    /// Acceptance rate of the cold chain's internal Forest ReCom proposals.
    pub fn cold_chain_acceptance_rate(&self) -> f64 {
        self.replicas[0].acceptance_rate()
    }

    /// Mean edge-cut count across all cold chain records.
    pub fn cold_chain_ec_mean(&self) -> f64 {
        if self.cold_chain_records.is_empty() {
            return 0.0;
        }
        let adj = &self.replicas[0].adj;
        let sum: usize = self
            .cold_chain_records
            .iter()
            .map(|(_, a)| count_edge_cuts_u32(a, adj))
            .sum();
        sum as f64 / self.cold_chain_records.len() as f64
    }

    /// Select a plan from the cold chain records by percentile `p` of edge-cut
    /// count (ascending). `p=0.0` returns the minimum-EC plan; `p=1.0` returns
    /// the maximum-EC plan.
    pub fn select_plan(&self, p: f64) -> Vec<u32> {
        assert!(
            !self.cold_chain_records.is_empty(),
            "no cold chain records to select from"
        );
        let adj = &self.replicas[0].adj;
        let mut indexed: Vec<(usize, usize)> = self
            .cold_chain_records
            .iter()
            .enumerate()
            .map(|(idx, (_, a))| (idx, count_edge_cuts_u32(a, adj)))
            .collect();
        // Sort by (EC ASC, step index ASC) for deterministic tie-breaking.
        indexed.sort_by_key(|&(idx, ec)| (ec, idx));
        let rank = (p * (indexed.len() - 1) as f64).floor() as usize;
        let rank = rank.min(indexed.len() - 1);
        self.cold_chain_records[indexed[rank].0].1.clone()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ── Graph helpers ─────────────────────────────────────────────────────────

    fn path_adj_u32(n: usize) -> Vec<Vec<u32>> {
        let mut adj = vec![vec![]; n];
        for i in 0..n - 1 {
            adj[i].push((i + 1) as u32);
            adj[i + 1].push(i as u32);
        }
        adj
    }

    fn grid_adj_u32(rows: usize, cols: usize) -> Vec<Vec<u32>> {
        let n = rows * cols;
        let mut adj = vec![vec![]; n];
        for r in 0..rows {
            for c in 0..cols {
                let v = r * cols + c;
                if c + 1 < cols {
                    adj[v].push((v + 1) as u32);
                    adj[v + 1].push(v as u32);
                }
                if r + 1 < rows {
                    adj[v].push((v + cols) as u32);
                    adj[v + cols].push(v as u32);
                }
            }
        }
        adj
    }

    /// Build a PT chain on a 4x4 grid with k=2 and a 2-replica ladder.
    fn make_grid_pt(
        n_replicas: usize,
        swap_interval: usize,
        cold: f64,
        hot: f64,
    ) -> ParallelTemperingChain {
        let adj = grid_adj_u32(4, 4);
        let pop = vec![1000i64; 16];
        let assignment: Vec<u32> = (0..16u32).map(|i| if i < 8 { 1 } else { 2 }).collect();
        ParallelTemperingChain::new(adj, pop, assignment, 2, cold, hot, n_replicas, swap_interval)
    }

    /// Advance a PT chain T steps using deterministic seeds derived from base_seed.
    fn run_pt(pt: &mut ParallelTemperingChain, steps: usize, base_seed: u64) {
        for _s in 0..steps {
            let step = pt.steps_taken + 1;
            let mut rng_replicas: Vec<(SmallRng, SmallRng)> = (0..pt.n_replicas)
                .map(|i| {
                    let rseed = replica_seed(base_seed, i as u32, step);
                    replica_rngs(rseed)
                })
                .collect();
            // Swap seed for this step uses pair=0 as the per-step swap RNG.
            let sseed = swap_seed(base_seed, step, 0);
            let mut rng_swap = SmallRng::seed_from_u64(sseed);
            pt.step(&mut rng_replicas, &mut rng_swap);
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn tolerance_ladder_correct() {
        let cold = 0.005_f64;
        let hot = 0.05_f64;
        let n = 4usize;
        let pt = make_grid_pt(n, 10, cold, hot);

        assert!(
            (pt.tolerances[0] - cold).abs() < 1e-12,
            "tolerances[0] must equal cold_tolerance"
        );
        assert!(
            (pt.tolerances[n - 1] - hot).abs() < 1e-12,
            "tolerances[n-1] must equal hot_tolerance"
        );
        for i in 0..n - 1 {
            assert!(
                pt.tolerances[i] < pt.tolerances[i + 1],
                "tolerances must be strictly increasing: {} >= {}",
                pt.tolerances[i],
                pt.tolerances[i + 1]
            );
        }
    }

    #[test]
    fn replica_seed_distinct_from_swap_seed() {
        // For any base seed and coinciding (replica, pair, step) values,
        // replica_seed and swap_seed must differ due to domain-separated prefixes.
        for base in [0u64, 1, 42, u64::MAX] {
            let rs = replica_seed(base, 0, 0);
            let ss = swap_seed(base, 0, 0);
            assert_ne!(
                rs, ss,
                "replica_seed({base},0,0) must differ from swap_seed({base},0,0)"
            );
        }
    }

    #[test]
    fn pt_fwd_rev_distinct() {
        // replica_rngs must produce two distinct initial states.
        for rseed in [0u64, 1, 99, 0xDEAD_BEEF] {
            let (fwd, rev) = replica_rngs(rseed);
            // SmallRng does not expose its state directly, but we can compare
            // initial generated values.
            let mut f = fwd;
            let mut r = rev;
            let fv: u64 = f.gen();
            let rv: u64 = r.gen();
            assert_ne!(fv, rv, "PT_FWD_ and PT_REV_ must produce distinct streams");
        }
    }

    #[test]
    fn cold_chain_records_length() {
        let t = 30usize;
        let mut pt = make_grid_pt(2, 5, 0.005, 0.05);
        run_pt(&mut pt, t, 12345);
        assert_eq!(
            pt.cold_chain_records.len(),
            t + 1,
            "cold_chain_records must have T+1 entries (initial + one per step)"
        );
    }

    #[test]
    fn swap_attempts_count() {
        let t = 30usize;
        let swap_interval = 5usize;
        let n_replicas = 3usize;
        let mut pt = make_grid_pt(n_replicas, swap_interval, 0.005, 0.05);
        run_pt(&mut pt, t, 999);
        let expected = (t / swap_interval) * (n_replicas - 1);
        assert_eq!(
            pt.swap_attempts, expected as u64,
            "swap_attempts must equal (T / swap_interval) * (n_replicas - 1)"
        );
    }

    #[test]
    fn n_replicas_1_degenerate() {
        let t = 20usize;
        let adj = path_adj_u32(4);
        let pop = vec![1000i64; 4];
        let assignment = vec![1u32, 1, 2, 2];
        let mut pt =
            ParallelTemperingChain::new(adj, pop, assignment, 2, 0.05, 0.05, 1, 5);
        run_pt(&mut pt, t, 77);

        assert_eq!(
            pt.swap_attempts, 0,
            "n_replicas=1: no swap attempts possible"
        );
        assert_eq!(
            pt.cold_chain_records.len(),
            t + 1,
            "n_replicas=1: cold_chain_records must have T+1 entries"
        );
        // All records must be valid 2-district plans.
        for (_, a) in &pt.cold_chain_records {
            let ds: HashSet<u32> = a.iter().copied().collect();
            assert_eq!(ds.len(), 2, "degenerate case: must always have 2 districts");
        }
    }

    #[test]
    fn all_replicas_valid_plans() {
        let n_replicas = 3usize;
        let mut pt = make_grid_pt(n_replicas, 5, 0.005, 0.10);
        run_pt(&mut pt, 20, 42);

        for (i, replica) in pt.replicas.iter().enumerate() {
            let ds: HashSet<u32> = replica.assignment.iter().copied().collect();
            assert_eq!(
                ds.len(),
                2,
                "replica {i} must have exactly 2 districts after 20 steps"
            );
            assert_eq!(
                replica.assignment.len(),
                16,
                "replica {i} assignment must cover all 16 nodes"
            );
        }
    }

    #[test]
    fn pt_deterministic() {
        let t = 25usize;
        let base_seed = 314159u64;

        let mut pt1 = make_grid_pt(3, 5, 0.005, 0.05);
        run_pt(&mut pt1, t, base_seed);

        let mut pt2 = make_grid_pt(3, 5, 0.005, 0.05);
        run_pt(&mut pt2, t, base_seed);

        assert_eq!(
            pt1.cold_chain_records, pt2.cold_chain_records,
            "same base_seed must produce identical cold chain records"
        );
        assert_eq!(
            pt1.swap_acceptances, pt2.swap_acceptances,
            "same base_seed must produce identical swap history"
        );
        for i in 0..3 {
            assert_eq!(
                pt1.replicas[i].assignment, pt2.replicas[i].assignment,
                "same base_seed must produce identical final assignments for replica {i}"
            );
        }
    }

    #[test]
    fn p0_le_p1_ec() {
        // After T steps, select_plan(0.0) should have EC <= select_plan(1.0).
        let mut pt = make_grid_pt(2, 5, 0.005, 0.05);
        run_pt(&mut pt, 40, 271828);

        let adj = &pt.replicas[0].adj;
        let plan_min = pt.select_plan(0.0);
        let plan_max = pt.select_plan(1.0);
        let ec_min = count_edge_cuts_u32(&plan_min, adj);
        let ec_max = count_edge_cuts_u32(&plan_max, adj);
        assert!(
            ec_min <= ec_max,
            "select_plan(0.0).EC={ec_min} must be <= select_plan(1.0).EC={ec_max}"
        );
    }

    // ── Additional invariant tests ────────────────────────────────────────────

    #[test]
    fn swap_acceptances_le_swap_attempts() {
        let mut pt = make_grid_pt(4, 5, 0.005, 0.10);
        run_pt(&mut pt, 50, 54321);
        assert!(
            pt.swap_acceptances <= pt.swap_attempts,
            "swap_acceptances {} must not exceed swap_attempts {}",
            pt.swap_acceptances, pt.swap_attempts
        );
    }

    #[test]
    fn cold_chain_step_indices_monotone() {
        let t = 15usize;
        let mut pt = make_grid_pt(2, 5, 0.005, 0.05);
        run_pt(&mut pt, t, 888);
        // Step indices: 0, 1, 2, ..., t
        for (expected_idx, (step_idx, _)) in pt.cold_chain_records.iter().enumerate() {
            assert_eq!(
                *step_idx, expected_idx,
                "cold_chain_records[{expected_idx}].step must be {expected_idx}, got {step_idx}"
            );
        }
    }

    #[test]
    fn select_plan_single_record() {
        let adj = path_adj_u32(4);
        let pop = vec![1000i64; 4];
        let assignment = vec![1u32, 1, 2, 2];
        let pt = ParallelTemperingChain::new(adj, pop, assignment.clone(), 2, 0.05, 0.10, 1, 5);
        // Only the initial record; select_plan at any percentile returns it.
        assert_eq!(pt.select_plan(0.0), assignment);
        assert_eq!(pt.select_plan(1.0), assignment);
    }

    #[test]
    fn tolerance_ladder_n2() {
        // n_replicas=2: tolerances must be [cold, hot] exactly.
        let cold = 0.01_f64;
        let hot = 0.08_f64;
        let pt = make_grid_pt(2, 5, cold, hot);
        assert!((pt.tolerances[0] - cold).abs() < 1e-12);
        assert!((pt.tolerances[1] - hot).abs() < 1e-12);
    }
}
