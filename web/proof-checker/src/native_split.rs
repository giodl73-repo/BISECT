//! Native external split certificates: compile every query and check every
//! proof through the concrete VeriPB kernel. No submitted receipt is trusted.
use std::collections::BTreeMap;
use bisect_ilp::{CertifiedSplitInstance, CertifiedSplitDiscovery, CertifiedDecisionStage, CertifiedOpbArtifact, CertifiedSplitResult,
    compile_certified_split_compact_proof_request, compile_certified_split_compact_boundary_branch};
use serde::{Deserialize, Serialize};
use crate::{Request, Report, check, hash};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptRequest {
    pub operation: String,
    pub instance_json: String,
    pub discovery_json: String,
    pub proofs: BTreeMap<String, String>,
    #[serde(default)]
    pub expected_certificate: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct CertifiedExternalSplitCertificate {
    pub schema_version: &'static str,
    pub certificate_id: String,
    pub instance_hash: String,
    pub discovery_id: String,
    pub model_id: String,
    pub node_path: String,
    pub parent_certificate_id: Option<String>,
    pub instance_source_sha256: String,
    pub discovery_source_sha256: String,
    pub compiler_source_sha256: String,
    #[serde(serialize_with = "serialize_exact_result")]
    pub result: CertifiedSplitResult,
    pub proof_kind: &'static str,
    pub decisions: BTreeMap<String, CheckedDecision>,
    pub scope: &'static str,
}
#[derive(Serialize)]
pub struct CheckedDecision {
    pub request_id: String,
    pub opb_sha256: String,
    pub request_json: String,
    pub proof_sha256: String,
    pub checker: Report,
}
fn serialize_exact_result<S: serde::Serializer>(result: &CertifiedSplitResult, serializer: S) -> Result<S::Ok, S::Error> {
    match result {
        CertifiedSplitResult::Optimal { assignment, objective } => serde_json::json!({
            "result": "optimal", "assignment": assignment,
            "objective": {"primary": {
                "max_population_deviation_scaled": objective.primary.max_population_deviation_scaled.to_string(),
                "total_population_deviation_scaled": objective.primary.total_population_deviation_scaled.to_string(),
                "weighted_boundary_cut": objective.primary.weighted_boundary_cut.to_string()
            }, "canonical_assignment": objective.canonical_assignment}
        }).serialize(serializer),
        CertifiedSplitResult::Infeasible => serde_json::json!({"result": "infeasible"}).serialize(serializer),
    }
}

pub fn adopt(request: AdoptRequest) -> Result<CertifiedExternalSplitCertificate, String> {
    if request.operation != "adopt-native-external-split" { return Err("Unsupported native certificate operation".into()); }
    if request.instance_json.len() > crate::MAX_SOURCE || request.discovery_json.len() > crate::MAX_SOURCE { return Err("Native certificate sources exceed 3 MiB each".into()); }
    if request.proofs.len() > 5 || request.proofs.values().any(|text| text.len() > crate::MAX_PROOF) || request.proofs.values().map(String::len).sum::<usize>() > 128 * 1024 * 1024 { return Err("Native certificate proof inputs exceed their limits".into()); }
    let instance: CertifiedSplitInstance = serde_json::from_str(&request.instance_json).map_err(|e| e.to_string())?;
    let discovery: CertifiedSplitDiscovery = serde_json::from_str(&request.discovery_json).map_err(|e| e.to_string())?;
    instance.validate().map_err(|e| e.to_string())?;
    if instance.unit_ids.len() > 100000 || instance.edges.len() > 200000 { return Err("Native certificate graph exceeds its limit".into()); }
    let bits = usize::BITS as usize - (instance.unit_ids.len() - 1).leading_zeros() as usize;
    let estimate = 5u128 * (instance.unit_ids.len() as u128 * 4096 + instance.edges.len() as u128 * (1024 + 256 * bits as u128));
    if estimate > 128 * 1024 * 1024 { return Err("Native compact compiler estimated working set exceeds 128 MiB".into()); }
    let mut models: BTreeMap<String, CertifiedOpbArtifact> = BTreeMap::new();
    for (stem, stage) in [("01-population", CertifiedDecisionStage::PopulationLowerBound), ("02-boundary", CertifiedDecisionStage::BoundaryLowerBound), ("03-canonical", CertifiedDecisionStage::CanonicalTieBreak)] {
        models.insert(stem.into(), compile_certified_split_compact_proof_request(&instance, &discovery, stage).map_err(|e| e.to_string())?);
    }
    if discovery.objective.primary.max_population_deviation_scaled > 0 {
        let total: i128 = instance.populations.iter().map(|&value| value as i128).sum();
        let numerator = instance.k_right as i128 * total; let divisor = instance.k_parent as i128;
        let low = i64::try_from(numerator.div_euclid(divisor)).map_err(|e| e.to_string())?;
        let high = i64::try_from((numerator + divisor - 1).div_euclid(divisor)).map_err(|e| e.to_string())?;
        for (side, population) in [("low", low), ("high", high)] {
            models.insert(format!("02-{side}-boundary-right-{population}"), compile_certified_split_compact_boundary_branch(&instance, &discovery, population).map_err(|e| e.to_string())?);
        }
    }
    if models.keys().ne(request.proofs.keys()) { return Err("Native certificate requires exactly every compiled decision proof, including population branches".into()); }
    if models.values().map(|model| model.opb.len()).sum::<usize>() > 25 * 1024 * 1024 { return Err("Native model package exceeds 25 MiB".into()); }
    let mut decisions = BTreeMap::new();
    for (stem, model) in models {
        if model.request.request_id != model.request.compute_id().map_err(|e| e.to_string())? { return Err("Native request id mismatch".into()); }
        let opb_sha256 = hash(model.opb.as_bytes());
        if model.request.opb_sha256 != format!("sha256:{opb_sha256}") { return Err("Native request OPB hash mismatch before proof checking".into()); }
        let proof = request.proofs[&stem].clone(); let proof_sha256 = hash(proof.as_bytes());
        let checked = check(Request { opb_sha256, opb: model.opb, proof_sha256: proof_sha256.clone(), proof, require_unsat: true })
            .map_err(|error| format!("{stem}: {error}"))?;
        let request_json = serde_json::to_string_pretty(&model.request).map_err(|e| e.to_string())?;
        decisions.insert(stem, CheckedDecision { request_id: model.request.request_id, opb_sha256: model.request.opb_sha256, request_json, proof_sha256, checker: checked });
    }
    let mut certificate = CertifiedExternalSplitCertificate {
        schema_version: "certified-recursive-bisection-external-split-certificate-v1", certificate_id: String::new(),
        instance_hash: instance.hash().map_err(|e| e.to_string())?, discovery_id: discovery.discovery_id,
        model_id: instance.model_id, node_path: instance.node_path, parent_certificate_id: instance.parent_certificate_id,
        instance_source_sha256: hash(request.instance_json.as_bytes()), discovery_source_sha256: hash(request.discovery_json.as_bytes()),
        compiler_source_sha256: hash(include_bytes!("../../../crates/bisect-ilp/src/proof_backend.rs")),
        result: CertifiedSplitResult::Optimal { assignment: discovery.objective.canonical_assignment.clone(), objective: discovery.objective },
        proof_kind: "veripb-compact-decisions", decisions,
        scope: "Canonical optimum for this supplied native split instance, established by freshly compiled decision models and concrete VeriPB UNSAT checking. Recursive tree assembly, global plan optimum, source truth and legal compliance are separate requirements.",
    };
    let mut projection = serde_json::to_value(&certificate).map_err(|e| e.to_string())?;
    projection.as_object_mut().expect("certificate object").remove("certificate_id");
    certificate.certificate_id = format!("sha256:{}", hash(&serde_json::to_vec(&projection).map_err(|e| e.to_string())?));
    if let Some(expected) = request.expected_certificate {
        if expected != serde_json::to_value(&certificate).map_err(|e| e.to_string())? { return Err("Imported native external certificate differs from fresh proof verification".into()); }
    }
    Ok(certificate)
}
