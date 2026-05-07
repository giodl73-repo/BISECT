//! MultiScaleChain -- full algorithm stub. Implemented in task #128.

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MultiScaleConfig {
    pub total_steps: usize,
    pub alpha: f64,         // P(coarse move per step). Default: 0.3
    pub pop_tolerance: f64, // Fine-level balance tolerance. Default: 0.005
    pub coarse_tol: f64,    // Coarse-level tolerance = 2 x pop_tolerance by default
    pub base_seed: u64,
    pub chain_idx: u32,
}

impl Default for MultiScaleConfig {
    fn default() -> Self {
        let pop_tolerance = 0.005;
        Self {
            total_steps: 2000,
            alpha: 0.3,
            pop_tolerance,
            coarse_tol: 2.0 * pop_tolerance,
            base_seed: 0,
            chain_idx: 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum MultiScaleError {
    #[error("invalid configuration: {msg}")]
    InvalidConfig { msg: String },
    #[error("block-group adjacency not provided -- run: redist fetch --resolution block_group")]
    MissingCoarseAdjacency,
}

/// Multi-scale MCMC chain. Full implementation in task #128.
pub struct MultiScaleChain {
    pub fine_adj: Vec<Vec<usize>>,
    pub fine_pop: Vec<i64>,
    pub assignment: Vec<u32>,
    pub k: u32,
    pub config: MultiScaleConfig,
    pub steps_taken: u64,
    pub fine_steps: u64,
    pub coarse_steps: u64,
}

impl MultiScaleChain {
    pub fn new(_fine_adj: Vec<Vec<usize>>, _fine_pop: Vec<i64>,
               _assignment: Vec<u32>, _k: u32, _config: MultiScaleConfig) -> Self {
        unimplemented!("MultiScaleChain::new -- task #128")
    }
}

/// Configuration for AdaptiveMultiScaleChain — Robbins-Monro self-tuning alpha.
///
/// Extends MultiScaleConfig by replacing the fixed alpha with adaptive tuning:
/// coarse-move probability alpha is updated every `adapt_interval` steps toward
/// `target_accept` using a decaying Robbins-Monro step size gamma_t = gamma_0 / sqrt(t).
///
/// coarse_tol = 3 × pop_tolerance (looser than MultiScaleConfig's 2×) to avoid
/// over-rejection during early adaptation when alpha may be poorly calibrated.
///
/// See spec: docs/specs/2026-05-07-adaptive-multiscale.md
#[derive(Debug, Clone)]
pub struct AdaptiveMultiScaleConfig {
    pub total_steps: usize,
    pub target_accept: f64,    // target coarse acceptance rate (default: 0.30)
    pub initial_alpha: f64,    // starting alpha (default: 0.30)
    pub adapt_interval: usize, // steps between alpha updates (default: 50)
    pub gamma_0: f64,          // Robbins-Monro step size (default: 0.10)
    pub pop_tolerance: f64,    // fine-level balance tolerance (default: 0.005)
    pub coarse_tol: f64,       // = 3 × pop_tolerance by default
    pub p: f64,                // percentile of visited plans (default: 0.0)
    pub base_seed: u64,
    pub chain_idx: u32,
}

impl Default for AdaptiveMultiScaleConfig {
    fn default() -> Self {
        let pop_tolerance = 0.005;
        Self {
            total_steps: 2000,
            target_accept: 0.30,
            initial_alpha: 0.30,
            adapt_interval: 50,
            gamma_0: 0.10,
            pop_tolerance,
            coarse_tol: 3.0 * pop_tolerance,
            p: 0.0,
            base_seed: 0,
            chain_idx: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiscale_config_coarse_tol_is_twice_pop_tolerance() {
        let cfg = MultiScaleConfig::default();
        assert!((cfg.coarse_tol - 2.0 * cfg.pop_tolerance).abs() < 1e-12,
            "coarse_tol must be 2 x pop_tolerance by default");
    }

    #[test]
    fn multiscale_config_custom_pop_tolerance() {
        let pop_tolerance = 0.01;
        let cfg = MultiScaleConfig {
            pop_tolerance,
            coarse_tol: 2.0 * pop_tolerance,
            ..Default::default()
        };
        assert!((cfg.coarse_tol - 0.02).abs() < 1e-12);
    }

    #[test]
    fn adaptive_config_coarse_tol_is_three_times_pop_tolerance() {
        let cfg = AdaptiveMultiScaleConfig::default();
        assert!(
            (cfg.coarse_tol - 3.0 * cfg.pop_tolerance).abs() < 1e-12,
            "AdaptiveMultiScaleConfig::default() coarse_tol must be 3 x pop_tolerance, \
             got coarse_tol={} pop_tolerance={}", cfg.coarse_tol, cfg.pop_tolerance
        );
    }

    #[test]
    fn adaptive_config_custom_pop_tolerance() {
        let pop_tolerance = 0.01;
        let cfg = AdaptiveMultiScaleConfig {
            pop_tolerance,
            coarse_tol: 3.0 * pop_tolerance,
            ..Default::default()
        };
        assert!(
            (cfg.coarse_tol - 0.03).abs() < 1e-12,
            "custom pop_tolerance=0.01 must yield coarse_tol=0.03, got {}", cfg.coarse_tol
        );
    }
}
