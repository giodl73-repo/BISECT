//! `BISECT-ilp` — Exact redistricting via Integer Linear Programming.
//!
//! Generates an ILP formulation for minimum-edge-cut redistricting.
//! Solves via external GLPK/HiGHS subprocess or writes MPS for manual solving.
//! Only practical for n <= 500 tracts. Spec: docs/specs/2026-05-07-ilp-redistricting.md

pub mod canonical;
pub mod certificates;
pub mod certified_single;
pub mod certified_split;
pub mod certified_tree;
pub mod connectivity_cuts;
pub mod formulation;
pub mod lp;
pub mod output;
pub mod proof_backend;
pub mod result;
pub mod separation;
pub mod solver;

pub use canonical::{
    solve_exact_canonical, solve_exact_canonical_artifacts, verify_exact_canonical_artifacts,
    verify_exact_canonical_certificate, ExactCanonicalArtifacts, ExactCanonicalCertificate,
    ExactCanonicalInstance, ExactCertificateError, ExactCertificateResult, ExactEdge,
    ExactObjective, ExactProofTranscript, ExhaustiveProof, PrimaryObjective,
    EXACT_CERTIFICATE_SCHEMA_VERSION, EXACT_ENUMERATION_LIMIT, EXACT_INSTANCE_SCHEMA_VERSION,
    EXACT_MODEL_ID, EXACT_PROOF_SCHEMA_VERSION,
};
pub use certificates::{
    branch_and_cut_certificate, BranchAndCutCertificate, BranchAndCutMode,
    BranchAndCutSeparationStatus,
};
pub use certified_single::{
    build_certified_single_district, verify_certified_single_district,
    CertifiedSingleDistrictCertificate, CertifiedSingleDistrictError,
    CertifiedSingleDistrictInstance, CERTIFIED_SINGLE_CERTIFICATE_SCHEMA_VERSION,
    CERTIFIED_SINGLE_INSTANCE_SCHEMA_VERSION,
};
pub use certified_split::{
    canonical_orientation_rule, canonical_seat_split, certified_split_children_connected,
    certified_split_unit_universe_hash, evaluate_certified_split_objective,
    solve_certified_split_bounded, verify_certified_split_bounded, CertifiedSplitArtifacts,
    CertifiedSplitCertificate, CertifiedSplitError, CertifiedSplitInstance,
    CertifiedSplitObjective, CertifiedSplitPrimaryObjective, CertifiedSplitProof,
    CertifiedSplitProofSummary, CertifiedSplitResult, SplitOrientationRule,
    CERTIFIED_SPLIT_CERTIFICATE_SCHEMA_VERSION, CERTIFIED_SPLIT_ENUMERATION_LIMIT,
    CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION, CERTIFIED_SPLIT_MODEL_ID,
    CERTIFIED_SPLIT_PROOF_SCHEMA_VERSION,
};
pub use certified_tree::{
    solve_certified_bisection_tree_bounded, verify_certified_bisection_tree_bounded,
    CertifiedBisectionLeaf, CertifiedBisectionTree, CertifiedTreeError,
    CERTIFIED_BISECTION_TREE_SCHEMA_VERSION,
};
pub use connectivity_cuts::{separate_connectivity_cuts, ConnectivityCut, DistrictComponents};
pub use formulation::{build_formulation, IlpFormulation};
pub use lp::{master_lp_string, LpExportError};
pub use output::{
    solve_report_json, solve_report_json_with_model_artifact, verify_model_artifact_for_report,
    IlpModelArtifact, IlpSolveAuditSummary, IlpSolveReport, ModelArtifactVerificationError,
    VerifiedModelArtifact, ILP_SOLVE_REPORT_SCHEMA_VERSION,
};
pub use proof_backend::{
    certified_split_discovery, compile_certified_split_boundary_relaxation,
    compile_certified_split_boundary_relaxation_outside_core,
    compile_certified_split_compact_boundary_branch, compile_certified_split_compact_proof_request,
    compile_certified_split_compact_proof_requests, compile_certified_split_cutset_boundary_branch,
    compile_certified_split_cutset_boundary_branch_with_fixes,
    compile_certified_split_proof_requests, compile_certified_split_reduced_boundary_relaxation,
    compile_certified_split_reduced_cutset_boundary_branch,
    compile_certified_split_regional_boundary_relaxation, CertifiedDecisionStage,
    CertifiedDecisionStatus, CertifiedOpbArtifact, CertifiedProofRequest, CertifiedSplitDiscovery,
    ProofBackendError, RegionalPopulationConstraint, RegionalPopulationRelation,
    CERTIFIED_DISCOVERY_SCHEMA_VERSION, CERTIFIED_PROOF_REQUEST_SCHEMA_VERSION,
};
pub use result::{IlpResult, SolverStatus};
pub use separation::{
    separate_round, SeparationRound, SeparationRoundStatus, SEPARATION_ROUND_SCHEMA_VERSION,
};
pub use solver::{solve, IlpSolver};
