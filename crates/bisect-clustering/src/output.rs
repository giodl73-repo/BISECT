use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::repair::RepairStatus;

pub const CLUSTER_SUMMARY_SCHEMA_VERSION: &str = "bisect-clustering-summary-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterSummary {
    pub schema_version: String,
    pub method: String,
    pub seed_method: String,
    pub repair_method: String,
    pub capacity_status: String,
    pub repair_status: RepairStatus,
    pub population_deviation: f64,
    pub edge_cut: usize,
    pub parameter_hash: String,
}

impl ClusterSummary {
    pub fn new(
        method: &str,
        seed_method: &str,
        repair_method: &str,
        capacity_status: &str,
        repair_status: RepairStatus,
        population_deviation: f64,
        edge_cut: usize,
    ) -> Self {
        let mut summary = Self {
            schema_version: CLUSTER_SUMMARY_SCHEMA_VERSION.to_string(),
            method: method.to_string(),
            seed_method: seed_method.to_string(),
            repair_method: repair_method.to_string(),
            capacity_status: capacity_status.to_string(),
            repair_status,
            population_deviation,
            edge_cut,
            parameter_hash: String::new(),
        };
        summary.parameter_hash = summary.compute_parameter_hash();
        summary
    }

    fn compute_parameter_hash(&self) -> String {
        let payload = serde_json::json!({
            "schema_version": self.schema_version,
            "method": self.method,
            "seed_method": self.seed_method,
            "repair_method": self.repair_method,
            "capacity_status": self.capacity_status,
            "repair_status": self.repair_status,
            "population_deviation": self.population_deviation,
            "edge_cut": self.edge_cut,
        });
        let bytes = serde_json::to_vec(&payload).expect("cluster summary payload serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    pub fn algorithm_lineage(
        &self,
        producer_version: impl Into<String>,
        parent_plan_hashes: Vec<String>,
    ) -> Result<rplan_audit::AlgorithmLineage, rplan_audit::AuditError> {
        rplan_audit::AlgorithmLineage::new(
            "bisect-clustering",
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
            "seed_method": self.seed_method,
            "repair_method": self.repair_method,
            "capacity_status": self.capacity_status,
            "repair_status": self.repair_status,
            "population_deviation": self.population_deviation,
            "edge_cut": self.edge_cut,
            "parameter_hash": self.parameter_hash,
        })
    }
}
