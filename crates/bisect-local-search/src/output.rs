use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const LOCAL_SEARCH_SUMMARY_SCHEMA_VERSION: &str = "bisect-local-search-summary-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalSearchSummary {
    pub schema_version: String,
    pub method: String,
    pub status: String,
    pub moves_evaluated: usize,
    pub moves_accepted: usize,
    pub initial_edge_cut: usize,
    pub final_edge_cut: usize,
    pub initial_population_deviation: f64,
    pub final_population_deviation: f64,
    pub tolerance: f64,
    pub parameter_hash: String,
}

impl LocalSearchSummary {
    pub fn new(
        method: &str,
        status: &str,
        moves_evaluated: usize,
        moves_accepted: usize,
        initial_edge_cut: usize,
        final_edge_cut: usize,
        initial_population_deviation: f64,
        final_population_deviation: f64,
        tolerance: f64,
    ) -> Self {
        let mut summary = Self {
            schema_version: LOCAL_SEARCH_SUMMARY_SCHEMA_VERSION.to_string(),
            method: method.to_string(),
            status: status.to_string(),
            moves_evaluated,
            moves_accepted,
            initial_edge_cut,
            final_edge_cut,
            initial_population_deviation,
            final_population_deviation,
            tolerance,
            parameter_hash: String::new(),
        };
        summary.parameter_hash = summary.compute_parameter_hash();
        summary
    }

    fn compute_parameter_hash(&self) -> String {
        let payload = serde_json::json!({
            "schema_version": self.schema_version,
            "method": self.method,
            "status": self.status,
            "moves_evaluated": self.moves_evaluated,
            "moves_accepted": self.moves_accepted,
            "initial_edge_cut": self.initial_edge_cut,
            "final_edge_cut": self.final_edge_cut,
            "initial_population_deviation": self.initial_population_deviation,
            "final_population_deviation": self.final_population_deviation,
            "tolerance": self.tolerance,
        });
        let bytes = serde_json::to_vec(&payload).expect("local-search summary serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    pub fn algorithm_lineage(
        &self,
        producer_version: impl Into<String>,
        parent_plan_hashes: Vec<String>,
    ) -> Result<rplan_audit::AlgorithmLineage, rplan_audit::AuditError> {
        rplan_audit::AlgorithmLineage::new(
            "bisect-local-search",
            producer_version,
            self.method.clone(),
            parent_plan_hashes,
            self.algorithm_lineage_extra(),
        )
    }

    pub fn algorithm_lineage_extra(&self) -> serde_json::Value {
        serde_json::json!({
            "lineage_schema_version": self.schema_version,
            "method": self.method,
            "status": self.status,
            "moves_evaluated": self.moves_evaluated,
            "moves_accepted": self.moves_accepted,
            "initial_edge_cut": self.initial_edge_cut,
            "final_edge_cut": self.final_edge_cut,
            "initial_population_deviation": self.initial_population_deviation,
            "final_population_deviation": self.final_population_deviation,
            "tolerance": self.tolerance,
            "parameter_hash": self.parameter_hash,
        })
    }
}
