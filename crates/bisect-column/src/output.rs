use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::master::{MasterFormulationSize, MasterSolution};

pub const BRANCH_PRICE_REPORT_SCHEMA_VERSION: &str = "bisect-branch-price-report-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BranchPriceStatus {
    FormulationOnly,
    ExactFixtureOptimal,
    Infeasible,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchPriceReport {
    pub schema_version: String,
    pub method: String,
    pub status: BranchPriceStatus,
    pub pricing_rounds: usize,
    pub generated_columns: usize,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
    pub gap: Option<f64>,
    pub formulation_size: MasterFormulationSize,
    pub solution: Option<MasterSolution>,
    pub parameter_hash: String,
}

impl BranchPriceReport {
    pub fn new(
        status: BranchPriceStatus,
        pricing_rounds: usize,
        generated_columns: usize,
        formulation_size: MasterFormulationSize,
        solution: Option<MasterSolution>,
    ) -> Self {
        let objective = solution.as_ref().map(|solution| solution.objective as f64);
        let mut report = Self {
            schema_version: BRANCH_PRICE_REPORT_SCHEMA_VERSION.to_string(),
            method: "branch-and-price".to_string(),
            status,
            pricing_rounds,
            generated_columns,
            lower_bound: objective,
            upper_bound: objective,
            gap: objective.map(|_| 0.0),
            formulation_size,
            solution,
            parameter_hash: String::new(),
        };
        if status == BranchPriceStatus::FormulationOnly {
            report.lower_bound = None;
            report.upper_bound = None;
            report.gap = None;
        }
        report.parameter_hash = report.compute_parameter_hash();
        report
    }

    fn compute_parameter_hash(&self) -> String {
        let payload = serde_json::json!({
            "schema_version": self.schema_version,
            "method": self.method,
            "status": self.status,
            "pricing_rounds": self.pricing_rounds,
            "generated_columns": self.generated_columns,
            "lower_bound": self.lower_bound,
            "upper_bound": self.upper_bound,
            "gap": self.gap,
            "formulation_size": self.formulation_size,
            "solution": self.solution,
        });
        let bytes = serde_json::to_vec(&payload).expect("branch-price report payload serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    pub fn algorithm_lineage(
        &self,
        producer_version: impl Into<String>,
        parent_plan_hashes: Vec<String>,
    ) -> Result<rplan_audit::AlgorithmLineage, rplan_audit::AuditError> {
        rplan_audit::AlgorithmLineage::new(
            "bisect-column",
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
            "pricing_rounds": self.pricing_rounds,
            "generated_columns": self.generated_columns,
            "lower_bound": self.lower_bound,
            "upper_bound": self.upper_bound,
            "gap": self.gap,
            "parameter_hash": self.parameter_hash,
        })
    }
}
