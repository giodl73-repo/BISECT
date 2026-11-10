//! Deterministic recursive-bisection engine.
//!
//! Split from a single 10k-line file into per-algorithm submodules. Shared
//! imports and cross-section helpers are re-exported here so each submodule
//! can `use super::*;` and reach its siblings.

pub(crate) use bisect_core::BisectionTree;
pub(crate) use rand::rngs::SmallRng;
pub(crate) use rand::SeedableRng;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use rayon::prelude::*;
#[cfg(target_arch = "wasm32")]
pub(crate) use crate::serial::*;
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::path::Path;

// Native builds retain their compile-time C choice. A browser build containing
// both kernels selects C only inside an explicitly requested execution scope.
fn use_c_metis()->bool {
    #[cfg(feature="c-ffi-engine")]
    {true}
    #[cfg(all(not(feature="c-ffi-engine"),feature="wasm-c-ffi"))]
    {metis_sidecar::is_enabled()}
    #[cfg(not(any(feature="c-ffi-engine",feature="wasm-c-ffi")))]
    {false}
}
#[cfg(all(feature="wasm-c-ffi",not(feature="c-ffi-engine")))]
pub(crate) use metis_sidecar as metis;

mod round_monitor;
pub use round_monitor::{RoundMonitor,RoundMonitorEvent};
mod round_capture;
pub use round_capture::{RoundCapture,RoundObserver};
pub(crate) use round_capture::{round_output_enabled,observe_round};
mod adaptive_multiscale;
mod bfs;
mod bisection_ensemble;
mod core;
mod convergence;
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
pub use convergence::*;
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
