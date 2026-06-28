//! bisect-runner — the recursive bisection redistricting engine.
//!
//! This crate was extracted from `bisect-cli` to make the ~13k lines of engine
//! logic reusable and unit-testable outside the `bisect` binary. The CLI's
//! `runner.rs` (multi-state orchestration) stays in `bisect-cli` and depends on
//! this crate.
//!
//! Module names (`args`, `runner`) deliberately mirror the original `bisect-cli`
//! module paths so the moved `bisection_runner` engine resolves its internal
//! `crate::args::*` / `crate::runner::*` references unchanged.
//!
//! - [`args`] — the three-layer compositor enums plus the ILP-audit CLI args.
//! - [`runner`] — engine support types shared with the bisection runner.
//! - [`adjacency_loader`] — adjacency `.adj.bin` / coarsening helpers.
//! - [`geosection_orientation`] — GeoSection minor-axis orientation math.
//! - [`ilp_audit`] — ILP solve-report verification and summary writing.
//! - [`bisection_runner`] — the level-parallel METIS bisection engine.

pub mod adjacency_loader;
pub mod args;
pub mod bisection_runner;
pub mod geosection_orientation;
pub mod ilp_audit;
pub mod runner;
