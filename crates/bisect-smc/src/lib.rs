//! `bisect-smc` — Sequential Monte Carlo redistricting sampler.
//!
//! Runs the repository's sequential particle proposals, importance weights and
//! ESS-triggered resampling. Numerical diagnostics and a finite weighted sample
//! do not establish distribution calibration. Check final plan constraints
//! independently; proposal tolerance is based on remaining-component population.
//! Protocol 2 uses ChaCha12 and u64 index draws across native and WASM targets.
//!
//! Spec: docs/specs/2026-05-07-smc-redistricting.md (Accepted, R2 avg 3.1/4)

pub mod algorithm;
pub mod output;
pub mod partial_plan;
pub mod proposal;
pub mod resample;
pub mod seeds;

pub use algorithm::{run_smc, SmcConfig, SmcError};
pub use output::{SmcResult, WriteConfig};
