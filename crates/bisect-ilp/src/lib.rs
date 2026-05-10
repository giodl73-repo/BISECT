//! `BISECT-ilp` — Exact redistricting via Integer Linear Programming.
//!
//! Generates an ILP formulation for minimum-edge-cut redistricting.
//! Solves via external GLPK/HiGHS subprocess or writes MPS for manual solving.
//! Only practical for n <= 500 tracts. Spec: docs/specs/2026-05-07-ilp-redistricting.md

pub mod certificates;
pub mod connectivity_cuts;
pub mod formulation;
pub mod lp;
pub mod output;
pub mod result;
pub mod separation;
pub mod solver;

pub use certificates::{
    branch_and_cut_certificate, BranchAndCutCertificate, BranchAndCutMode,
    BranchAndCutSeparationStatus,
};
pub use connectivity_cuts::{separate_connectivity_cuts, ConnectivityCut, DistrictComponents};
pub use formulation::{build_formulation, IlpFormulation};
pub use lp::{master_lp_string, LpExportError};
pub use output::{
    solve_report_json, solve_report_json_with_model_artifact, verify_model_artifact_for_report,
    IlpModelArtifact, IlpSolveAuditSummary, IlpSolveReport, ModelArtifactVerificationError,
    VerifiedModelArtifact, ILP_SOLVE_REPORT_SCHEMA_VERSION,
};
pub use result::{IlpResult, SolverStatus};
pub use separation::{
    separate_round, SeparationRound, SeparationRoundStatus, SEPARATION_ROUND_SCHEMA_VERSION,
};
pub use solver::{solve, IlpSolver};
