// Engine modules extracted into the `bisect-runner` crate. Re-exported so the
// existing `crate::{adjacency_loader, bisection_runner, geosection_orientation,
// ilp_audit}` paths across the CLI continue to resolve unchanged.
pub use bisect_runner::{adjacency_loader, bisection_runner, geosection_orientation, ilp_audit};

pub mod aggregate;
pub mod algo_config;
pub mod analyze;
pub mod analyze_label;
pub mod args;
pub mod build_cmd;
pub mod civic;
pub mod compare;
pub mod demographics;
pub mod depo;
pub mod doctor;
pub mod edge_weights;
pub mod ensemble;
pub mod exact_cmd;
pub mod export_cmd;
pub mod fetch;
pub mod fletch;
pub mod geometry;
pub mod housing;
pub mod housing_evidence;
pub mod import_cmd;
pub mod import_label;
pub mod improve_cmd;
#[cfg(test)]
mod integration_pipeline_tests;
pub mod io_utils;
pub mod label;
pub mod label_cmd;
pub mod lodes;
pub mod lodes_evidence;
pub mod map_cmd;
pub mod migrate;
pub mod output;
pub mod paper_mode;
pub mod pareto_cmd;
pub mod partisan;
pub mod partisan_shares;
pub mod plan_cmd;
pub mod plan_context;
pub mod policy;
pub mod provenance;
pub mod registry;
pub mod report_cmd;
pub mod research;
pub mod run_registry;
pub mod runner;
pub mod sa_evidence;
pub mod status;
pub mod suite;
pub mod sweep;
pub mod validate;
pub mod verify;
pub mod vertex_weights;
