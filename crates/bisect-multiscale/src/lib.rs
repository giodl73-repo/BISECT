//! `BISECT-multiscale` — Multi-scale MCMC redistricting sampler.
//!
//! Interleaves fine-level (tract) and coarse-level (block-group) ReCom moves
//! to improve mixing for large-k states (TX k=38, CA k=52).
//! Per spec: docs/specs/2026-05-07-multiscale-mcmc.md (Accepted, R2 avg 3.0/4)

pub mod adaptive_evidence;
pub mod chain;
pub mod hierarchy;
pub mod rebalance;
pub mod resolution_evidence;
pub mod seeds;

pub use chain::{MultiScaleChain, MultiScaleConfig, MultiScaleError};
