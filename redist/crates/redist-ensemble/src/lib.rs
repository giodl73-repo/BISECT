//! `redist-ensemble` — Rust ReCom feasibility sampler.
//!
//! Implements the Recombination (ReCom) Markov Chain Monte Carlo proposal
//! for exploring the space of valid redistricting plans (DeFord, Duchin &
//! Solomon 2021). Uses Wilson's loop-erased random walk for uniform random
//! spanning trees.

pub mod spanning;
pub mod recom;
pub mod chain;
pub mod merge_split;
pub mod forest_recom;
pub mod parallel_tempering;
pub mod vra_recom;

pub use merge_split::MergeSplitChain;
pub use forest_recom::ForestRecomChain;
pub use parallel_tempering::ParallelTemperingChain;
pub use vra_recom::VraRecomChain;
