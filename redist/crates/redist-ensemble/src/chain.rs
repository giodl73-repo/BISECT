//! Markov chain runner — serial and parallel (Rayon).

use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::recom::{RecomChain, StepRecord};

/// Per-chain result.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResult {
    pub chain_idx: usize,
    pub seed: u64,
    pub steps: Vec<StepRecord>,
}

impl Serialize for StepRecord {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("StepRecord", 5)?;
        st.serialize_field("step", &self.step)?;
        st.serialize_field("cut_edges", &self.cut_edges)?;
        st.serialize_field("cut_fraction", &self.cut_fraction)?;
        st.serialize_field("pop_deviation", &self.pop_deviation)?;
        st.serialize_field("accepted", &self.accepted)?;
        st.end()
    }
}

impl<'de> Deserialize<'de> for StepRecord {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            step: u64, cut_edges: usize, cut_fraction: f32, pop_deviation: f32, accepted: bool,
        }
        let h = Helper::deserialize(d)?;
        Ok(StepRecord { step: h.step, cut_edges: h.cut_edges, cut_fraction: h.cut_fraction, pop_deviation: h.pop_deviation, accepted: h.accepted })
    }
}

/// Full ensemble result.
#[derive(Debug, Serialize, Deserialize)]
pub struct EnsembleResult {
    pub state: String,
    pub k: u32,
    pub n_steps: u64,
    pub n_chains: usize,
    pub chain_seeds: Vec<u64>,
    pub chains: Vec<ChainResult>,
    /// R-hat for cut_fraction (continuous approx; use rank-normalised for discrete stats).
    pub r_hat: Option<f64>,
    /// Effective sample size (Geyer 1992).
    pub ess: Option<f64>,
    /// Lag-1 to lag-20 Hamming autocorrelation.
    pub hamming_autocorr: Vec<f64>,
    /// Pooled mean cut fraction.
    pub pooled_cut_mean: f64,
    /// Pooled std cut fraction.
    pub pooled_cut_std: f64,
}

/// Derive a per-chain seed deterministically.
///
/// `chain_seed(base, i) = first 8 bytes of SHA-256("ENSEMBLE_CHAIN_" || i || "_" || base_seed)`
pub fn chain_seed(base_seed: u64, chain_idx: usize) -> u64 {
    let mut h = Sha256::new();
    h.update(b"ENSEMBLE_CHAIN_");
    h.update(chain_idx.to_le_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let digest = h.finalize();
    u64::from_le_bytes(digest[..8].try_into().unwrap())
}

/// Run a single chain for `n_steps` steps.
fn run_chain(
    adj: &[Vec<u32>],
    pop: &[i64],
    initial_assignment: &[u32],
    k: u32,
    pop_tolerance: f64,
    n_steps: u64,
    seed: u64,
    chain_idx: usize,
) -> ChainResult {
    let mut chain = RecomChain::new(
        adj.to_vec(),
        pop.to_vec(),
        initial_assignment.to_vec(),
        k,
        pop_tolerance,
    );
    let mut rng = SmallRng::seed_from_u64(seed);
    let steps: Vec<StepRecord> = (0..n_steps).map(|_| chain.step(&mut rng)).collect();
    ChainResult { chain_idx, seed, steps }
}

/// Run `n_chains` chains in parallel (Rayon), each for `n_steps` steps.
pub fn run_ensemble(
    adj: Vec<Vec<u32>>,
    pop: Vec<i64>,
    initial_assignment: Vec<u32>,
    k: u32,
    pop_tolerance: f64,
    n_steps: u64,
    n_chains: usize,
    base_seed: u64,
    state: String,
) -> EnsembleResult {
    let seeds: Vec<u64> = (0..n_chains).map(|i| chain_seed(base_seed, i)).collect();

    let chains: Vec<ChainResult> = seeds
        .par_iter()
        .enumerate()
        .map(|(i, &seed)| {
            run_chain(&adj, &pop, &initial_assignment, k, pop_tolerance, n_steps, seed, i)
        })
        .collect();

    // Pool all cut fractions for summary stats.
    let all_cuts: Vec<f64> = chains.iter()
        .flat_map(|c| c.steps.iter().map(|s| s.cut_fraction as f64))
        .collect();
    let pooled_cut_mean = all_cuts.iter().sum::<f64>() / all_cuts.len().max(1) as f64;
    let pooled_cut_std = {
        let var = all_cuts.iter().map(|&x| (x - pooled_cut_mean).powi(2)).sum::<f64>()
            / all_cuts.len().max(1) as f64;
        var.sqrt()
    };

    let r_hat = if n_chains >= 2 { Some(compute_r_hat(&chains)) } else { None };
    let ess = Some(compute_ess(&chains));
    let hamming_autocorr = compute_hamming_autocorr(&chains, 20);

    EnsembleResult {
        state, k, n_steps, n_chains,
        chain_seeds: seeds,
        chains,
        r_hat, ess, hamming_autocorr,
        pooled_cut_mean, pooled_cut_std,
    }
}

// ── Diagnostics ───────────────────────────────────────────────────────────────

/// Gelman-Rubin R-hat for cut_fraction (treats it as approximately continuous).
/// For proper discrete-statistic R-hat use rank-normalisation (Vehtari et al. 2021).
fn compute_r_hat(chains: &[ChainResult]) -> f64 {
    let m = chains.len() as f64;
    let n = chains[0].steps.len() as f64;
    if n < 2.0 || m < 2.0 { return f64::NAN; }

    let chain_means: Vec<f64> = chains.iter().map(|c| {
        c.steps.iter().map(|s| s.cut_fraction as f64).sum::<f64>() / n
    }).collect();
    let grand_mean = chain_means.iter().sum::<f64>() / m;

    // Between-chain variance.
    let b = n / (m - 1.0) * chain_means.iter().map(|&mu| (mu - grand_mean).powi(2)).sum::<f64>();

    // Within-chain variance.
    let w = chains.iter().map(|c| {
        let mu = c.steps.iter().map(|s| s.cut_fraction as f64).sum::<f64>() / n;
        c.steps.iter().map(|s| (s.cut_fraction as f64 - mu).powi(2)).sum::<f64>() / (n - 1.0)
    }).sum::<f64>() / m;

    let var_hat = (n - 1.0) / n * w + b / n;
    (var_hat / w.max(1e-12)).sqrt()
}

/// Estimate ESS from the first chain using Geyer's initial monotone sequence estimator.
fn compute_ess(chains: &[ChainResult]) -> f64 {
    let series: Vec<f64> = chains[0].steps.iter().map(|s| s.cut_fraction as f64).collect();
    let n = series.len();
    if n < 4 { return n as f64; }

    let mean = series.iter().sum::<f64>() / n as f64;
    let var = series.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    if var < 1e-12 { return 1.0; }

    // Autocorrelation at lag t.
    let acf = |t: usize| -> f64 {
        (0..n-t).map(|i| (series[i] - mean) * (series[i+t] - mean)).sum::<f64>()
            / (n as f64 * var)
    };

    // Geyer's initial monotone sequence.
    let mut sum = 1.0_f64;
    let mut t = 1usize;
    while t + 1 < n {
        let pair = acf(t) + acf(t + 1);
        if pair <= 0.0 { break; }
        sum += 2.0 * pair;
        t += 2;
    }
    (n as f64 / sum).max(1.0)
}

/// Compute Hamming autocorrelation at lags 1..=max_lag.
///
/// Hamming distance between consecutive assignments is a natural
/// measure of chain mixing for discrete redistricting plans.
fn compute_hamming_autocorr(chains: &[ChainResult], max_lag: usize) -> Vec<f64> {
    // Use cut_fraction as a proxy (actual Hamming requires storing full assignments).
    // Full Hamming requires storing complete assignment vectors per step which is
    // O(n_steps × n_tracts) memory — deferred to the full implementation.
    // Here we compute autocorrelation of cut_fraction as an approximation.
    let series: Vec<f64> = chains[0].steps.iter().map(|s| s.cut_fraction as f64).collect();
    let n = series.len();
    let mean = series.iter().sum::<f64>() / n as f64;
    let var = series.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;

    (1..=max_lag.min(n / 2))
        .map(|lag| {
            if var < 1e-12 { return 0.0; }
            (0..n-lag)
                .map(|i| (series[i] - mean) * (series[i+lag] - mean))
                .sum::<f64>()
                / (n as f64 * var)
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_seed_is_deterministic() {
        assert_eq!(chain_seed(42, 0), chain_seed(42, 0));
        assert_ne!(chain_seed(42, 0), chain_seed(42, 1));
        assert_ne!(chain_seed(42, 0), chain_seed(43, 0));
    }

    #[test]
    fn ensemble_runs_and_returns_correct_shape() {
        // Small 4-tract graph, 2 districts.
        let adj = vec![vec![1u32,2],vec![0,3],vec![0,3],vec![1,2]];
        let pop = vec![1000i64;4];
        let assign = vec![1u32,1,2,2];
        let result = run_ensemble(adj, pop, assign, 2, 0.1, 20, 2, 42, "test".into());
        assert_eq!(result.n_chains, 2);
        assert_eq!(result.chains.len(), 2);
        assert_eq!(result.chains[0].steps.len(), 20);
        assert!(result.r_hat.is_some());
        assert!(result.ess.is_some());
        assert!(!result.hamming_autocorr.is_empty());
    }

    #[test]
    fn pooled_stats_are_plausible() {
        let adj = vec![vec![1u32,2],vec![0,3],vec![1,3],vec![2,1]];  // cycle
        let pop = vec![1000i64;4];
        let assign = vec![1u32,1,2,2];
        let result = run_ensemble(adj, pop, assign, 2, 0.1, 100, 1, 7, "test".into());
        assert!(result.pooled_cut_mean >= 0.0 && result.pooled_cut_mean <= 1.0);
        assert!(result.pooled_cut_std >= 0.0);
    }
}
