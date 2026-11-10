//! Crate for handling pseudo-Boolean unit propagation in the VeriPB proof checker. The main way to use the this library for pseudo-Boolean propagation is through interaction with the [`PropagationEngine`](`propagation_engine::PropagationEngine`).

#![allow(clippy::restriction)]

pub mod cardinality_watcher;
pub mod clause_watcher;
pub mod error;
pub mod general_pb_watcher;
pub mod propagation_engine;
pub mod propagation_set;
pub mod propagator;
pub mod trail;
pub mod watcher;
