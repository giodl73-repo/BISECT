//! Top-level SMC algorithm: run_smc(). Per spec §2.1.
//!
//! Runs the Fifield et al. (2020) Sequential Monte Carlo redistricting sampler.
//! Produces a weighted sample of N valid k-district plans.

use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use thiserror::Error;

use crate::partial_plan::PartialPlan;
use crate::proposal::{propose_district, ProposeError};
use crate::resample::{ess, kahan_softmax, systematic_resample};
use crate::seeds::{particle_seed, resample_seed};
use crate::output::{ResampleRecord, SmcResult};

#[derive(Debug, Clone)]
pub struct SmcConfig {
    /// Number of particles (plans in the weighted sample).
    pub n_particles: usize,
    /// Resample when ESS < resample_threshold * n_particles. Default: 0.5.
    pub resample_threshold: f64,
    /// Population balance tolerance as a fraction of ideal. Default: 0.005 (±0.5%).
    pub pop_tolerance: f64,
    /// Base seed for all derived seeds. Default: 0 (use content-derived seed in CLI).
    pub base_seed: u64,
}

impl Default for SmcConfig {
    fn default() -> Self {
        Self {
            n_particles: 5000,
            resample_threshold: 0.5,
            pop_tolerance: 0.005,
            base_seed: 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum SmcError {
    #[error("all {n_particles} particles killed at stage {stage} — ESS reached 0; try more particles or a looser pop_tolerance")]
    AllParticlesKilled { stage: usize, n_particles: usize },
    #[error("invalid configuration: {msg}")]
    InvalidConfig { msg: String },
}

/// Run the Sequential Monte Carlo redistricting sampler.
///
/// Returns `SmcResult` with a weighted sample of `config.n_particles` valid k-district plans.
/// Per spec §2.1.
pub fn run_smc(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    k: usize,
    config: SmcConfig,
) -> Result<SmcResult, SmcError> {
    let n = adjacency.len();
    let n_p = config.n_particles;

    if k == 0 {
        return Err(SmcError::InvalidConfig { msg: "k must be ≥ 1".into() });
    }
    if n_p == 0 {
        return Err(SmcError::InvalidConfig { msg: "n_particles must be ≥ 1".into() });
    }

    // k=1: all plans trivially assign every tract to district 1
    if k == 1 {
        let plan: Vec<u32> = vec![1u32; n];
        return Ok(SmcResult {
            plans: vec![plan; n_p],
            weights: vec![1.0 / n_p as f64; n_p],
            resample_count: 0,
            resample_rounds: vec![],
            ess_trace: vec![],
            index_maps: vec![],
            base_seed: config.base_seed,
            n_particles: n_p,
        });
    }

    // Initialise: N empty partial plans, log-weights = 0
    let mut particles: Vec<PartialPlan> = (0..n_p)
        .map(|_| PartialPlan::empty(n))
        .collect();
    let mut log_weights: Vec<f64> = vec![0.0f64; n_p];

    let mut ess_trace: Vec<f64> = Vec::with_capacity(k - 1);
    let mut resample_rounds: Vec<usize> = Vec::new();
    let mut index_maps: Vec<Vec<usize>> = Vec::new();
    let mut resample_count = 0u32;

    // Main loop: propose districts 1..=(k-1), then assign_remaining for district k
    for stage in 1..k {
        // ── Proposal step (parallel across particles) ──────────────────────────
        // Each particle at this stage is independent: its proposal uses a seed
        // derived from (base_seed, stage, i), so there is no shared mutable state.
        // We collect results into a Vec (deterministic order via enumerate) before
        // applying them sequentially to avoid non-determinism in the weight update.
        //
        // Note: we do NOT use par_iter() on the C METIS library (the proposal uses
        // redist-ensemble's pure-Rust Wilson's algorithm, which is thread-safe).
        let proposal_results: Vec<(Option<PartialPlan>, f64)> = (0..n_p)
            .into_par_iter()
            .map(|i| {
                if log_weights[i] == f64::NEG_INFINITY {
                    return (None, f64::NEG_INFINITY); // already killed
                }
                let seed = particle_seed(config.base_seed, stage as u32, i as u32);
                let mut rng = SmallRng::seed_from_u64(seed);
                match propose_district(
                    &particles[i], adjacency, vertex_weights,
                    k, stage, config.pop_tolerance, &mut rng, i,
                ) {
                    Ok((new_partial, log_w)) => (Some(new_partial), log_w),
                    Err(ProposeError::NoValidCut { .. } | ProposeError::EmptySubgraph { .. }) => {
                        (None, f64::NEG_INFINITY)
                    }
                }
            })
            .collect();

        // Apply results sequentially (deterministic order)
        for (i, (new_partial_opt, log_w)) in proposal_results.into_iter().enumerate() {
            match new_partial_opt {
                Some(new_partial) => {
                    particles[i] = new_partial;
                    log_weights[i] += log_w;
                }
                None => {
                    log_weights[i] = f64::NEG_INFINITY;
                }
            }
        }

        // ── ESS check ──────────────────────────────────────────────────────────
        let current_ess = ess(&log_weights);
        ess_trace.push(current_ess);

        if current_ess == 0.0 {
            return Err(SmcError::AllParticlesKilled { stage, n_particles: n_p });
        }

        // ── Resample if needed ─────────────────────────────────────────────────
        if current_ess < config.resample_threshold * n_p as f64 {
            let rseed = resample_seed(config.base_seed, resample_count);
            let (new_indices, index_map) = systematic_resample(n_p, &log_weights, rseed);

            // Rebuild particles according to new_indices
            let old_particles = particles.clone();
            for (j, &src) in new_indices.iter().enumerate() {
                particles[j] = old_particles[src].clone();
            }
            // Reset weights to uniform (equal after resample)
            for lw in log_weights.iter_mut() {
                *lw = 0.0;
            }

            resample_rounds.push(stage);
            index_maps.push(index_map);
            resample_count += 1;
        }
    }

    // ── Final stage: assign remaining tracts to district k ─────────────────────
    for particle in particles.iter_mut() {
        particle.assign_remaining(k as u32);
    }

    // ── Normalise weights ──────────────────────────────────────────────────────
    let weights = kahan_softmax(&log_weights);

    // ── Finalise plans ─────────────────────────────────────────────────────────
    let plans: Vec<Vec<u32>> = particles.iter()
        .map(|p| p.finalise())
        .collect();

    Ok(SmcResult {
        plans,
        weights,
        resample_count,
        resample_rounds,
        ess_trace,
        index_maps,
        base_seed: config.base_seed,
        n_particles: n_p,
    })
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

    fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<usize>> {
        let n = rows * cols;
        let mut adj = vec![vec![]; n];
        for r in 0..rows {
            for c in 0..cols {
                let v = r * cols + c;
                if c + 1 < cols { adj[v].push(v + 1); adj[v + 1].push(v); }
                if r + 1 < rows { adj[v].push(v + cols); adj[v + cols].push(v); }
            }
        }
        adj
    }

    fn is_connected(tracts: &[usize], adj: &[Vec<usize>]) -> bool {
        if tracts.is_empty() { return true; }
        let set: std::collections::HashSet<usize> = tracts.iter().copied().collect();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(tracts[0]);
        visited.insert(tracts[0]);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if set.contains(&nb) && !visited.contains(&nb) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        visited.len() == tracts.len()
    }

    #[test]
    fn smc_k1_trivial() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let cfg = SmcConfig { n_particles: 10, base_seed: 0, ..Default::default() };
        let result = run_smc(&adj, &pop, 1, cfg).unwrap();
        assert_eq!(result.n_plans(), 10);
        assert!(result.plans.iter().all(|p| p.iter().all(|&d| d == 1)),
            "k=1: all tracts in district 1");
        let wsum: f64 = result.weights.iter().sum();
        assert!((wsum - 1.0).abs() < 1e-9, "weights sum to 1: {wsum}");
        assert_eq!(result.resample_count, 0);
        assert!(result.ess_trace.is_empty());
    }

    #[test]
    fn smc_path_k2_produces_valid_plans() {
        let adj = path_adj(6);
        let pop = vec![100i64; 6];
        let cfg = SmcConfig { n_particles: 50, base_seed: 42,
                              pop_tolerance: 0.2, ..Default::default() };
        let result = run_smc(&adj, &pop, 2, cfg).unwrap();

        assert_eq!(result.n_plans(), 50);
        let wsum: f64 = result.weights.iter().sum();
        assert!((wsum - 1.0).abs() < 1e-6, "weights sum 1.0 ± 1e-6: {wsum}");

        for plan in &result.plans {
            assert_eq!(plan.len(), 6, "all 6 tracts assigned");
            let d1: Vec<usize> = (0..6).filter(|&i| plan[i] == 1).collect();
            let d2: Vec<usize> = (0..6).filter(|&i| plan[i] == 2).collect();
            assert!(!d1.is_empty() && !d2.is_empty(), "both districts non-empty");
            assert_eq!(d1.len() + d2.len(), 6, "partition covers all tracts");
            assert!(is_connected(&d1, &adj), "district 1 contiguous");
            assert!(is_connected(&d2, &adj), "district 2 contiguous");
        }
    }

    #[test]
    fn smc_deterministic_same_seed() {
        let adj = path_adj(8);
        let pop = vec![100i64; 8];
        let cfg1 = SmcConfig { n_particles: 20, base_seed: 99,
                               pop_tolerance: 0.2, ..Default::default() };
        let cfg2 = cfg1.clone();
        let r1 = run_smc(&adj, &pop, 2, cfg1).unwrap();
        let r2 = run_smc(&adj, &pop, 2, cfg2).unwrap();
        assert_eq!(r1.plans, r2.plans, "same seed → same plans");
        assert_eq!(r1.weights, r2.weights, "same seed → same weights");
        assert_eq!(r1.resample_count, r2.resample_count);
    }

    #[test]
    fn smc_weights_all_positive() {
        let adj = grid_adj(3, 3);
        let pop = vec![100i64; 9];
        let cfg = SmcConfig { n_particles: 30, base_seed: 7,
                              pop_tolerance: 0.4, ..Default::default() };
        let result = run_smc(&adj, &pop, 3, cfg).unwrap();
        assert!(result.weights.iter().all(|&w| w >= 0.0),
            "all weights must be non-negative");
    }

    #[test]
    fn smc_ess_trace_length_k_minus_1() {
        let adj = path_adj(6);
        let pop = vec![100i64; 6];
        let cfg = SmcConfig { n_particles: 20, base_seed: 1,
                              pop_tolerance: 0.3, ..Default::default() };
        let result = run_smc(&adj, &pop, 3, cfg).unwrap();
        assert_eq!(result.ess_trace.len(), 2, "k=3 → ESS trace length = k-1 = 2");
    }

    #[test]
    fn smc_k1_error_on_zero_particles() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let cfg = SmcConfig { n_particles: 0, ..Default::default() };
        assert!(matches!(run_smc(&adj, &pop, 2, cfg), Err(SmcError::InvalidConfig { .. })));
    }
}
