use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::output::{ParetoEntry, ParetoResult};

pub const SELECTED_FRONTIER_PACKAGE_SCHEMA_VERSION: &str =
    "bisect-pareto-selected-frontier-package-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedFrontierPackage {
    pub schema_version: String,
    pub selected_frontier_index: usize,
    pub rplan_path: String,
    pub rctx_path: String,
    pub audit_certificate_path: String,
    pub audit_certificate_sha256: String,
    pub audit_certificate_content_hash: String,
    pub manifest_path: String,
}

#[derive(Debug, Error)]
pub enum ParetoAuditError {
    #[error("selected frontier index {selected} out of bounds for frontier size {frontier_size}")]
    IndexOutOfBounds {
        selected: usize,
        frontier_size: usize,
    },
    #[error("frontier plan district id {district} is outside 1..={k}")]
    InvalidDistrict { district: u32, k: usize },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("rplan io error: {0}")]
    RplanIo(#[from] rplan_io::RplanIoError),
    #[error("audit error: {0}")]
    Audit(#[from] rplan_audit::AuditError),
    #[error("selected Pareto frontier plan failed audit")]
    PlanFailedAudit,
}

#[allow(clippy::too_many_arguments)]
pub fn write_selected_frontier_package(
    result: &ParetoResult,
    selected_frontier_index: usize,
    context: &rplan_core::RplanContext,
    k: usize,
    out_dir: &Path,
    label: &str,
    tolerance_percent: f64,
    generated_at: &str,
) -> Result<SelectedFrontierPackage, ParetoAuditError> {
    let entry =
        result
            .frontier
            .get(selected_frontier_index)
            .ok_or(ParetoAuditError::IndexOutOfBounds {
                selected: selected_frontier_index,
                frontier_size: result.frontier.len(),
            })?;
    let assignment = zero_based_assignment(entry, k)?;
    std::fs::create_dir_all(out_dir)?;

    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "US".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment,
            k,
            display_labels: (0..k).map(|district| district.to_string()).collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: label.to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.to_string(),
            description: Some("U.19 selected Pareto frontier plan".to_string()),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect pareto")),
                ("crate".to_string(), serde_json::json!("bisect-pareto")),
                ("method".to_string(), serde_json::json!("nsga2")),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = pareto_legal_profile(
        &jurisdiction,
        &document.metadata.chamber,
        document.plan.units.year.unwrap_or(2020),
        tolerance_percent,
    );
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-pareto",
        env!("CARGO_PKG_VERSION"),
        "nsga2-selected-frontier",
        Vec::new(),
        selected_frontier_lineage_extra(result, entry, selected_frontier_index),
    )?;
    let certificate = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: None,
        },
        &[
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        generated_at,
        Some(lineage),
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(ParetoAuditError::PlanFailedAudit);
    }

    let rplan_path = out_dir.join("selected-frontier.rplan");
    let rctx_path = out_dir.join("selected-frontier.rctx");
    let certificate_path = out_dir.join("audit-certificate.json");
    let manifest_path = out_dir.join("manifest.json");
    std::fs::write(&rplan_path, rplan_io::write_rplan_string(&document)?)?;
    std::fs::write(&rctx_path, rplan_io::write_rctx_string(context)?)?;
    std::fs::write(
        &certificate_path,
        serde_json::to_string_pretty(&certificate)?,
    )?;
    let certificate_sha256 = sha256_file(&certificate_path)?;
    let package = SelectedFrontierPackage {
        schema_version: SELECTED_FRONTIER_PACKAGE_SCHEMA_VERSION.to_string(),
        selected_frontier_index,
        rplan_path: "selected-frontier.rplan".to_string(),
        rctx_path: "selected-frontier.rctx".to_string(),
        audit_certificate_path: "audit-certificate.json".to_string(),
        audit_certificate_sha256: certificate_sha256,
        audit_certificate_content_hash: certificate.content_hash.clone(),
        manifest_path: "manifest.json".to_string(),
    };
    let manifest = serde_json::json!({
        "schema_version": package.schema_version,
        "label": label,
        "selected_frontier_index": selected_frontier_index,
        "frontier_size": result.frontier.len(),
        "rplan_path": package.rplan_path,
        "rctx_path": package.rctx_path,
        "audit_certificate_path": package.audit_certificate_path,
        "audit_certificate_sha256": package.audit_certificate_sha256,
        "audit_certificate_content_hash": package.audit_certificate_content_hash,
        "audit_result": rplan_audit_result_label(&certificate.result),
        "legal_profile_id": profile.profile_id,
        "context_hash": certificate.context_hash,
        "objectives": {
            "ec": entry.objectives.ec,
            "d_seats": entry.objectives.d_seats,
            "vra_deficit": entry.objectives.vra_deficit,
        },
        "validity_status": entry.validity_status,
    });
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(package)
}

fn zero_based_assignment(entry: &ParetoEntry, k: usize) -> Result<Vec<u32>, ParetoAuditError> {
    entry
        .plan
        .iter()
        .map(|&district| {
            if district == 0 || district as usize > k {
                Err(ParetoAuditError::InvalidDistrict { district, k })
            } else {
                Ok(district - 1)
            }
        })
        .collect()
}

fn selected_frontier_lineage_extra(
    result: &ParetoResult,
    entry: &ParetoEntry,
    selected_frontier_index: usize,
) -> serde_json::Value {
    serde_json::json!({
        "lineage_schema_version": SELECTED_FRONTIER_PACKAGE_SCHEMA_VERSION,
        "method": "nsga2",
        "selected_frontier_index": selected_frontier_index,
        "frontier_size": result.frontier.len(),
        "n_population": result.config.n_population,
        "n_generations": result.config.n_generations,
        "base_seed": result.config.base_seed,
        "balance_tolerance": result.config.balance_tolerance,
        "generation_found": entry.generation_found,
        "validity_status": entry.validity_status,
        "objectives": {
            "ec": entry.objectives.ec,
            "d_seats": entry.objectives.d_seats,
            "vra_deficit": entry.objectives.vra_deficit,
        },
    })
}

fn pareto_legal_profile(
    jurisdiction: &str,
    chamber: &str,
    year: u16,
    tolerance_percent: f64,
) -> rplan_audit::LegalProfile {
    let chamber = match chamber {
        "congressional" => rplan_audit::Chamber::Congressional,
        other => rplan_audit::Chamber::Custom(other.to_string()),
    };
    rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "BISECT_PARETO_SELECTED_FRONTIER_V1".to_string(),
        jurisdiction: jurisdiction.to_string(),
        chamber,
        year,
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: tolerance_percent,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::CountOnly,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    }
}

fn rplan_audit_result_label(result: &rplan_audit::AuditResult) -> &'static str {
    match result {
        rplan_audit::AuditResult::Pass => "pass",
        rplan_audit::AuditResult::Fail => "fail",
        rplan_audit::AuditResult::PassWithWarnings => "pass-with-warnings",
    }
}

fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(PathBuf::from(path))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
