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
}
