//! SmcResult and NDJSON serialisation. Per spec §3.2 and §6.
//! Full NDJSON streaming with file_sha256 implemented in task #110.

use serde::{Deserialize, Serialize};

/// A resampling event record — embedded in the NDJSON stream between stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResampleRecord {
    #[serde(rename = "type")]
    pub record_type: String,     // "resample"
    pub stage: usize,
    pub resample_round: u32,
    pub ess_before: f64,
    pub resample_seed: u64,
    pub index_map: Vec<usize>,   // new_particle[j] came from old_particle[index_map[j]]
}

/// Final metadata record — last line of the NDJSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRecord {
    #[serde(rename = "type")]
    pub record_type: String,     // "metadata"
    pub base_seed: u64,
    pub n_particles: usize,
    pub resample_threshold: f64,
    pub pop_tolerance: f64,
    pub k: usize,
    pub resample_count: u32,
    pub resample_rounds: Vec<usize>,  // stages at which resampling occurred
    pub ess_trace: Vec<f64>,          // ESS after each of k-1 proposal stages
    pub particle_seed_formula: String,
    pub resample_seed_formula: String,
    pub smc_version: String,
    pub ensemble_output_version: String,
    pub file_sha256: String,     // SHA-256 of all preceding NDJSON lines (LF-normalised)
}

/// The output of a complete SMC run.
#[derive(Debug, Clone)]
pub struct SmcResult {
    /// N complete plans, each a Vec<u32> of length n_tracts (1-based district IDs)
    pub plans: Vec<Vec<u32>>,
    /// Normalised importance weights, sum = 1.0 ± 1e-6 (Kahan)
    pub weights: Vec<f64>,
    /// Number of resampling events
    pub resample_count: u32,
    /// Stages at which resampling occurred (1-based stage indices)
    pub resample_rounds: Vec<usize>,
    /// ESS after each of k-1 proposal stages
    pub ess_trace: Vec<f64>,
    /// Resampling index maps — one per resample event
    pub index_maps: Vec<Vec<usize>>,
    /// Configuration used
    pub base_seed: u64,
    pub n_particles: usize,
}

impl SmcResult {
    pub fn n_plans(&self) -> usize { self.plans.len() }
    pub fn k(&self) -> u32 {
        self.plans.first()
            .and_then(|p| p.iter().copied().max())
            .unwrap_or(0)
    }
}
