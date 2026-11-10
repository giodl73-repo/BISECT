//! VeriPB forward checking library. Forward checking checks every rule in order from the start to end of the proof.
//!
//! This is done by implementing the [`ProofProcessor`](veripb_parser::pbp_parser::proof_processor::ProofProcessor) trait for the structure [`ForwardsChecker`](checker::ForwardsChecker), so that the [`ForwardsChecker`](checker::ForwardsChecker) can handle rules parsed by the parser crate [`veripb_parser`].
//!
//! To implement the checking functionality for each [`VeriPBRule`](veripb_rules::VeriPBRule), the this crate implements the [`Checkable`](rules::Checkable) trait for each [`VeriPBRule`](veripb_rules::VeriPBRule).

#![allow(clippy::restriction)]

pub mod checker;
pub mod config;
pub mod context;
pub mod database;
pub mod deletion_sequence;
pub mod error;
pub mod occurrence_list;
pub mod order;
pub mod order_context;
pub mod prelude;
pub mod proofgoal;
pub mod proofgoal_context;
pub mod rules;
pub mod subproof_context;
pub mod utils;
