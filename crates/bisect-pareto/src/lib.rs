//! `bisect-pareto` — Multi-objective Pareto redistricting via NSGA-II.
//!
//! Implements the Deb et al. (2002) NSGA-II genetic algorithm adapted for
//! redistricting plans as chromosomes. Produces a Pareto-optimal frontier of
//! plans over three objectives: edge cuts (compactness), partisan seat deviation,
//! and VRA minority-district deficit.
//!
//! Spec: docs/specs/2026-05-07-pareto-redistricting.md (Accepted, R2 avg 3.75/4)

pub mod objectives;
pub mod dominance;
pub mod crossover;
pub mod mutation;
pub mod seeds;
pub mod algorithm;
pub mod output;

pub use algorithm::{run_nsga2, ParetoConfig, ParetoError};
pub use output::{ParetoEntry, ParetoResult};
pub use objectives::Objectives;
