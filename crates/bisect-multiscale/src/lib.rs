//! `redist-multiscale` — Multi-scale MCMC redistricting sampler.
//!
//! Interleaves fine-level (tract) and coarse-level (block-group) ReCom moves
//! to improve mixing for large-k states (TX k=38, CA k=52).
//! Per spec: docs/specs/2026-05-07-multiscale-mcmc.md (Accepted, R2 avg 3.0/4)

pub mod hierarchy;
pub mod chain;
pub mod seeds;
pub mod rebalance;

pub use chain::{MultiScaleChain, MultiScaleConfig, MultiScaleError};
