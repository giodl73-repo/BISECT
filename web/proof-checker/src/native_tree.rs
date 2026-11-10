//! Assemble externally proved splits with the native induced-child contract.
use std::collections::{BTreeMap, VecDeque};
use serde::{Deserialize, Serialize};
use bisect_ilp::{CertifiedSplitInstance, CertifiedSplitResult, CertifiedBisectionLeaf, certified_split_unit_universe_hash};
use bisect_ilp::certified_tree::derive_certified_split_children;
use crate::{hash, native_split::{AdoptRequest, CertifiedExternalSplitCertificate, adopt}};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptTreeRequest {
    pub operation: String,
    pub root_instance_json: String,
    pub nodes: Vec<AdoptRequest>,
    #[serde(default)]
    pub expected_tree: Option<serde_json::Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeriveChildrenRequest {
    pub operation: String,
    pub parent_request: AdoptRequest,
}
pub fn derive_proved_children(request: DeriveChildrenRequest) -> Result<serde_json::Value, String> {
    if request.operation != "derive-native-proved-children" { return Err("Unsupported child derivation operation".into()); }
    let parent: CertifiedSplitInstance = serde_json::from_str(&request.parent_request.instance_json).map_err(|e| e.to_string())?;
    let certificate = adopt(request.parent_request)?;
    let CertifiedSplitResult::Optimal { assignment, .. } = &certificate.result else { return Err("Proved parent must be optimal".into()); };
    let children = derive_certified_split_children(&parent, assignment, &certificate.certificate_id).map_err(|e| e.to_string())?;
    let texts = children.iter().map(serde_json::to_string_pretty).collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({"schema_version":"bisect-native-proved-children-v1", "parent_certificate":certificate, "children_json":texts, "scope":"Fresh concrete parent proof verification and native induced-child derivation. One-seat children are leaves; no unproved child split is adopted."}))
}
#[derive(Serialize)]
pub struct CertifiedExternalTree {
    pub schema_version: &'static str,
    pub tree_id: String,
    pub root_instance_hash: String,
    pub k: usize,
    pub nodes: Vec<CertifiedExternalSplitCertificate>,
    pub leaves: Vec<CertifiedBisectionLeaf>,
    pub assignments: BTreeMap<String, usize>,
    pub scope: &'static str,
}
pub fn adopt_tree(request: AdoptTreeRequest) -> Result<CertifiedExternalTree, String> {
    if request.operation != "adopt-native-external-tree" || request.root_instance_json.len() > crate::MAX_MODEL { return Err("Unsupported external tree request or oversized root".into()); }
    let root: CertifiedSplitInstance = serde_json::from_str(&request.root_instance_json).map_err(|e| e.to_string())?;
    root.validate().map_err(|e| e.to_string())?;
    if !root.node_path.is_empty() || root.parent_certificate_id.is_some() || root.k_parent > 1024 || root.k_parent > root.unit_ids.len() || request.nodes.len() != root.k_parent - 1 { return Err("External tree requires a canonical root, 2–1024 seats and exactly k-1 split nodes".into()); }
    if request.nodes.iter().flat_map(|node| node.proofs.values()).map(String::len).sum::<usize>() > 128 * 1024 * 1024 { return Err("External tree proofs exceed 128 MiB".into()); }
    let mut queue = VecDeque::from([root.clone()]); let mut nodes = Vec::new(); let mut leaves = Vec::new();
    for submitted in request.nodes {
        let expected = queue.pop_front().ok_or("Extra or reordered external split node")?;
        let actual: CertifiedSplitInstance = serde_json::from_str(&submitted.instance_json).map_err(|e| e.to_string())?;
        if actual != expected { return Err(format!("Native parent/induced-child instance mismatch at {}", expected.node_path)); }
        let certificate = adopt(submitted)?;
        let CertifiedSplitResult::Optimal { assignment, .. } = &certificate.result else { return Err("External tree node must be optimal".into()); };
        let children = derive_certified_split_children(&actual, assignment, &certificate.certificate_id).map_err(|e| e.to_string())?;
        for child in children {
            if child.k_parent == 1 {
                let mut leaf = CertifiedBisectionLeaf { leaf_id: String::new(), node_path: child.node_path, parent_certificate_id: certificate.certificate_id.clone(), unit_universe_hash: certified_split_unit_universe_hash(&child.unit_ids).map_err(|e| e.to_string())?, unit_ids: child.unit_ids, district_index: 0 };
                leaf.leaf_id = leaf.compute_id().map_err(|e| e.to_string())?; leaves.push(leaf);
            } else { queue.push_back(child); }
        }
        nodes.push(certificate);
    }
    if !queue.is_empty() || leaves.len() != root.k_parent { return Err("Incomplete external tree schedule or leaves".into()); }
    leaves.sort_by(|left, right| left.node_path.cmp(&right.node_path));
    let mut assignments = BTreeMap::new();
    for (index, leaf) in leaves.iter_mut().enumerate() {
        leaf.district_index = index;
        leaf.leaf_id = leaf.compute_id().map_err(|e| e.to_string())?;
        for unit in &leaf.unit_ids { if assignments.insert(unit.clone(), index).is_some() { return Err("Duplicate external tree leaf coverage".into()); } }
    }
    if assignments.keys().ne(root.unit_ids.iter()) { return Err("External tree leaves do not exactly cover the root units".into()); }
    let mut tree = CertifiedExternalTree { schema_version: "certified-recursive-bisection-external-tree-v1", tree_id: String::new(), root_instance_hash: root.hash().map_err(|e| e.to_string())?, k: root.k_parent, nodes, leaves, assignments,
        scope: "Every recursive split is freshly proved and linked to its exact native induced child instances, with canonical BFS scheduling and complete one-seat leaf coverage. Global plan optimality, source truth and legal compliance are not established." };
    let mut projection = serde_json::to_value(&tree).map_err(|e| e.to_string())?; projection.as_object_mut().expect("tree object").remove("tree_id");
    tree.tree_id = format!("sha256:{}", hash(&serde_json::to_vec(&projection).map_err(|e| e.to_string())?));
    if let Some(expected) = request.expected_tree { if expected != serde_json::to_value(&tree).map_err(|e| e.to_string())? { return Err("Imported external tree differs from full fresh verification".into()); } }
    Ok(tree)
}
