//! `bisect-ensemble` — Rust ReCom feasibility sampler.
//!
//! Implements the Recombination (ReCom) Markov Chain Monte Carlo proposal
//! for exploring the space of valid redistricting plans (DeFord, Duchin &
//! Solomon 2021). Uses Wilson's loop-erased random walk for uniform random
//! spanning trees.

pub mod block_input;
pub mod chain;
pub mod evidence_manifest;
pub mod forest_recom;
pub mod merge_split;
pub mod parallel_tempering;
pub mod recom;
pub mod search_evidence;
pub mod short_burst_evidence;
pub mod spanning;
pub mod vra_recom;

pub use forest_recom::ForestRecomChain;
pub use merge_split::MergeSplitChain;
pub use parallel_tempering::ParallelTemperingChain;
pub use vra_recom::VraRecomChain;

/// Fixed-width draws used by portable browser/native chain entry points.
pub(crate) fn portable_shuffle<T, R: rand::Rng>(values: &mut [T], rng: &mut R) {
    for end in (1..values.len()).rev() {
        let index = rng.gen_range(0..=end as u64) as usize;
        values.swap(end, index);
    }
}
pub(crate) fn portable_index<R: rand::Rng>(length: usize, rng: &mut R) -> usize {
    rng.gen_range(0..length as u64) as usize
}
