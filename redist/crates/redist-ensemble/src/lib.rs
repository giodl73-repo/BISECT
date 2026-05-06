//! `redist-ensemble` — Rust ReCom feasibility sampler.
//!
//! Implements the Recombination (ReCom) Markov Chain Monte Carlo proposal
//! for exploring the space of valid redistricting plans (DeFord, Duchin &
//! Solomon 2021). Uses Wilson's loop-erased random walk for uniform random
//! spanning trees.

pub mod spanning;
pub mod recom;
pub mod chain;
