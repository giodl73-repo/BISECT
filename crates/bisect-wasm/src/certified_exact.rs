//! Lossless JSON text boundary for bounded native exact split/tree proofs.
use bisect_ilp::{CertifiedSplitArtifacts, CertifiedSplitInstance, CertifiedSplitResult, CertifiedBisectionTree};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

pub(crate) fn parse<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    if text.len() > 3 * 1024 * 1024 { return Err("Exact artifact exceeds 3 MiB.".into()); }
    serde_json::from_str(text).map_err(|e| format!("Invalid exact artifact: {e}"))
}
pub(crate) fn bounded(instance: &CertifiedSplitInstance) -> Result<(), String> {
    let n = instance.unit_ids.len();
    if !(2..=bisect_ilp::CERTIFIED_SPLIT_ENUMERATION_LIMIT).contains(&n)
        || !(2..=n).contains(&instance.k_parent)
        || instance.edges.len() > n * (n - 1) / 2
        || instance.node_path.len() > 24
        || instance.unit_ids.iter().any(|s| s.len() > 200) {
        return Err("Exact browser instances require 2–24 units, 2–unit-count seats, bounded IDs/path and canonical edges.".into());
    }
    instance.validate().map_err(|e| e.to_string())
}
fn split_output(artifacts: &CertifiedSplitArtifacts) -> Result<Value, String> {
    let (status, assignment, objective) = match &artifacts.certificate.result {
        CertifiedSplitResult::Optimal {assignment, objective} => ("optimal", Some(assignment), Some(json!({
            "max_population_deviation_scaled":objective.primary.max_population_deviation_scaled.to_string(),
            "total_population_deviation_scaled":objective.primary.total_population_deviation_scaled.to_string(),
            "weighted_boundary_cut":objective.primary.weighted_boundary_cut.to_string()
        }))),
        CertifiedSplitResult::Infeasible => ("infeasible", None, None),
    };
    Ok(json!({"schema_version":"bisect-certified-split-output-v1","status":status,
        "artifacts_json":serde_json::to_string(artifacts).map_err(|e|e.to_string())?,
        "certificate_id":artifacts.certificate.certificate_id,"assignment":assignment,"objective":objective,
        "scope":"Exact lexicographic population-deviation, weighted-boundary and canonical-assignment optimum for this supplied split model. This does not prove a global district-plan optimum or legal compliance."}))
}
pub fn solve_split(text: &str) -> Result<Value, String> {
    let instance: CertifiedSplitInstance = parse(text)?;
    bounded(&instance)?;
    let artifacts = bisect_ilp::solve_certified_split_bounded(&instance).map_err(|e|e.to_string())?;
    split_output(&artifacts)
}
pub fn verify_split(text: &str) -> Result<Value, String> {
    let a: CertifiedSplitArtifacts = parse(text)?;
    bounded(&a.instance)?;
    bisect_ilp::verify_certified_split_bounded(&a.instance, &a.certificate, &a.proof).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-certified-split-verification-v1","verified":true,"certificate_id":a.certificate.certificate_id,
        "scope":"Reenumerated the supplied bounded split and checked its certificate and proof; source truth and legal compliance are not established."}))
}
pub fn solve_tree(text: &str) -> Result<Value, String> {
    let instance: CertifiedSplitInstance = parse(text)?;
    bounded(&instance)?;
    let tree = bisect_ilp::solve_certified_bisection_tree_bounded(instance).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-certified-tree-output-v1","tree_id":tree.tree_id,"districts":tree.k,
        "tree_json":serde_json::to_string(&tree).map_err(|e|e.to_string())?,
        "scope":"Each recursive split is exact for its local certified model. The completed tree does not certify a globally optimal district plan or legal compliance."}))
}
pub(crate) fn read_tree(text: &str) -> Result<CertifiedBisectionTree, String> {
    let tree: CertifiedBisectionTree = parse(text)?;
    if !(2..=24).contains(&tree.k) || tree.nodes.is_empty() || tree.nodes.len() > 23 || tree.leaves.len() > 24 {
        return Err("Exact browser tree exceeds the bounded node/seat/leaf scope.".into());
    }
    for node in &tree.nodes { bounded(&node.instance)?; }
    Ok(tree)
}
pub fn verify_tree(text: &str) -> Result<Value, String> {
    let tree = read_tree(text)?;
    bisect_ilp::verify_certified_bisection_tree_bounded(&tree).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-certified-tree-verification-v1","verified":true,"tree_id":tree.tree_id,
        "scope":"Reenumerated local split proofs and checked recursive links and leaf coverage. Global plan optimality, source truth and legal compliance are not established."}))
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SingleArtifacts {
    instance: bisect_ilp::CertifiedSingleDistrictInstance,
    certificate: bisect_ilp::CertifiedSingleDistrictCertificate,
}
fn bounded_single(instance: &bisect_ilp::CertifiedSingleDistrictInstance) -> Result<(), String> {
    if instance.unit_ids.is_empty() || instance.unit_ids.len() > 100_000
        || instance.edges.len() > 200_000 || instance.unit_ids.iter().any(|id|id.len()>200) {
        return Err("Single-district browser instances require 1–100000 units, at most 200000 edges and bounded IDs.".into());
    }
    instance.validate().map_err(|e|e.to_string())
}
pub fn solve_single(text: &str) -> Result<Value, String> {
    let instance: bisect_ilp::CertifiedSingleDistrictInstance = parse(text)?;
    bounded_single(&instance)?;
    let certificate = bisect_ilp::build_certified_single_district(&instance).map_err(|e|e.to_string())?;
    let summary = json!({"schema_version":"bisect-certified-single-output-v1","status":"trivial-single-district",
        "districts":1,"unit_count":certificate.unit_count,"population_total":certificate.population_total.to_string(),
        "weighted_boundary_cut":certificate.weighted_boundary_cut.to_string(),"connected":certificate.connected,
        "certificate_id":certificate.certificate_id,
        "scope":"Every supplied unit belongs to district zero; connectivity and exact population are checked. No split, search or global-plan optimality claim is made; source truth and legal compliance remain unverified."});
    let artifacts = SingleArtifacts {instance,certificate};
    let mut output = summary;
    output["artifacts_json"] = json!(serde_json::to_string(&artifacts).map_err(|e|e.to_string())?);
    Ok(output)
}
pub fn verify_single(text: &str) -> Result<Value, String> {
    let artifacts: SingleArtifacts = parse(text)?;
    bounded_single(&artifacts.instance)?;
    bisect_ilp::verify_certified_single_district(&artifacts.instance,&artifacts.certificate).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-certified-single-verification-v1","verified":true,
        "verification_method":"Checked trivial single-district certificate","districts":1,
        "certificate_id":artifacts.certificate.certificate_id,
        "population_total":artifacts.certificate.population_total.to_string(),
        "scope":"Checked the complete district-zero assignment, connected supplied graph, population sum and native certificate identity. No split was performed; source truth and legal compliance remain unverified."}))
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanonicalArtifacts {
    instance: bisect_ilp::ExactCanonicalInstance,
    certificate: bisect_ilp::ExactCanonicalCertificate,
    proof: bisect_ilp::ExactProofTranscript,
}
fn bounded_canonical(instance: &bisect_ilp::ExactCanonicalInstance) -> Result<(), String> {
    let n = instance.unit_ids.len();
    if !(2..=bisect_ilp::EXACT_ENUMERATION_LIMIT).contains(&n)
        || instance.edges.len() > n * (n - 1) / 2
        || instance.unit_ids.iter().any(|id|id.len()>200) {
        return Err("Canonical browser benchmark requires 2–24 units, bounded IDs and canonical edges.".into());
    }
    instance.validate().map_err(|e|e.to_string())
}
pub fn solve_canonical(text: &str) -> Result<Value, String> {
    let instance: bisect_ilp::ExactCanonicalInstance = parse(text)?;
    bounded_canonical(&instance)?;
    let native = bisect_ilp::solve_exact_canonical_artifacts(&instance).map_err(|e|e.to_string())?;
    let (status,assignment,objective) = match &native.certificate.result {
        bisect_ilp::ExactCertificateResult::Optimal {assignment,objective} => ("optimal",Some(assignment),Some(json!({
            "max_population_deviation_scaled":objective.primary.max_population_deviation_scaled.to_string(),
            "total_population_deviation_scaled":objective.primary.total_population_deviation_scaled.to_string(),
            "weighted_boundary_cut":objective.primary.weighted_boundary_cut.to_string()
        }))),
        bisect_ilp::ExactCertificateResult::Infeasible => ("infeasible",None,None),
    };
    let mut output = json!({"schema_version":"bisect-certified-canonical-output-v1","status":status,
        "districts":2,"assignment":assignment,"objective":objective,"certificate_id":native.certificate.certificate_id,
        "scope":"Exact complete two-district assignment for the supplied canonical model: connected districts, lexicographic population deviation, weighted boundary and assignment. Optimality is specific to this model; source truth and legal compliance remain unverified."});
    let artifacts = CanonicalArtifacts {instance,certificate:native.certificate,proof:native.proof};
    output["artifacts_json"] = json!(serde_json::to_string(&artifacts).map_err(|e|e.to_string())?);
    Ok(output)
}
pub fn verify_canonical(text: &str) -> Result<Value, String> {
    let artifacts: CanonicalArtifacts = parse(text)?;
    bounded_canonical(&artifacts.instance)?;
    bisect_ilp::verify_exact_canonical_artifacts(&artifacts.instance,&artifacts.certificate,&artifacts.proof).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-certified-canonical-verification-v1","verified":true,"districts":2,
        "certificate_id":artifacts.certificate.certificate_id,
        "scope":"Reenumerated the complete supplied two-district canonical model and checked its certificate and proof transcript. Source truth and legal compliance remain unverified."}))
}
