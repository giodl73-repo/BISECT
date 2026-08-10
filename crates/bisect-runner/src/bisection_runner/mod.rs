//! Deterministic recursive-bisection engine.
//!
//! Split from a single 10k-line file into per-algorithm submodules. Shared
//! imports and cross-section helpers are re-exported here so each submodule
//! can `use super::*;` and reach its siblings.

pub(crate) use bisect_core::{ufactor_for_depth, BisectionTree};
pub(crate) use rand::rngs::SmallRng;
pub(crate) use rand::SeedableRng;
pub(crate) use rayon::prelude::*;
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::path::Path;

mod adaptive_multiscale;
mod bfs;
mod bisection_ensemble;
mod core;
mod cvd;
mod dispatch;
mod flip;
mod forest_recom;
mod ilp;
mod merge_split;
mod mka;
mod multiscale;
mod nway_geo;
mod percentile;
mod short_burst;
mod simulated_annealing;
mod smc;

pub use adaptive_multiscale::*;
pub use bfs::*;
pub use bisection_ensemble::*;
pub use core::*;
pub use cvd::*;
pub use dispatch::*;
pub use flip::*;
pub use forest_recom::*;
pub use ilp::*;
pub use merge_split::*;
pub use mka::*;
pub use multiscale::*;
pub use nway_geo::*;
pub use percentile::*;
pub use short_burst::*;
pub use simulated_annealing::*;
pub use smc::*;

#[cfg(test)]
mod tests;
