//! `BISECT-ilp` — Exact redistricting via Integer Linear Programming.
//!
//! Generates an ILP formulation for minimum-edge-cut redistricting.
//! Solves via external GLPK/HiGHS subprocess or writes MPS for manual solving.
//! Only practical for n <= 500 tracts. Spec: docs/specs/2026-05-07-ilp-redistricting.md

pub mod formulation;
pub mod result;
pub mod solver;

pub use formulation::{build_formulation, IlpFormulation};
pub use result::{IlpResult, SolverStatus};
pub use solver::{solve, IlpSolver};
