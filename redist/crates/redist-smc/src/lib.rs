//! `redist-smc` — Sequential Monte Carlo redistricting sampler.
//!
//! Implements the Fifield, Imai, Kawahara & Kenny (2020) SMC algorithm for
//! generating a calibrated weighted sample from the space of valid k-district
//! redistricting plans. Unlike ReCom (a Markov chain approximation), SMC
//! produces importance-weighted plans that correctly represent the uniform
//! distribution over all valid plans, without mixing assumptions.
//!
//! Spec: docs/specs/2026-05-07-smc-redistricting.md (Accepted, R2 avg 3.1/4)

pub mod seeds;
pub mod resample;
pub mod partial_plan;
pub mod proposal;
pub mod algorithm;
pub mod output;

pub use algorithm::{run_smc, SmcConfig, SmcError};
pub use output::{SmcResult, WriteConfig};
