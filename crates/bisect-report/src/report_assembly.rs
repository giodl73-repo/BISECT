//! Shared ten-section native report assembly from supplied values.
use crate::manifest_schema::PlanManifestWithCounts;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const REQUIRED_ANALYSIS_FILES: &[&str] =
    &["summary.json", "contiguity.json", "compactness.json"];

/// Optional analysis files (not required — report section will note "unavailable").
pub const OPTIONAL_ANALYSIS_FILES: &[&str] = &[
    "partisan.json",
    "splits.json",
    "vra_analysis.json",
    "comparison.json",
    "nesting.json",
];

/// Fully assembled 10-section redistricting report.
#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub report_version: String,
    pub label: String,
    pub state: String,
    pub year: String,
    pub generated_at: String,
    pub sections: ReportSections,
    /// Whether any external analyzers were used (triggers disclaimer in HTML)
    pub has_external_analyzers: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSections {
    /// Section 1: Executive summary
    pub executive_summary: Value,
    /// Section 2: Population equality
    pub population_equality: Value,
    /// Section 3: Geographic constraints (contiguity + splits)
    pub geographic_constraints: Value,
    /// Section 4: Partisan fairness
    pub partisan_fairness: Value,
    /// Section 5: VRA compliance
    pub vra_compliance: Value,
    /// Section 6: Compactness
    pub compactness: Value,
    /// Section 7: Comparison vs enacted plan
    pub comparison: Value,
    /// Section 8: Nesting compliance
    pub nesting: Value,
    /// Section 9: Audit trail
    pub audit: Value,
    /// Section 10: Maps
    pub maps: Value,
}

fn with_status(mut value: Value) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.entry("status")
            .or_insert_with(|| serde_json::json!("ok"));
    }
    value
}
pub fn assemble_report_from_values<C: serde::Serialize>(
    m: &PlanManifestWithCounts<C>,
    analysis: &std::collections::BTreeMap<String, Value>,
    audit: Value,
    maps: Value,
    generated_at: String,
) -> anyhow::Result<Report> {
    // R3 amendment: check for required files first
    let missing: Vec<_> = REQUIRED_ANALYSIS_FILES
        .iter()
        .filter(|name| !analysis.contains_key(**name))
        .copied()
        .collect();
    if !missing.is_empty() {
        anyhow::bail!(
            "Cannot assemble report: required analysis files missing: {}. \
             Run 'bisect analyze' first.",
            missing.join(", ")
        );
    }

    // Section 1: Executive summary
    let exec_summary = serde_json::json!({
        "label": m.label,
        "state": m.state_code,
        "year": m.year,
        "chamber": m.chamber,
        "num_districts": m.num_districts,
        "binary_version": m.binary_version,
        "created_at": m.created_at,
        "population_balance_valid": m.population_balance_valid,
        "balance_tolerance_pct": m.balance_tolerance_pct,
    });

    // Section 2: Population equality (written by SummaryAnalyzer as summary.json)
    let population_json = analysis
        .get("summary.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 3: Geographic constraints
    let contiguity_json = analysis
        .get("contiguity.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));
    let splits_json = analysis
        .get("splits.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));
    let geographic = serde_json::json!({
        "contiguity": contiguity_json,
        "splits": splits_json,
    });

    // Section 4: Partisan fairness (optional)
    let partisan = analysis
        .get("partisan.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 5: VRA compliance (optional)
    let vra = analysis
        .get("vra_analysis.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 6: Compactness
    let compactness = analysis
        .get("compactness.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 7: Comparison (optional)
    let comparison = analysis
        .get("comparison.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Section 8: Nesting (optional)
    let nesting = analysis
        .get("nesting.json")
        .cloned()
        .map(with_status)
        .unwrap_or_else(|| serde_json::json!({"status": "unavailable"}));

    // Audit evidence and embedded map bytes are supplied by the host adapter.
    Ok(Report {
        report_version: "1.0".into(),
        label: m.label.clone(),
        state: m.state_code.clone(),
        year: m.year.clone(),
        generated_at,
        has_external_analyzers: false,
        sections: ReportSections {
            executive_summary: exec_summary,
            population_equality: population_json,
            geographic_constraints: geographic,
            partisan_fairness: partisan,
            vra_compliance: vra,
            compactness,
            comparison,
            nesting,
            audit,
            maps,
        },
    })
}

use crate::verification::{
    generate_verification_command_from_manifest, generate_verification_script,
};
pub fn build_audit_from_values<C: std::fmt::Display>(
    m: &PlanManifestWithCounts<C>,
    source_hashes: Value,
    lineage: Value,
) -> anyhow::Result<Value> {
    let verification_command = generate_verification_command_from_manifest(m);
    let verification_script = generate_verification_script(m);
    Ok(serde_json::json!({
        "verification_command": verification_command,
        "verification_script": verification_script,
        "binary_version": m.binary_version,
        "binary_download_url": m.binary_download_url,
        "binary_sha256": m.binary_sha256,
        "adjacency_file": m.adjacency_file,
        "adjacency_sha256": if m.adjacency_sha256.is_empty() { "(not computed — run: sha256sum adjacency_file)".to_string() } else { m.adjacency_sha256.clone() },
        "tiger_source_url": m.tiger_source_url,
        "tiger_sha256": m.tiger_sha256.clone().unwrap_or_else(|| format!("(not recorded — download from {} and compute manually)", m.tiger_source_url)),
        "created_at": m.created_at,
        "seed": m.recorded_seed_json(),
        "rplan_path": m.rplan_path,
        "rctx_path": m.rctx_path,
        "audit_certificate_path": m.audit_certificate_path,
        "audit_certificate_sha256": m.audit_certificate_sha256,
        "audit_certificate_content_hash": m.audit_certificate_content_hash,
        "audit_result": m.audit_result,
        "legal_profile_id": m.legal_profile_id,
        "context_hash": m.context_hash,
        "rplan_source_hashes": source_hashes,
        "algorithm_lineage": lineage,
    }))
}
