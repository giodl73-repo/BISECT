//! Ensemble convergence diagnostics (Researcher Toolkit plan Task 7, S-03).
//!
//! Pure-Rust implementations of the standard MCMC convergence diagnostics
//! the Researcher Toolkit's `validate-ensemble` consumer requires. No
//! `arviz`/Python dependency — the project's diagnostics output is its own
//! source of truth.
//!
//! Three diagnostics:
//!
//! 1. **Gelman-Rubin R-hat** — between-chain vs within-chain variance ratio
//!    on a summary statistic. Requires `n_chains >= 4` (S-03 contract).
//!    `R-hat < 1.05` is the field-standard convergence threshold.
//!
//! 2. **Effective Sample Size (ESS)** — autocorrelation-corrected sample
//!    size on a summary statistic. Computed using the initial monotone
//!    sequence estimator (Geyer 1992); flag if any ESS < 100.
//!
//! 3. **Hamming-distance autocorrelation** — partition-space mixing measure.
//!    For two partitions `p_t` and `p_{t+k}`, the Hamming distance is the
//!    fraction of units assigned to different districts. The autocorrelation
//!    function decays from 1.0 (at lag 0) to ~0 (well-mixed); the integrated
//!    autocorrelation time (`tau_int`) is the conventional summary.
//!
//! Spec Task 7.1; the JSON output shapes match the spec's `ess.json`,
//! `rhat.json`, `hamming_autocorr.json` schemas.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from the diagnostics module.
#[derive(Debug, Error)]
pub enum DiagnosticsError {
    #[error("[INPUT] S-03 requires >=4 parallel chains for Gelman-Rubin R-hat; got {0}")]
    InsufficientChains(usize),
    #[error("[INPUT] empty chain at index {0}")]
    EmptyChain(usize),
    #[error("[INPUT] chains have differing lengths: {0:?}")]
    UnequalChainLengths(Vec<usize>),
    #[error("[INPUT] empty partition trajectory")]
    EmptyTrajectory,
    #[error("[INPUT] partitions have differing unit counts: first={first}, at index {idx}={got}")]
    PartitionLengthMismatch { first: usize, idx: usize, got: usize },
}

// ===========================================================================
// Gelman-Rubin R-hat
// ===========================================================================

/// JSON-shape for `rhat.json` (one entry per analyzed metric).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhatRecord {
    pub metric: String,
    pub r_hat: f64,
    pub n_chains: usize,
    pub n_per_chain: usize,
    /// True iff `r_hat >= 1.05`.
    pub above_threshold: bool,
    pub threshold: f64,
}

/// Compute R-hat for one metric across `n_chains >= 4` parallel chains.
///
/// Each `chain` is a sequence of post-burn-in scalar samples of the metric
/// (e.g., efficiency gap at every accepted state). Chains MUST have equal
/// length per the standard Gelman-Rubin formulation (the caller is responsible
/// for trimming to the shortest chain after burn-in cuts).
///
/// Returns the potential scale reduction factor (PSRF). Values close to 1.0
/// indicate convergence; the field-standard threshold is `R-hat < 1.05`.
pub fn gelman_rubin_rhat(chains: &[&[f64]]) -> Result<f64, DiagnosticsError> {
    let m = chains.len();
    if m < 4 {
        return Err(DiagnosticsError::InsufficientChains(m));
    }
    let n = chains[0].len();
    if n == 0 {
        return Err(DiagnosticsError::EmptyChain(0));
    }
    for (i, c) in chains.iter().enumerate() {
        if c.is_empty() {
            return Err(DiagnosticsError::EmptyChain(i));
        }
        if c.len() != n {
            let lens: Vec<usize> = chains.iter().map(|c| c.len()).collect();
            return Err(DiagnosticsError::UnequalChainLengths(lens));
        }
    }

    // Per-chain means + variances.
    let chain_means: Vec<f64> = chains
        .iter()
        .map(|c| c.iter().sum::<f64>() / (n as f64))
        .collect();
    let grand_mean: f64 = chain_means.iter().sum::<f64>() / (m as f64);
    let chain_vars: Vec<f64> = chains
        .iter()
        .zip(&chain_means)
        .map(|(c, &mean)| {
            c.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ((n - 1).max(1) as f64)
        })
        .collect();

    // Between-chain variance B/n.
    let b_over_n: f64 = chain_means
        .iter()
        .map(|&mean| (mean - grand_mean).powi(2))
        .sum::<f64>()
        / ((m - 1).max(1) as f64);

    // Within-chain variance W.
    let w: f64 = chain_vars.iter().sum::<f64>() / (m as f64);

    if w == 0.0 {
        // Degenerate: all samples identical within every chain. R-hat is
        // conventionally 1.0 in this case (no within-chain variation).
        return Ok(1.0);
    }

    // Pooled variance estimator + R-hat.
    let n_f = n as f64;
    let var_plus = ((n_f - 1.0) / n_f) * w + b_over_n;
    let r_hat = (var_plus / w).sqrt();
    Ok(r_hat)
}

/// Compute R-hat records for a list of (metric_name, per-chain-samples).
pub fn rhat_records(metrics: &[(String, Vec<&[f64]>)]) -> Result<Vec<RhatRecord>, DiagnosticsError> {
    metrics
        .iter()
        .map(|(name, chains)| {
            let m = chains.len();
            let n = chains.first().map(|c| c.len()).unwrap_or(0);
            let r = gelman_rubin_rhat(chains)?;
            let threshold = 1.05;
            Ok(RhatRecord {
                metric: name.clone(),
                r_hat: r,
                n_chains: m,
                n_per_chain: n,
                above_threshold: r >= threshold,
                threshold,
            })
        })
        .collect()
}

// ===========================================================================
// Effective Sample Size (ESS)
// ===========================================================================

/// JSON-shape for `ess.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssRecord {
    pub metric: String,
    pub ess: f64,
    pub n_total: usize,
    /// Always `"summary_statistic"` per S-03 (NOT computed on partition labels).
    pub computed_on: String,
    /// True iff `ess < 100`.
    pub below_threshold: bool,
}

/// Effective sample size on a single concatenated trace using the initial
/// monotone sequence estimator (Geyer 1992): sum autocorrelations until the
/// pair sum (rho_{2k} + rho_{2k+1}) becomes non-positive.
///
/// `ess = N / (1 + 2 * sum(rho_k))` for `k = 1..K` where `K` is the cutoff.
pub fn effective_sample_size(trace: &[f64]) -> f64 {
    let n = trace.len();
    if n < 4 {
        return n as f64;
    }
    let mean: f64 = trace.iter().sum::<f64>() / (n as f64);
    let centered: Vec<f64> = trace.iter().map(|x| x - mean).collect();
    let var: f64 = centered.iter().map(|x| x * x).sum::<f64>() / (n as f64);
    if var == 0.0 {
        return n as f64;
    }
    // Compute autocorrelations by direct sum (small N, no FFT needed).
    let autocorr_at = |lag: usize| -> f64 {
        if lag >= n {
            return 0.0;
        }
        let mut s = 0.0;
        for i in 0..(n - lag) {
            s += centered[i] * centered[i + lag];
        }
        s / ((n as f64) * var)
    };
    // Initial monotone sequence: sum pairs (rho_{2k+1} + rho_{2k+2}) while
    // the pair sum is positive AND monotonically decreasing.
    let mut sum_rho = 0.0_f64;
    let mut prev_pair = f64::INFINITY;
    let max_lag = (n / 4).max(2);
    let mut k = 0usize;
    while 2 * k + 2 <= max_lag {
        let pair = autocorr_at(2 * k + 1) + autocorr_at(2 * k + 2);
        if pair <= 0.0 {
            break;
        }
        // Enforce monotone decrease — clamp to prev_pair if the pair would
        // increase (Geyer's "initial monotone sequence" refinement).
        let pair = pair.min(prev_pair);
        sum_rho += pair;
        prev_pair = pair;
        k += 1;
    }
    let denom = 1.0 + 2.0 * sum_rho;
    if denom <= 0.0 {
        n as f64 // numerical guard
    } else {
        (n as f64) / denom
    }
}

/// Build an EssRecord per metric. Each metric trace is a SINGLE concatenated
/// trace (the caller may concatenate post-burn-in samples across chains, OR
/// pass per-chain traces and average; this module computes the simple per-
/// trace ESS).
pub fn ess_records(metrics: &[(String, &[f64])]) -> Vec<EssRecord> {
    metrics
        .iter()
        .map(|(name, trace)| {
            let ess = effective_sample_size(trace);
            EssRecord {
                metric: name.clone(),
                ess,
                n_total: trace.len(),
                computed_on: "summary_statistic".to_string(),
                below_threshold: ess < 100.0,
            }
        })
        .collect()
}

// ===========================================================================
// Hamming-distance autocorrelation (partition-space mixing)
// ===========================================================================

/// JSON-shape for `hamming_autocorr.json` (per chain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HammingAutocorrRecord {
    pub chain_idx: usize,
    /// Mean Hamming distance per lag, indexed by lag (0..max_lag inclusive).
    pub autocorr_per_lag: Vec<f64>,
    /// Integrated autocorrelation time (sum of (1 - normalized_hamming) until
    /// the increment becomes non-positive; bounded by `max_lag`). Units are
    /// "effective samples per actual sample" — higher means slower mixing.
    pub tau_int: f64,
}

/// Compute the per-lag mean Hamming distance for one partition trajectory.
///
/// `partitions` is a `Vec<Vec<usize>>` where `partitions[t][u]` is the
/// district id of unit `u` at MCMC step `t`. Every partition MUST have the
/// same number of units.
///
/// `max_lag` caps the lag range (typical: `partitions.len() / 4` or 100).
///
/// At lag `k`, the value is the mean over `t` of `hamming(p_t, p_{t+k}) /
/// n_units`, ranging from 0.0 (identical) to 1.0 (every unit reassigned).
pub fn hamming_autocorrelation(
    partitions: &[Vec<usize>],
    max_lag: usize,
) -> Result<Vec<f64>, DiagnosticsError> {
    let t = partitions.len();
    if t == 0 {
        return Err(DiagnosticsError::EmptyTrajectory);
    }
    let n_units = partitions[0].len();
    for (i, p) in partitions.iter().enumerate() {
        if p.len() != n_units {
            return Err(DiagnosticsError::PartitionLengthMismatch {
                first: n_units,
                idx: i,
                got: p.len(),
            });
        }
    }
    let max_lag = max_lag.min(t.saturating_sub(1));
    let mut out = Vec::with_capacity(max_lag + 1);
    out.push(0.0); // lag 0: distance to self is always 0
    for lag in 1..=max_lag {
        let mut total = 0.0;
        let pairs = t - lag;
        for i in 0..pairs {
            let mut diff = 0usize;
            for u in 0..n_units {
                if partitions[i][u] != partitions[i + lag][u] {
                    diff += 1;
                }
            }
            total += (diff as f64) / (n_units as f64);
        }
        out.push(if pairs > 0 { total / (pairs as f64) } else { 0.0 });
    }
    Ok(out)
}

/// Compute the integrated autocorrelation time `tau_int` from a Hamming
/// autocorrelation series. Convention: treat `1 - h(lag)` as the
/// autocorrelation function (= 1 at lag 0, decays toward 0 as the chain
/// mixes). Sum positive contributions only, capped at the supplied
/// `autocorr_per_lag.len()`.
///
/// `tau_int = 1 + 2 * sum_{k=1..K} (1 - h_k)` where `K` is the smallest lag
/// at which `(1 - h_k) <= 0` (i.e., the chain has fully mixed).
pub fn integrated_autocorrelation_time(autocorr_per_lag: &[f64]) -> f64 {
    if autocorr_per_lag.len() <= 1 {
        return 1.0;
    }
    let mut tau = 1.0;
    for &h in &autocorr_per_lag[1..] {
        let rho = 1.0 - h;
        if rho <= 0.0 {
            break;
        }
        tau += 2.0 * rho;
    }
    tau
}

/// Build `HammingAutocorrRecord` for each chain in `partitions_per_chain`.
pub fn hamming_records(
    partitions_per_chain: &[Vec<Vec<usize>>],
    max_lag: usize,
) -> Result<Vec<HammingAutocorrRecord>, DiagnosticsError> {
    partitions_per_chain
        .iter()
        .enumerate()
        .map(|(idx, parts)| {
            let autocorr = hamming_autocorrelation(parts, max_lag)?;
            let tau = integrated_autocorrelation_time(&autocorr);
            Ok(HammingAutocorrRecord {
                chain_idx: idx,
                autocorr_per_lag: autocorr,
                tau_int: tau,
            })
        })
        .collect()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Gelman-Rubin R-hat ─────────────────────────────────────────────────

    #[test]
    fn test_rhat_requires_4_chains() {
        let c1 = vec![1.0; 50];
        let c2 = vec![1.0; 50];
        let c3 = vec![1.0; 50];
        let chains = vec![c1.as_slice(), c2.as_slice(), c3.as_slice()];
        match gelman_rubin_rhat(&chains) {
            Err(DiagnosticsError::InsufficientChains(3)) => {}
            other => panic!("expected InsufficientChains(3), got {other:?}"),
        }
    }

    #[test]
    fn test_rhat_identical_chains_returns_1() {
        // 4 chains, every sample identical -> within-chain variance is 0;
        // between-chain variance is also 0 -> conventional R-hat = 1.0.
        let c = vec![5.0; 50];
        let chains: Vec<&[f64]> = (0..4).map(|_| c.as_slice()).collect();
        let r = gelman_rubin_rhat(&chains).unwrap();
        assert!((r - 1.0).abs() < 1e-9, "identical chains -> R-hat ~ 1; got {r}");
    }

    #[test]
    fn test_rhat_well_mixed_chains_close_to_1() {
        // 4 chains drawn from the same distribution -> R-hat near 1.
        let mut chains_owned: Vec<Vec<f64>> = Vec::new();
        let mut rng_state: u64 = 0x1234_5678;
        let mut rand = || -> f64 {
            // Linear congruential generator (cheap; deterministic across platforms).
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_state >> 33) as f64) / ((1u64 << 31) as f64) - 1.0
        };
        for _ in 0..4 {
            let c: Vec<f64> = (0..1000).map(|_| rand()).collect();
            chains_owned.push(c);
        }
        let chains: Vec<&[f64]> = chains_owned.iter().map(|c| c.as_slice()).collect();
        let r = gelman_rubin_rhat(&chains).unwrap();
        assert!(r < 1.10, "well-mixed chains should have R-hat < 1.10; got {r}");
    }

    #[test]
    fn test_rhat_diverged_chains_above_threshold() {
        // 4 chains centered on different means -> R-hat substantially above 1.
        let chains_owned: Vec<Vec<f64>> = (0..4)
            .map(|i| (0..200).map(|j| (i as f64) * 10.0 + (j as f64) * 0.001).collect())
            .collect();
        let chains: Vec<&[f64]> = chains_owned.iter().map(|c| c.as_slice()).collect();
        let r = gelman_rubin_rhat(&chains).unwrap();
        assert!(r > 1.05, "diverged chains should have R-hat > 1.05; got {r}");
    }

    #[test]
    fn test_rhat_unequal_chain_lengths_rejected() {
        let c1 = vec![1.0; 50];
        let c2 = vec![1.0; 50];
        let c3 = vec![1.0; 50];
        let c4 = vec![1.0; 49];
        let chains: Vec<&[f64]> = vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()];
        assert!(matches!(
            gelman_rubin_rhat(&chains),
            Err(DiagnosticsError::UnequalChainLengths(_))
        ));
    }

    #[test]
    fn test_rhat_record_shape() {
        let c1 = vec![1.0; 100];
        let c2 = vec![1.0; 100];
        let c3 = vec![1.0; 100];
        let c4 = vec![1.0; 100];
        let metrics = vec![(
            "efficiency_gap".to_string(),
            vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()],
        )];
        let recs = rhat_records(&metrics).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].metric, "efficiency_gap");
        assert_eq!(recs[0].n_chains, 4);
        assert_eq!(recs[0].n_per_chain, 100);
        assert_eq!(recs[0].threshold, 1.05);
        assert!(!recs[0].above_threshold, "identical chains -> not above threshold");
    }

    // ── Effective Sample Size ──────────────────────────────────────────────

    #[test]
    fn test_ess_iid_trace_close_to_n() {
        // Independent samples should give ESS ~ N (autocorrelations near zero).
        let mut rng_state: u64 = 42;
        let mut rand = || -> f64 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_state >> 33) as f64) / ((1u64 << 31) as f64) - 1.0
        };
        let trace: Vec<f64> = (0..1000).map(|_| rand()).collect();
        let ess = effective_sample_size(&trace);
        assert!(ess > 500.0, "iid trace should have ESS > N/2; got {ess} of {}", trace.len());
    }

    #[test]
    fn test_ess_highly_autocorrelated_trace_below_n() {
        // AR(1) with high persistence: x_t = 0.9 * x_{t-1} + noise.
        // ESS should be much smaller than N.
        let mut rng_state: u64 = 1;
        let mut rand = || -> f64 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_state >> 33) as f64) / ((1u64 << 31) as f64) - 1.0
        };
        let mut trace = Vec::with_capacity(1000);
        let mut x: f64 = 0.0;
        for _ in 0..1000 {
            x = 0.9 * x + rand();
            trace.push(x);
        }
        let ess = effective_sample_size(&trace);
        assert!(ess < 200.0, "AR(0.9) should give small ESS; got {ess} of {}", trace.len());
        assert!(ess > 0.0);
    }

    #[test]
    fn test_ess_constant_trace_returns_n() {
        // No variance -> ESS conventionally = N.
        let trace = vec![5.0; 100];
        let ess = effective_sample_size(&trace);
        assert_eq!(ess, 100.0);
    }

    #[test]
    fn test_ess_short_trace_returns_n() {
        // Below the autocorrelation lag minimum -> just N.
        let trace = vec![1.0, 2.0, 3.0];
        let ess = effective_sample_size(&trace);
        assert_eq!(ess, 3.0);
    }

    #[test]
    fn test_ess_records_flag_below_threshold() {
        // Highly autocorrelated trace with N=200 -> ESS small enough to flag.
        let mut rng_state: u64 = 7;
        let mut rand = || -> f64 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_state >> 33) as f64) / ((1u64 << 31) as f64) - 1.0
        };
        let mut trace = Vec::with_capacity(200);
        let mut x: f64 = 0.0;
        for _ in 0..200 {
            x = 0.95 * x + rand();
            trace.push(x);
        }
        let recs = ess_records(&[("eff_gap".to_string(), trace.as_slice())]);
        assert_eq!(recs[0].computed_on, "summary_statistic", "S-03: ESS on summary statistics, NOT partitions");
        assert!(recs[0].ess < 100.0, "highly autocorrelated trace should flag below_threshold; got ess={}", recs[0].ess);
        assert!(recs[0].below_threshold);
    }

    // ── Hamming-distance autocorrelation ───────────────────────────────────

    #[test]
    fn test_hamming_lag_0_is_zero() {
        let parts = vec![
            vec![1, 1, 2, 2],
            vec![1, 2, 2, 1],
        ];
        let h = hamming_autocorrelation(&parts, 1).unwrap();
        assert_eq!(h[0], 0.0, "lag 0 must be zero");
    }

    #[test]
    fn test_hamming_lag_1_simple_case() {
        // 4 units; partition swaps units 1 and 3 between t=0 and t=1.
        // Hamming distance = 2/4 = 0.5.
        let parts = vec![
            vec![1, 1, 2, 2],
            vec![1, 2, 2, 1],
        ];
        let h = hamming_autocorrelation(&parts, 1).unwrap();
        assert_eq!(h[1], 0.5);
    }

    #[test]
    fn test_hamming_identical_partitions_give_zero() {
        let p = vec![1, 1, 2, 2, 3, 3];
        let parts = vec![p.clone(), p.clone(), p.clone(), p.clone()];
        let h = hamming_autocorrelation(&parts, 3).unwrap();
        for (lag, v) in h.iter().enumerate() {
            assert_eq!(*v, 0.0, "identical partitions: all lags must be 0; lag {lag} got {v}");
        }
    }

    #[test]
    fn test_hamming_full_reassignment_gives_one() {
        // Every unit moves at every step.
        let parts = vec![
            vec![1, 1, 1, 1],
            vec![2, 2, 2, 2],
            vec![1, 1, 1, 1],
            vec![2, 2, 2, 2],
        ];
        let h = hamming_autocorrelation(&parts, 1).unwrap();
        assert_eq!(h[1], 1.0, "full reassignment: lag-1 distance is 1.0");
    }

    #[test]
    fn test_hamming_partition_length_mismatch_rejected() {
        let parts = vec![vec![1, 2, 3], vec![1, 2]];
        assert!(matches!(
            hamming_autocorrelation(&parts, 1),
            Err(DiagnosticsError::PartitionLengthMismatch { .. })
        ));
    }

    #[test]
    fn test_hamming_empty_trajectory_rejected() {
        let parts: Vec<Vec<usize>> = Vec::new();
        assert!(matches!(
            hamming_autocorrelation(&parts, 1),
            Err(DiagnosticsError::EmptyTrajectory)
        ));
    }

    #[test]
    fn test_tau_int_well_mixed_chain_close_to_1() {
        // Well-mixed: hamming distance saturates near 1.0 from lag 1.
        // 1 - h_k ~ 0 for all k > 0 -> tau_int ~ 1.
        // For h = [0.0, 0.99, 0.99, 0.99, 1.0]:
        // tau = 1 + 2*0.01 + 2*0.01 + 2*0.01 = 1.06 (lag 4 stops the sum since 1-1.0=0)
        let h = vec![0.0, 0.99, 0.99, 0.99, 1.0];
        let tau = integrated_autocorrelation_time(&h);
        assert!(tau < 1.10, "well-mixed chain: tau_int ~ 1; got {tau}");
    }

    #[test]
    fn test_tau_int_slow_mixing_chain_above_1() {
        // Slow mixing: hamming stays near 0 for many lags -> 1 - h_k ~ 1
        // -> large tau_int.
        let h = vec![0.0, 0.05, 0.10, 0.15, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 1.0];
        let tau = integrated_autocorrelation_time(&h);
        assert!(tau > 5.0, "slow mixing should give large tau_int; got {tau}");
    }

    #[test]
    fn test_tau_int_lag_0_only_returns_1() {
        let h = vec![0.0];
        assert_eq!(integrated_autocorrelation_time(&h), 1.0);
    }

    #[test]
    fn test_hamming_records_per_chain_shape() {
        let chain1 = vec![vec![1, 2, 1], vec![1, 2, 1], vec![2, 1, 2]];
        let chain2 = vec![vec![1, 2, 1], vec![2, 1, 2], vec![1, 2, 1]];
        let recs = hamming_records(&[chain1, chain2], 2).unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].chain_idx, 0);
        assert_eq!(recs[1].chain_idx, 1);
        assert_eq!(recs[0].autocorr_per_lag.len(), 3); // lags 0, 1, 2
        assert!(recs[0].tau_int >= 1.0);
    }
}
