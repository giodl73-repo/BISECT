//! Permutation test for redistricting plan extremity.
//!
//! Given a GerryChain ensemble of N plans and a query plan, computes
//! the ESS-corrected p-value for the null hypothesis:
//! "the query plan is a random draw from the ensemble distribution."
//!
//! Used by S.1 (Hypothesis Testing for Partisan Gerrymandering).

use std::collections::HashMap;

/// A single plan in the ensemble, characterised by a scalar test statistic.
#[derive(Debug, Clone)]
pub struct EnsemblePlan {
    /// Plan identifier (e.g., step number or plan ID).
    pub id: u64,
    /// Test statistic value for this plan (e.g., normalized edge cut, efficiency gap).
    pub statistic: f64,
}

/// Result of a one-sided permutation test (query plan is an extreme low value).
#[derive(Debug, Clone)]
pub struct PermutationTestResult {
    /// Observed test statistic for the query plan.
    pub query_statistic: f64,
    /// Number of ensemble plans with statistic ≤ query_statistic.
    pub n_as_extreme: usize,
    /// Total number of ensemble plans.
    pub n_total: usize,
    /// Raw empirical p-value: n_as_extreme / n_total.
    pub p_value_raw: f64,
    /// ESS used for the correction (from G.4).
    pub ess: f64,
    /// ESS-corrected p-value using Beta(p_raw * ess, (1 - p_raw) * ess) posterior median.
    pub p_value_ess_corrected: f64,
    /// Bayesian Detection Score: P(true percentile < 0.05 | p_raw, ess).
    pub bds_at_0_05: f64,
}

impl PermutationTestResult {
    /// Returns true if the query plan is a statistically significant outlier
    /// at the 5% level (ESS-corrected p-value < 0.05).
    pub fn is_significant(&self) -> bool {
        self.p_value_ess_corrected < 0.05
    }
}

/// Run a one-sided permutation test: is the query plan unusually low?
///
/// # Arguments
/// * `query_statistic` — test statistic for the query plan (lower = more extreme)
/// * `ensemble` — slice of ensemble plans with their test statistics
/// * `ess` — effective sample size from G.4 (accounts for autocorrelation)
///
/// # Returns
/// [`PermutationTestResult`] with raw and ESS-corrected p-values.
pub fn permutation_test_lower_tail(
    query_statistic: f64,
    ensemble: &[EnsemblePlan],
    ess: f64,
) -> PermutationTestResult {
    let n_total = ensemble.len();
    let n_as_extreme = ensemble
        .iter()
        .filter(|p| p.statistic <= query_statistic)
        .count();

    let p_raw = n_as_extreme as f64 / n_total as f64;

    // ESS-corrected p-value: posterior median of Beta(p_raw*ess + 1, (1-p_raw)*ess + 1)
    // The posterior median ≈ (p_raw*ess + 1/3) / (ess + 2/3) for Beta distributions
    // (Bayes estimate with Jeffrey's prior adjustment).
    let a = p_raw * ess + 1.0;
    let b = (1.0 - p_raw) * ess + 1.0;
    let p_ess_corrected = (a - 1.0 / 3.0) / (a + b - 2.0 / 3.0);

    // BDS: P(true percentile < 0.05 | p_raw, ess)
    // Using regularized incomplete Beta function approximation.
    // For p_raw << 0.05: BDS ≈ 1 - Beta_CDF(0.05, a, b)
    // We use a simple numerical approximation via the continued fraction expansion.
    let bds = regularized_incomplete_beta(0.05, a, b);

    PermutationTestResult {
        query_statistic,
        n_as_extreme,
        n_total,
        p_value_raw: p_raw,
        ess,
        p_value_ess_corrected: p_ess_corrected.clamp(0.0, 1.0),
        bds_at_0_05: bds.clamp(0.0, 1.0),
    }
}

/// Regularised incomplete Beta function I_x(a, b) using continued fraction expansion.
/// Used to compute the Bayesian Detection Score.
fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    if x >= 1.0 { return 1.0; }

    // Use the continued fraction expansion (Lentz's method).
    // Switch sides if x > (a+1)/(a+b+2) for numerical stability.
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - regularized_incomplete_beta(1.0 - x, b, a);
    }

    let lbeta = lgamma(a) + lgamma(b) - lgamma(a + b);
    let front = (a * x.ln() + b * (1.0 - x).ln() - lbeta).exp() / a;

    front * beta_continued_fraction(x, a, b)
}

fn beta_continued_fraction(x: f64, a: f64, b: f64) -> f64 {
    let max_iter = 200;
    let eps = 1e-10;

    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < f64::MIN_POSITIVE { d = f64::MIN_POSITIVE; }
    d = 1.0 / d;
    let mut result = d;

    for m in 1..=max_iter {
        let m = m as f64;

        // Even step
        let dm = m * (b - m) * x / ((a + 2.0*m - 1.0) * (a + 2.0*m));
        d = 1.0 + dm * d;
        if d.abs() < f64::MIN_POSITIVE { d = f64::MIN_POSITIVE; }
        c = 1.0 + dm / c;
        if c.abs() < f64::MIN_POSITIVE { c = f64::MIN_POSITIVE; }
        d = 1.0 / d;
        result *= d * c;

        // Odd step
        let dm = -(a + m) * (a + b + m) * x / ((a + 2.0*m) * (a + 2.0*m + 1.0));
        d = 1.0 + dm * d;
        if d.abs() < f64::MIN_POSITIVE { d = f64::MIN_POSITIVE; }
        c = 1.0 + dm / c;
        if c.abs() < f64::MIN_POSITIVE { c = f64::MIN_POSITIVE; }
        d = 1.0 / d;
        let delta = d * c;
        result *= delta;

        if (delta - 1.0).abs() < eps { break; }
    }
    result
}

/// Natural log of Gamma function (Lanczos approximation).
fn lgamma(z: f64) -> f64 {
    let g = 7.0_f64;
    let c = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    if z < 0.5 {
        std::f64::consts::PI.ln() - ((std::f64::consts::PI * z).sin().ln()) - lgamma(1.0 - z)
    } else {
        let z = z - 1.0;
        let mut x = c[0];
        for (i, &ci) in c[1..].iter().enumerate() {
            x += ci / (z + i as f64 + 1.0);
        }
        let t = z + g + 0.5;
        0.5 * (2.0 * std::f64::consts::PI).ln()
            + (z + 0.5) * t.ln()
            - t
            + x.ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_test_lower_tail() {
        // Synthetic ensemble: 100 plans with statistic uniform in [0, 1]
        let ensemble: Vec<EnsemblePlan> = (0..100)
            .map(|i| EnsemblePlan {
                id: i,
                statistic: i as f64 / 100.0,
            })
            .collect();

        // Query plan at 0.003 (very extreme)
        let result = permutation_test_lower_tail(0.003, &ensemble, 70.0);

        assert_eq!(result.n_as_extreme, 1); // only plan 0 (0.000) is ≤ 0.003
        assert!((result.p_value_raw - 0.01).abs() < 0.005);
        assert!(result.p_value_ess_corrected < 0.10);
        assert!(result.bds_at_0_05 > 0.80, "BDS should be high for very extreme plan");
    }

    #[test]
    fn test_permutation_test_neutral_plan() {
        // Neutral plan at median
        let ensemble: Vec<EnsemblePlan> = (0..1000)
            .map(|i| EnsemblePlan { id: i, statistic: i as f64 / 1000.0 })
            .collect();

        let result = permutation_test_lower_tail(0.50, &ensemble, 500.0);
        assert!(result.p_value_raw > 0.40); // near 50th percentile
        assert!(!result.is_significant());
        assert!(result.bds_at_0_05 < 0.20);
    }

    #[test]
    fn test_regularized_incomplete_beta_boundary() {
        assert!((regularized_incomplete_beta(0.0, 2.0, 3.0) - 0.0).abs() < 1e-10);
        assert!((regularized_incomplete_beta(1.0, 2.0, 3.0) - 1.0).abs() < 1e-10);
        // I_0.5(2, 2) = 0.5 by symmetry
        assert!((regularized_incomplete_beta(0.5, 2.0, 2.0) - 0.5).abs() < 0.01);
    }
}
