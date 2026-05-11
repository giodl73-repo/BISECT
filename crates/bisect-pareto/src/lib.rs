//! `bisect-pareto` — Multi-objective Pareto redistricting via NSGA-II.
//!
//! Implements the Deb et al. (2002) NSGA-II genetic algorithm adapted for
//! redistricting plans as chromosomes. Produces a Pareto-optimal frontier of
//! plans over three objectives: edge cuts (compactness), partisan seat deviation,
//! and VRA minority-district deficit.
//!
//! Spec: docs/specs/2026-05-07-pareto-redistricting.md (Accepted, R2 avg 3.75/4)

pub mod algorithm;
pub mod audit;
pub mod crossover;
pub mod dominance;
pub mod mutation;
pub mod objectives;
pub mod output;
pub mod seeds;

pub use algorithm::{run_nsga2, ParetoConfig, ParetoError};
pub use audit::{write_selected_frontier_package, SelectedFrontierPackage};
pub use objectives::Objectives;
pub use output::{ParetoEntry, ParetoResult};
