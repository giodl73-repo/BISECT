pub mod mcmc {
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq, Eq)]
    pub enum DiagnosticsError {
        #[error("[INPUT] requires >=4 parallel chains for Gelman-Rubin R-hat; got {0}")]
        InsufficientChains(usize),
        #[error("[INPUT] empty chain at index {0}")]
        EmptyChain(usize),
        #[error("[INPUT] chains have differing lengths: {0:?}")]
        UnequalChainLengths(Vec<usize>),
        #[error("[INPUT] empty partition trajectory")]
        EmptyTrajectory,
        #[error(
            "[INPUT] partitions have differing unit counts: first={first}, at index {idx}={got}"
        )]
        PartitionLengthMismatch {
            first: usize,
            idx: usize,
            got: usize,
        },
    }

    pub fn gelman_rubin_rhat(chains: &[&[f64]]) -> Result<f64, DiagnosticsError> {
        let m = chains.len();
        if m < 4 {
            return Err(DiagnosticsError::InsufficientChains(m));
        }
        let n = chains[0].len();
        if n == 0 {
            return Err(DiagnosticsError::EmptyChain(0));
        }
        for (i, chain) in chains.iter().enumerate() {
            if chain.is_empty() {
                return Err(DiagnosticsError::EmptyChain(i));
            }
            if chain.len() != n {
                return Err(DiagnosticsError::UnequalChainLengths(
                    chains.iter().map(|c| c.len()).collect(),
                ));
            }
        }

        let chain_means: Vec<f64> = chains
            .iter()
            .map(|chain| chain.iter().sum::<f64>() / n as f64)
            .collect();
        let grand_mean = chain_means.iter().sum::<f64>() / m as f64;
        let chain_vars: Vec<f64> = chains
            .iter()
            .zip(&chain_means)
            .map(|(chain, mean)| {
                chain.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1).max(1) as f64
            })
            .collect();

        let b_over_n = chain_means
            .iter()
            .map(|mean| (mean - grand_mean).powi(2))
            .sum::<f64>()
            / (m - 1).max(1) as f64;
        let w = chain_vars.iter().sum::<f64>() / m as f64;
        if w == 0.0 {
            return Ok(1.0);
        }

        let n_f = n as f64;
        Ok((((n_f - 1.0) / n_f) * w + b_over_n).sqrt() / w.sqrt())
    }

    pub fn effective_sample_size(trace: &[f64]) -> f64 {
        let n = trace.len();
        if n < 4 {
            return n as f64;
        }
        let mean = trace.iter().sum::<f64>() / n as f64;
        let centered: Vec<f64> = trace.iter().map(|x| x - mean).collect();
        let var = centered.iter().map(|x| x * x).sum::<f64>() / n as f64;
        if var == 0.0 {
            return n as f64;
        }

        let autocorr_at = |lag: usize| -> f64 {
            if lag >= n {
                return 0.0;
            }
            let mut sum = 0.0;
            for i in 0..(n - lag) {
                sum += centered[i] * centered[i + lag];
            }
            sum / (n as f64 * var)
        };

        let mut sum_rho = 0.0_f64;
        let mut prev_pair = f64::INFINITY;
        let max_lag = (n / 4).max(2);
        let mut k = 0usize;
        while 2 * k + 2 <= max_lag {
            let pair = autocorr_at(2 * k + 1) + autocorr_at(2 * k + 2);
            if pair <= 0.0 {
                break;
            }
            let pair = pair.min(prev_pair);
            sum_rho += pair;
            prev_pair = pair;
            k += 1;
        }

        let denom = 1.0 + 2.0 * sum_rho;
        if denom <= 0.0 {
            n as f64
        } else {
            n as f64 / denom
        }
    }

    pub fn hamming_autocorrelation(
        partitions: &[Vec<usize>],
        max_lag: usize,
    ) -> Result<Vec<f64>, DiagnosticsError> {
        let t = partitions.len();
        if t == 0 {
            return Err(DiagnosticsError::EmptyTrajectory);
        }
        let n_units = partitions[0].len();
        for (idx, partition) in partitions.iter().enumerate() {
            if partition.len() != n_units {
                return Err(DiagnosticsError::PartitionLengthMismatch {
                    first: n_units,
                    idx,
                    got: partition.len(),
                });
            }
        }

        let max_lag = max_lag.min(t.saturating_sub(1));
        let mut out = Vec::with_capacity(max_lag + 1);
        out.push(0.0);
        for lag in 1..=max_lag {
            let mut total = 0.0;
            let pairs = t - lag;
            for i in 0..pairs {
                let mut diff = 0usize;
                for unit in 0..n_units {
                    if partitions[i][unit] != partitions[i + lag][unit] {
                        diff += 1;
                    }
                }
                total += diff as f64 / n_units as f64;
            }
            out.push(if pairs > 0 { total / pairs as f64 } else { 0.0 });
        }
        Ok(out)
    }

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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rhat_rejects_too_few_chains() {
            let chain = vec![1.0; 10];
            let chains = vec![chain.as_slice(), chain.as_slice(), chain.as_slice()];
            assert_eq!(
                gelman_rubin_rhat(&chains),
                Err(DiagnosticsError::InsufficientChains(3))
            );
        }

        #[test]
        fn rhat_identical_chains_is_one() {
            let chain = vec![5.0; 50];
            let chains: Vec<&[f64]> = (0..4).map(|_| chain.as_slice()).collect();
            assert!((gelman_rubin_rhat(&chains).unwrap() - 1.0).abs() < 1e-9);
        }

        #[test]
        fn ess_constant_trace_returns_n() {
            assert_eq!(effective_sample_size(&vec![5.0; 100]), 100.0);
        }

        #[test]
        fn hamming_rejects_length_mismatch() {
            let partitions = vec![vec![1, 2, 3], vec![1, 2]];
            assert!(matches!(
                hamming_autocorrelation(&partitions, 1),
                Err(DiagnosticsError::PartitionLengthMismatch { .. })
            ));
        }

        #[test]
        fn tau_lag_zero_only_is_one() {
            assert_eq!(integrated_autocorrelation_time(&[0.0]), 1.0);
        }
    }
}

pub mod probability {
    pub fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }

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
        if d.abs() < f64::MIN_POSITIVE {
            d = f64::MIN_POSITIVE;
        }
        d = 1.0 / d;
        let mut result = d;

        for m in 1..=max_iter {
            let m = m as f64;

            let dm = m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m));
            d = 1.0 + dm * d;
            if d.abs() < f64::MIN_POSITIVE {
                d = f64::MIN_POSITIVE;
            }
            c = 1.0 + dm / c;
            if c.abs() < f64::MIN_POSITIVE {
                c = f64::MIN_POSITIVE;
            }
            d = 1.0 / d;
            result *= d * c;

            let dm = -(a + m) * (a + b + m) * x / ((a + 2.0 * m) * (a + 2.0 * m + 1.0));
            d = 1.0 + dm * d;
            if d.abs() < f64::MIN_POSITIVE {
                d = f64::MIN_POSITIVE;
            }
            c = 1.0 + dm / c;
            if c.abs() < f64::MIN_POSITIVE {
                c = f64::MIN_POSITIVE;
            }
            d = 1.0 / d;
            let delta = d * c;
            result *= delta;

            if (delta - 1.0).abs() < eps {
                break;
            }
        }
        result
    }

    fn lgamma(z: f64) -> f64 {
        let g = 7.0_f64;
        let c = [
            0.9999999999998099,
            676.5203681218851,
            -1259.1392167224028,
            771.3234287776531,
            -176.6150291621406,
            12.507343278686905,
            -0.13857109526572012,
            9.984369578019572e-6,
            1.5056327351493116e-7,
        ];
        if z < 0.5 {
            std::f64::consts::PI.ln() - (std::f64::consts::PI * z).sin().ln() - lgamma(1.0 - z)
        } else {
            let z = z - 1.0;
            let mut x = c[0];
            for (i, &ci) in c[1..].iter().enumerate() {
                x += ci / (z + i as f64 + 1.0);
            }
            let t = z + g + 0.5;
            0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn beta_boundaries_are_exact() {
            assert!((regularized_incomplete_beta(0.0, 2.0, 3.0) - 0.0).abs() < 1e-10);
            assert!((regularized_incomplete_beta(1.0, 2.0, 3.0) - 1.0).abs() < 1e-10);
        }

        #[test]
        fn beta_symmetric_midpoint_is_half() {
            assert!((regularized_incomplete_beta(0.5, 2.0, 2.0) - 0.5).abs() < 0.01);
        }
    }
}
