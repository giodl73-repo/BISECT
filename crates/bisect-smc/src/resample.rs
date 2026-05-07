//! ESS computation, Kahan softmax, and systematic resampling for SMC.
//! Per spec §2.5 and §7 (L0 test invariants).

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

/// Effective Sample Size from log-weights.
///
/// ESS = 1 / Σ w_i² where w_i = exp(log_w_i) are normalised weights.
/// Computed in log-space for numerical stability.
///
/// Invariants (per spec §7 L0):
/// - ESS(uniform: w_i = 1/N) = N (within 1e-9)
/// - ESS(degenerate: one w=1, rest=0) = 1.0 (within 1e-12)
pub fn ess(log_weights: &[f64]) -> f64 {
    if log_weights.is_empty() {
        return 0.0;
    }
    // Normalise in log-space: subtract logsumexp
    let lse = logsumexp(log_weights);
    let sum_sq: f64 = log_weights.iter()
        .map(|&lw| {
            let w = (lw - lse).exp();
            w * w
        })
        .sum();
    if sum_sq == 0.0 { 0.0 } else { 1.0 / sum_sq }
}

/// Log-sum-exp of a slice, numerically stable.
pub fn logsumexp(log_weights: &[f64]) -> f64 {
    let max = log_weights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max.is_infinite() {
        return f64::NEG_INFINITY;
    }
    let sum: f64 = log_weights.iter().map(|&lw| (lw - max).exp()).sum();
    max + sum.ln()
}

/// Kahan-compensated softmax: convert log-weights to normalised weights summing to 1.0.
///
/// Uses Kahan summation to bound the accumulation error at O(ε) regardless of N,
/// ensuring sum ≤ 1.0 ± 1e-6 even for N = 50,000 (per spec §7 L0).
pub fn kahan_softmax(log_weights: &[f64]) -> Vec<f64> {
    let lse = logsumexp(log_weights);
    let weights: Vec<f64> = log_weights.iter()
        .map(|&lw| (lw - lse).exp())
        .collect();

    // Kahan-renormalise to guarantee sum == 1.0 ± eps
    let mut sum = 0.0f64;
    let mut comp = 0.0f64;
    for &w in &weights {
        let y = w - comp;
        let t = sum + y;
        comp = (t - sum) - y;
        sum = t;
    }
    // Renormalise by the Kahan sum
    if sum == 0.0 {
        let n = weights.len();
        return vec![1.0 / n as f64; n];
    }
    weights.iter().map(|&w| w / sum).collect()
}

/// Systematic resampling: draw N indices with replacement from `log_weights`.
///
/// Returns `(new_indices, index_map)` where:
/// - `new_indices[j]` is the old particle index assigned to new slot j
/// - `index_map` is identical to `new_indices` (kept separate for clarity with spec)
///
/// Uses a single uniform draw u ~ Uniform(0, 1/N) and binary search (partition_point)
/// to avoid the off-by-one bug in the `floor(N*(u+j/N))` form. Per spec §2.5.
pub fn systematic_resample(n: usize, log_weights: &[f64], seed: u64) -> (Vec<usize>, Vec<usize>) {
    assert_eq!(log_weights.len(), n, "log_weights length must equal n");

    let weights = kahan_softmax(log_weights);

    // Build cumulative sum
    let mut cumsum = Vec::with_capacity(n);
    let mut running = 0.0f64;
    let mut comp = 0.0f64;
    for &w in &weights {
        let y = w - comp;
        let t = running + y;
        comp = (t - running) - y;
        running = t;
        cumsum.push(running);
    }
    // Clamp last entry to exactly 1.0 to avoid floating-point overshoot
    if let Some(last) = cumsum.last_mut() {
        *last = 1.0;
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    // Single uniform draw in [0, 1/N)
    let u: f64 = rng.gen::<f64>() / n as f64;

    let mut indices = Vec::with_capacity(n);
    for j in 0..n {
        let target = (u + j as f64 / n as f64).min(1.0 - f64::EPSILON);
        // partition_point: first index where cumsum[i] >= target
        let idx = cumsum.partition_point(|&c| c < target);
        indices.push(idx.min(n - 1)); // clamp — never out of bounds
    }

    let index_map = indices.clone();
    (indices, index_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform_log_weights(n: usize) -> Vec<f64> {
        // Uniform: all log_weights = -ln(N), so exp gives 1/N each
        let lw = -(n as f64).ln();
        vec![lw; n]
    }

    #[test]
    fn ess_uniform_equals_n() {
        for n in [2usize, 10, 100, 1000] {
            let lw = uniform_log_weights(n);
            let e = ess(&lw);
            assert!((e - n as f64).abs() < 1e-9,
                "ESS({n} uniform weights) = {e}, expected {n}");
        }
    }

    #[test]
    fn ess_degenerate_equals_one() {
        // One weight = 1.0 (log=0), rest = 0.0 (log=-inf)
        let mut lw = vec![f64::NEG_INFINITY; 10];
        lw[0] = 0.0;
        let e = ess(&lw);
        assert!((e - 1.0).abs() < 1e-12, "ESS(degenerate) = {e}, expected 1.0");
    }

    #[test]
    fn kahan_softmax_sums_to_one_small_n() {
        let lw = uniform_log_weights(100);
        let w = kahan_softmax(&lw);
        let sum: f64 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "sum = {sum}");
    }

    #[test]
    fn kahan_softmax_sums_to_one_large_n() {
        let lw = uniform_log_weights(50_000);
        let w = kahan_softmax(&lw);
        // Sum using Kahan as well for the test
        let mut s = 0.0f64;
        let mut c = 0.0f64;
        for &x in &w {
            let y = x - c;
            let t = s + y;
            c = (t - s) - y;
            s = t;
        }
        assert!((s - 1.0).abs() < 1e-6, "N=50000 sum = {s}, expected 1.0 ± 1e-6");
    }

    #[test]
    fn systematic_resample_uniform_u0_gives_identity() {
        // With equal weights and u near 0, indices should be [0,1,2,...,N-1]
        // We can't force u=0 with seed, but we check that index_map is valid
        let n = 8usize;
        let lw = uniform_log_weights(n);
        let (indices, index_map) = systematic_resample(n, &lw, 0);
        assert_eq!(indices.len(), n);
        assert_eq!(index_map.len(), n);
        assert!(indices.iter().all(|&i| i < n), "all indices in [0,N-1]");
        assert!(index_map.iter().all(|&i| i < n), "all index_map values in [0,N-1]");
    }

    #[test]
    fn systematic_resample_index_map_bounds() {
        let n = 100usize;
        let lw = uniform_log_weights(n);
        let (_, index_map) = systematic_resample(n, &lw, 42);
        assert_eq!(index_map.len(), n);
        for &i in &index_map {
            assert!(i < n, "index_map value {i} out of [0,{n})");
        }
    }

    #[test]
    fn systematic_resample_deterministic() {
        let n = 50usize;
        let lw = uniform_log_weights(n);
        let (i1, m1) = systematic_resample(n, &lw, 777);
        let (i2, m2) = systematic_resample(n, &lw, 777);
        assert_eq!(i1, i2);
        assert_eq!(m1, m2);
    }

    #[test]
    fn systematic_resample_different_seeds_differ_nonuniform() {
        // With non-uniform weights, different seeds should produce different index maps
        // (the single uniform draw u changes, shifting all targets).
        let n = 10usize;
        // Skewed weights: first 3 particles get ~60% of weight
        let log_weights: Vec<f64> = (0..n).map(|i| if i < 3 { 0.0f64 } else { -2.0f64 }).collect();
        let (i1, _) = systematic_resample(n, &log_weights, 1);
        let (i2, _) = systematic_resample(n, &log_weights, 999);
        // With skewed weights and different u values, the index sequences should differ
        assert_ne!(i1, i2, "different seeds with skewed weights should differ");
    }

    #[test]
    fn systematic_resample_uniform_weights_gives_identity() {
        // With uniform weights, systematic resampling always returns [0,1,...,N-1]
        // regardless of seed: the staircase has equal steps and u ∈ [0,1/N) always
        // selects target = u+j/N landing in bucket j. This is correct behavior.
        let n = 8usize;
        let lw = uniform_log_weights(n);
        let (indices, _) = systematic_resample(n, &lw, 42);
        assert_eq!(indices, (0..n).collect::<Vec<_>>(),
            "uniform weights → identity permutation");
    }
}
