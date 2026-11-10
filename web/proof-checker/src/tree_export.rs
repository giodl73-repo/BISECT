//! Native plan delivery requires fresh tree proofs and matching original RCTX.
use std::collections::{BTreeMap, BTreeSet};
use serde::Deserialize;
use serde_json::{json, Value};
use bisect_ilp::CertifiedSplitInstance;
use rplan_audit::{AuditConstraint, AuditResult};
use crate::{hash, native_tree::{AdoptTreeRequest, adopt_tree}};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRequest {
    pub operation: String,
    pub tree_request: AdoptTreeRequest,
    pub context_json: String,
    pub profile_json: String,
    pub constraints: Vec<AuditConstraint>,
    pub label: String,
    pub created_at: String,
}
pub fn export(request: ExportRequest) -> Result<Value, String> { inner(request).map_err(|e| e.to_string()) }
fn inner(request: ExportRequest) -> Result<Value, Box<dyn std::error::Error>> {
    if request.operation != "export-native-external-tree-plan" || request.context_json.len() > 8 * 1024 * 1024 || request.profile_json.len() > 3 * 1024 * 1024 { return Err("Unsupported plan export request or oversized context/profile".into()); }
    for text in [&request.label, &request.created_at] { if text.is_empty() || text.len() > 200 || text.chars().any(char::is_control) { return Err("Invalid plan label or creation timestamp".into()); } }
    if request.constraints.len() > 6 || request.constraints.iter().collect::<BTreeSet<_>>().len() != request.constraints.len() || ![AuditConstraint::PlanShape, AuditConstraint::Population, AuditConstraint::Contiguity].iter().all(|item| request.constraints.contains(item)) { return Err("Plan delivery requires distinct shape, population and contiguity audit checks".into()); }
    let context = rplan_io::read_rctx_str(&request.context_json)?;
    let profile: rplan_audit::LegalProfile = serde_json::from_str(&request.profile_json)?;
    if !profile.contiguity_required || context.units.year.is_some_and(|year| year != profile.year) || context.units.state.as_ref().is_some_and(|state| state != &profile.jurisdiction && profile.jurisdiction != "US") { return Err("Audit profile must match context jurisdiction/year and require contiguity".into()); }
    let root: CertifiedSplitInstance = serde_json::from_str(&request.tree_request.root_instance_json)?;
    if context.units.unit_ids != root.unit_ids || context.populations.as_ref() != Some(&root.populations) { return Err("Original context unit/population identity differs from the proved root".into()); }
    let graph = context.graph.as_ref().ok_or("Original context graph is required")?;
    let mut edges = BTreeMap::new();
    for (left, neighbors) in graph.adjacency.iter().enumerate() { for edge in neighbors {
        let right = edge.to as usize; if left >= right { continue; }
        let weight = match edge.weight { None => 1, Some(value) if value.is_finite() && value > 0.0 && (value - value.round()).abs() < 1e-9 && value < u64::MAX as f64 => value as u64, _ => return Err("Context proof graph requires positive integer edge weights".into()) };
        edges.insert((left, right), weight);
    } }
    if edges != root.edges.iter().map(|edge| ((edge.left, edge.right), edge.weight)).collect::<BTreeMap<_, _>>() { return Err("Original context weighted graph differs from the proved root".into()); }
    let tree = adopt_tree(request.tree_request)?;
    let assignment = context.units.unit_ids.iter().map(|id| tree.assignments.get(id).copied().ok_or("Missing proved tree assignment").and_then(|index| u32::try_from(index).map_err(|_| "District index overflow"))).collect::<Result<Vec<_>, _>>()?;
    let chamber = match &profile.chamber { rplan_audit::Chamber::Congressional => "congressional".into(), rplan_audit::Chamber::StateHouse => "state-house".into(), rplan_audit::Chamber::StateSenate => "state-senate".into(), rplan_audit::Chamber::Local => "local".into(), rplan_audit::Chamber::Custom(name) => name.clone() };
    if chamber.is_empty() || chamber.len() > 200 || chamber.chars().any(char::is_control) { return Err("Invalid profile chamber label".into()); }
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.into(),
        plan: rplan_core::DistrictPlan { schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.into(), units: context.units.clone(), assignment, k: tree.k, display_labels: (1..=tree.k).map(|index| index.to_string()).collect(), allow_empty_districts: false },
        metadata: rplan_io::RplanMetadataV02 { label: request.label, jurisdiction: context.units.state.clone().unwrap_or_else(|| profile.jurisdiction.clone()), chamber, created_at: request.created_at.clone(), description: Some("Externally proved native recursive bisection tree; chosen-profile audit is separate from split optimality".into()) },
        provenance: rplan_io::RplanProvenance { producer: BTreeMap::from([("name".into(), json!("bisect-proof-checker")), ("method".into(), json!("external-certified-recursive")), ("certified_tree_id".into(), json!(tree.tree_id))]), source_hashes: context.source_hashes.entries.clone(), conversion_lineage: Vec::new() }, geometry: None, extensions: BTreeMap::new(),
    };
    document.plan.validate()?;
    let lineage = rplan_audit::AlgorithmLineage::new("bisect-proof-checker", env!("CARGO_PKG_VERSION"), "external-certified-recursive", Vec::new(), json!({"external_tree_id": tree.tree_id, "split_count": tree.nodes.len(), "leaf_count": tree.leaves.len()}))?;
    let audit = rplan_audit::audit_plan_with_lineage(&document.plan, Some(&context), &profile, rplan_audit::RuntimeProvenance { binary_name: "bisect-proof-checker".into(), binary_version: env!("CARGO_PKG_VERSION").into(), git_commit: None, build_profile: None, solver: None }, &request.constraints, &request.created_at, Some(lineage))?;
    if matches!(audit.result, AuditResult::Fail) { return Err("Externally proved tree plan failed the selected native audit".into()); }
    let mut files = BTreeMap::from([("external-tree.json".to_string(), serde_json::to_string_pretty(&tree)?), ("exact.rplan".into(), rplan_io::write_rplan_string(&document)?), ("exact.rctx".into(), rplan_io::write_rctx_string(&context)?), ("audit-certificate.json".into(), serde_json::to_string_pretty(&audit)?) ]);
    let hashes = files.iter().map(|(name, text)| (name.clone(), hash(text.as_bytes()))).collect::<BTreeMap<_, _>>();
    let manifest = json!({"schema_version":"native-external-tree-plan-package-v1", "tree_id":tree.tree_id, "context_hash":context.context_hash, "context_source_sha256":hash(request.context_json.as_bytes()), "profile_source_sha256":hash(request.profile_json.as_bytes()), "districts":tree.k, "constraints":request.constraints, "files":hashes, "scope":"Fresh external tree proof verification, original context graph/population binding and selected native audit. No geometry, global map optimum, source authenticity or legal authority is asserted."});
    files.insert("manifest.json".into(), serde_json::to_string_pretty(&manifest)?);
    let hashes = files.iter().map(|(name, text)| (name.clone(), hash(text.as_bytes()))).collect::<BTreeMap<_, _>>();
    Ok(json!({"schema_version":"bisect-external-tree-plan-output-v1", "status":"completed", "tree_id":tree.tree_id, "districts":tree.k, "document_json":files["exact.rplan"], "context_json":files["exact.rctx"], "audit_certificate_json":files["audit-certificate.json"], "files":files, "file_sha256":hashes, "context_source_sha256":hash(request.context_json.as_bytes()), "profile_source_sha256":hash(request.profile_json.as_bytes()), "scope":manifest["scope"]}))
}
