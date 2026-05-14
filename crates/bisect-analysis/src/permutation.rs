//! Permutation test for redistricting plan extremity.
//!
//! Given a GerryChain ensemble of N plans and a query plan, computes
//! the ESS-corrected p-value for the null hypothesis:
//! "the query plan is a random draw from the ensemble distribution."
//!
//! Used by S.1 (Hypothesis Testing for Partisan Gerrymandering).

use rstat_core::hypothesis::{
    bayesian_detection_score, empirical_p_value, ess_beta_median, HypothesisError, Tail,
};

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
) -> Result<PermutationTestResult, HypothesisError> {
    let reference: Vec<f64> = ensemble.iter().map(|p| p.statistic).collect();
    let (n_as_extreme, n_total, p_raw) =
        empirical_p_value(query_statistic, &reference, Tail::Lower)?;
    let p_ess_corrected = ess_beta_median(p_raw, ess)?;
    let bds = bayesian_detection_score(0.05, p_raw, ess)?;

    Ok(PermutationTestResult {
        query_statistic,
        n_as_extreme,
        n_total,
        p_value_raw: p_raw,
        ess,
        p_value_ess_corrected: p_ess_corrected.clamp(0.0, 1.0),
        bds_at_0_05: bds.clamp(0.0, 1.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstat_core::probability::ProbabilityError;

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
        let result = permutation_test_lower_tail(0.003, &ensemble, 70.0).unwrap();

        assert_eq!(result.n_as_extreme, 1); // only plan 0 (0.000) is ≤ 0.003
        assert!((result.p_value_raw - 0.01).abs() < 0.005);
        assert!(result.p_value_ess_corrected < 0.10);
        assert!(
            result.bds_at_0_05 > 0.80,
            "BDS should be high for very extreme plan"
        );
    }

    #[test]
    fn test_permutation_test_neutral_plan() {
        // Neutral plan at median
        let ensemble: Vec<EnsemblePlan> = (0..1000)
            .map(|i| EnsemblePlan {
                id: i,
                statistic: i as f64 / 1000.0,
            })
            .collect();

        let result = permutation_test_lower_tail(0.50, &ensemble, 500.0).unwrap();
        assert!(result.p_value_raw > 0.40); // near 50th percentile
        assert!(!result.is_significant());
        assert!(result.bds_at_0_05 < 0.20);
    }

    #[test]
    fn test_regularized_incomplete_beta_boundary() {
        assert!((bayesian_detection_score(0.0, 0.5, 10.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((bayesian_detection_score(1.0, 0.5, 10.0).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_permutation_rejects_overflowed_detection_score_shapes() {
        let ensemble: Vec<EnsemblePlan> = (0..100)
            .map(|i| EnsemblePlan {
                id: i,
                statistic: i as f64 / 100.0,
            })
            .collect();

        match permutation_test_lower_tail(0.50, &ensemble, f64::MAX) {
            Err(HypothesisError::Probability(ProbabilityError::NonFiniteResult {
                operation,
                value,
            })) => {
                assert_eq!(operation, "beta lgamma(a)");
                assert!(value.is_infinite());
            }
            other => panic!("expected beta shape overflow error, got {other:?}"),
        }
    }
}
