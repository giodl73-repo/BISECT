//! Top-level SMC algorithm: run_smc(). Per spec §2.1.
//! Full implementation in task #109.

use thiserror::Error;
use crate::output::SmcResult;

#[derive(Debug, Clone)]
pub struct SmcConfig {
    pub n_particles: usize,
    pub resample_threshold: f64,   // ESS < threshold * N → resample
    pub pop_tolerance: f64,        // ±fraction of ideal population
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
    #[error("all {n_particles} particles killed at stage {stage} — ESS reached 0")]
    AllParticlesKilled { stage: usize, n_particles: usize },
    #[error("invalid configuration: {msg}")]
    InvalidConfig { msg: String },
}

/// Run the Sequential Monte Carlo redistricting sampler.
///
/// Returns a weighted sample of `config.n_particles` valid k-district plans.
/// Implementation in task #109.
pub fn run_smc(
    _adjacency: &[Vec<usize>],
    _vertex_weights: &[i64],
    _k: usize,
    _config: SmcConfig,
) -> Result<SmcResult, SmcError> {
    Err(SmcError::InvalidConfig { msg: "run_smc not yet implemented (task #109)".into() })
}
