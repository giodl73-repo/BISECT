use crate::{
    canonical_orientation_rule, canonical_seat_split, certified_split::canonical_hash,
    certified_split_unit_universe_hash, solve_certified_split_bounded,
    verify_certified_split_bounded, CertifiedSplitArtifacts, CertifiedSplitError,
    CertifiedSplitInstance, CertifiedSplitResult, ExactEdge,
    CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION, CERTIFIED_SPLIT_MODEL_ID,
};
use bisect_core::BisectionTree;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub const CERTIFIED_BISECTION_TREE_SCHEMA_VERSION: &str = "certified-recursive-bisection-tree-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedBisectionLeaf {
    pub leaf_id: String,
    pub node_path: String,
    pub parent_certificate_id: String,
    pub unit_universe_hash: String,
    pub unit_ids: Vec<String>,
    pub district_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedBisectionTree {
    pub schema_version: String,
    pub tree_id: String,
    pub root_unit_universe_hash: String,
    pub k: usize,
    /// Split artifacts in the exact BFS order of `bisect_core::BisectionTree`.
    pub nodes: Vec<CertifiedSplitArtifacts>,
    /// One-seat leaves in lexicographic binary-path order.
    pub leaves: Vec<CertifiedBisectionLeaf>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CertifiedTreeError {
    #[error(transparent)]
    Split(#[from] CertifiedSplitError),
    #[error("root split instance must use the empty path and no parent certificate")]
    InvalidRoot,
    #[error("certified recursive trees require at least two districts")]
    InvalidTreeSeatCount,
    #[error("unsupported certified bisection tree schema: {0}")]
    TreeSchema(String),
    #[error("tree id mismatch: expected {expected}, found {found}")]
    TreeIdMismatch { expected: String, found: String },
    #[error("tree node set or BFS order does not match the canonical schedule")]
    NodeScheduleMismatch,
    #[error("tree contains duplicate node or leaf paths")]
    DuplicatePath,
    #[error("node {0} is infeasible and cannot produce a complete tree")]
    InfeasibleNode(String),
    #[error("child instance at path {0} does not match the certified parent partition")]
    ChildInstanceMismatch(String),
    #[error("leaf at path {0} does not match the certified parent partition")]
    LeafMismatch(String),
    #[error("leaf set does not provide canonical one-seat coverage")]
    LeafSetMismatch,
}

impl CertifiedBisectionLeaf {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            node_path: &'a str,
            parent_certificate_id: &'a str,
            unit_universe_hash: &'a str,
            unit_ids: &'a [String],
            district_index: usize,
        }
        canonical_hash(&Projection {
            node_path: &self.node_path,
            parent_certificate_id: &self.parent_certificate_id,
            unit_universe_hash: &self.unit_universe_hash,
            unit_ids: &self.unit_ids,
            district_index: self.district_index,
        })
    }
}

impl CertifiedBisectionTree {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            root_unit_universe_hash: &'a str,
            k: usize,
            nodes: &'a [CertifiedSplitArtifacts],
            leaves: &'a [CertifiedBisectionLeaf],
        }
        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            root_unit_universe_hash: &self.root_unit_universe_hash,
            k: self.k,
            nodes: &self.nodes,
            leaves: &self.leaves,
        })
    }
}

pub fn solve_certified_bisection_tree_bounded(
    root: CertifiedSplitInstance,
) -> Result<CertifiedBisectionTree, CertifiedTreeError> {
    root.validate()?;
    if !root.node_path.is_empty() || root.parent_certificate_id.is_some() {
        return Err(CertifiedTreeError::InvalidRoot);
    }
    let k = root.k_parent;
    let schedule = BisectionTree::from_k(k);
    let mut nodes = Vec::new();
    let mut pending_leaves = Vec::new();
    solve_node(root, &mut nodes, &mut pending_leaves)?;

    let order = schedule
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.path.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    nodes.sort_by_key(|node| order[&node_path(node)]);
    pending_leaves.sort_by(|left, right| left.0.cmp(&right.0));
    let mut leaves = pending_leaves
        .into_iter()
        .enumerate()
        .map(
            |(district_index, (node_path, parent_certificate_id, unit_ids))| {
                let unit_universe_hash = certified_split_unit_universe_hash(&unit_ids)?;
                let mut leaf = CertifiedBisectionLeaf {
                    leaf_id: String::new(),
                    node_path,
                    parent_certificate_id,
                    unit_universe_hash,
                    unit_ids,
                    district_index,
                };
                leaf.leaf_id = leaf.compute_id()?;
                Ok(leaf)
            },
        )
        .collect::<Result<Vec<_>, CertifiedSplitError>>()?;
    leaves.sort_by(|left, right| left.node_path.cmp(&right.node_path));

    let mut tree = CertifiedBisectionTree {
        schema_version: CERTIFIED_BISECTION_TREE_SCHEMA_VERSION.to_string(),
        tree_id: String::new(),
        root_unit_universe_hash: nodes[0].instance.unit_universe_hash.clone(),
        k,
        nodes,
        leaves,
    };
    tree.tree_id = tree.compute_id()?;
    verify_certified_bisection_tree_bounded(&tree)?;
    Ok(tree)
}

pub fn verify_certified_bisection_tree_bounded(
    tree: &CertifiedBisectionTree,
) -> Result<(), CertifiedTreeError> {
    if tree.schema_version != CERTIFIED_BISECTION_TREE_SCHEMA_VERSION {
        return Err(CertifiedTreeError::TreeSchema(tree.schema_version.clone()));
    }
    if tree.k < 2 {
        return Err(CertifiedTreeError::InvalidTreeSeatCount);
    }
    let expected_tree_id = tree.compute_id()?;
    if tree.tree_id != expected_tree_id {
        return Err(CertifiedTreeError::TreeIdMismatch {
            expected: expected_tree_id,
            found: tree.tree_id.clone(),
        });
    }
    let schedule = BisectionTree::from_k(tree.k);
    let submitted_paths = tree.nodes.iter().map(node_path).collect::<Vec<_>>();
    let expected_paths = schedule
        .nodes
        .iter()
        .map(|node| node.path.as_str())
        .collect::<Vec<_>>();
    if submitted_paths != expected_paths {
        return Err(CertifiedTreeError::NodeScheduleMismatch);
    }
    let node_map = tree
        .nodes
        .iter()
        .map(|node| (node.instance.node_path.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let leaf_map = tree
        .leaves
        .iter()
        .map(|leaf| (leaf.node_path.as_str(), leaf))
        .collect::<BTreeMap<_, _>>();
    if node_map.len() != tree.nodes.len() || leaf_map.len() != tree.leaves.len() {
        return Err(CertifiedTreeError::DuplicatePath);
    }
    if tree.nodes.is_empty()
        || !tree.nodes[0].instance.node_path.is_empty()
        || tree.nodes[0].instance.parent_certificate_id.is_some()
        || tree.root_unit_universe_hash != tree.nodes[0].instance.unit_universe_hash
    {
        return Err(CertifiedTreeError::InvalidRoot);
    }

    let schedule_by_path = schedule
        .nodes
        .iter()
        .map(|node| (node.path.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let expected_leaf_paths = canonical_leaf_paths(tree.k);
    let submitted_leaf_paths = tree
        .leaves
        .iter()
        .map(|leaf| leaf.node_path.clone())
        .collect::<Vec<_>>();
    if submitted_leaf_paths != expected_leaf_paths || tree.leaves.len() != tree.k {
        return Err(CertifiedTreeError::LeafSetMismatch);
    }

    for node in &tree.nodes {
        let path = node.instance.node_path.as_str();
        let schedule_node = schedule_by_path
            .get(path)
            .ok_or(CertifiedTreeError::NodeScheduleMismatch)?;
        if node.instance.k_parent != schedule_node.k
            || node.instance.k_left != schedule_node.k_left
            || node.instance.k_right != schedule_node.k_right
        {
            return Err(CertifiedTreeError::NodeScheduleMismatch);
        }
        verify_certified_split_bounded(&node.instance, &node.certificate, &node.proof)?;
        let CertifiedSplitResult::Optimal { assignment, .. } = &node.certificate.result else {
            return Err(CertifiedTreeError::InfeasibleNode(path.to_string()));
        };
        for (label, child_k) in [(0_u8, node.instance.k_left), (1_u8, node.instance.k_right)] {
            let child_path = format!("{path}{label}");
            let child_instance = derive_child_instance(
                &node.instance,
                assignment,
                label,
                child_k,
                child_path.clone(),
                node.certificate.certificate_id.clone(),
            )?;
            if child_k > 1 {
                let submitted = node_map
                    .get(child_path.as_str())
                    .ok_or_else(|| CertifiedTreeError::ChildInstanceMismatch(child_path.clone()))?;
                if submitted.instance != child_instance {
                    return Err(CertifiedTreeError::ChildInstanceMismatch(child_path));
                }
            } else {
                let submitted = leaf_map
                    .get(child_path.as_str())
                    .ok_or_else(|| CertifiedTreeError::LeafMismatch(child_path.clone()))?;
                let expected_index = expected_leaf_paths
                    .binary_search(&child_path)
                    .map_err(|_| CertifiedTreeError::LeafSetMismatch)?;
                let mut expected = CertifiedBisectionLeaf {
                    leaf_id: String::new(),
                    node_path: child_path.clone(),
                    parent_certificate_id: node.certificate.certificate_id.clone(),
                    unit_universe_hash: child_instance.unit_universe_hash,
                    unit_ids: child_instance.unit_ids,
                    district_index: expected_index,
                };
                expected.leaf_id = expected.compute_id()?;
                if **submitted != expected {
                    return Err(CertifiedTreeError::LeafMismatch(child_path));
                }
            }
        }
    }

    let root_units = tree.nodes[0]
        .instance
        .unit_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut leaf_units = BTreeSet::new();
    for leaf in &tree.leaves {
        for unit_id in &leaf.unit_ids {
            if !leaf_units.insert(unit_id.clone()) {
                return Err(CertifiedTreeError::LeafSetMismatch);
            }
        }
    }
    if leaf_units != root_units {
        return Err(CertifiedTreeError::LeafSetMismatch);
    }
    Ok(())
}

fn solve_node(
    instance: CertifiedSplitInstance,
    nodes: &mut Vec<CertifiedSplitArtifacts>,
    leaves: &mut Vec<(String, String, Vec<String>)>,
) -> Result<(), CertifiedTreeError> {
    let artifacts = solve_certified_split_bounded(&instance)?;
    let CertifiedSplitResult::Optimal { assignment, .. } = &artifacts.certificate.result else {
        return Err(CertifiedTreeError::InfeasibleNode(
            instance.node_path.clone(),
        ));
    };
    let parent_certificate_id = artifacts.certificate.certificate_id.clone();
    for (label, child_k) in [(0_u8, instance.k_left), (1_u8, instance.k_right)] {
        let child_path = format!("{}{label}", instance.node_path);
        let child = derive_child_instance(
            &instance,
            assignment,
            label,
            child_k,
            child_path.clone(),
            parent_certificate_id.clone(),
        )?;
        if child_k == 1 {
            leaves.push((child_path, parent_certificate_id.clone(), child.unit_ids));
        } else {
            solve_node(child, nodes, leaves)?;
        }
    }
    nodes.push(artifacts);
    Ok(())
}

fn derive_child_instance(
    parent: &CertifiedSplitInstance,
    assignment: &[u8],
    label: u8,
    child_k: usize,
    node_path: String,
    parent_certificate_id: String,
) -> Result<CertifiedSplitInstance, CertifiedSplitError> {
    let selected = assignment
        .iter()
        .enumerate()
        .filter_map(|(index, &assigned)| (assigned == label).then_some(index))
        .collect::<Vec<_>>();
    let mut new_index = vec![None; parent.unit_ids.len()];
    for (index, &old_index) in selected.iter().enumerate() {
        new_index[old_index] = Some(index);
    }
    let unit_ids = selected
        .iter()
        .map(|&index| parent.unit_ids[index].clone())
        .collect::<Vec<_>>();
    let populations = selected
        .iter()
        .map(|&index| parent.populations[index])
        .collect::<Vec<_>>();
    let edges = parent
        .edges
        .iter()
        .filter_map(|edge| match (new_index[edge.left], new_index[edge.right]) {
            (Some(left), Some(right)) => Some(ExactEdge {
                left,
                right,
                weight: edge.weight,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    let unit_universe_hash = certified_split_unit_universe_hash(&unit_ids)?;
    let (k_left, k_right) = if child_k > 1 {
        canonical_seat_split(child_k)?
    } else {
        (0, 1)
    };
    Ok(CertifiedSplitInstance {
        schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
        node_path,
        parent_certificate_id: Some(parent_certificate_id),
        unit_universe_hash,
        unit_ids,
        populations,
        edges,
        k_parent: child_k,
        k_left,
        k_right,
        orientation_rule: canonical_orientation_rule(k_left, k_right),
    })
}

fn canonical_leaf_paths(k: usize) -> Vec<String> {
    let mut queue = VecDeque::from([(k, String::new())]);
    let mut leaves = Vec::new();
    while let Some((seats, path)) = queue.pop_front() {
        if seats == 1 {
            leaves.push(path);
        } else {
            let left = seats / 2;
            queue.push_back((left, format!("{path}0")));
            queue.push_back((seats - left, format!("{path}1")));
        }
    }
    leaves.sort();
    leaves
}

fn node_path(node: &CertifiedSplitArtifacts) -> &str {
    node.instance.node_path.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_root(k: usize, unit_count: usize) -> CertifiedSplitInstance {
        let unit_ids = (0..unit_count)
            .map(|unit| format!("u{unit:02}"))
            .collect::<Vec<_>>();
        let edges = (0..unit_count - 1)
            .map(|left| ExactEdge {
                left,
                right: left + 1,
                weight: 1,
            })
            .collect();
        let (k_left, k_right) = canonical_seat_split(k).unwrap();
        CertifiedSplitInstance {
            schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1; unit_count],
            edges,
            k_parent: k,
            k_left,
            k_right,
            orientation_rule: canonical_orientation_rule(k_left, k_right),
        }
    }

    #[test]
    fn four_seat_tree_has_three_splits_and_four_exact_leaves() {
        let tree = solve_certified_bisection_tree_bounded(path_root(4, 8)).unwrap();
        assert_eq!(
            tree.nodes
                .iter()
                .map(|node| node.instance.node_path.as_str())
                .collect::<Vec<_>>(),
            vec!["", "0", "1"]
        );
        assert_eq!(tree.leaves.len(), 4);
        assert_eq!(
            tree.leaves
                .iter()
                .map(|leaf| leaf.node_path.as_str())
                .collect::<Vec<_>>(),
            vec!["00", "01", "10", "11"]
        );
        assert_eq!(verify_certified_bisection_tree_bounded(&tree), Ok(()));
    }

    #[test]
    fn five_seat_tree_preserves_two_three_schedule() {
        let tree = solve_certified_bisection_tree_bounded(path_root(5, 10)).unwrap();
        assert_eq!(tree.nodes[0].instance.k_left, 2);
        assert_eq!(tree.nodes[0].instance.k_right, 3);
        assert_eq!(
            tree.nodes
                .iter()
                .map(|node| (node.instance.node_path.as_str(), node.instance.k_parent))
                .collect::<Vec<_>>(),
            vec![("", 5), ("0", 2), ("1", 3), ("11", 2)]
        );
        assert_eq!(tree.leaves.len(), 5);
        assert_eq!(verify_certified_bisection_tree_bounded(&tree), Ok(()));
    }

    #[test]
    fn tree_verifier_rejects_child_universe_tamper() {
        let mut tree = solve_certified_bisection_tree_bounded(path_root(4, 8)).unwrap();
        tree.nodes[1].instance.unit_ids.swap(0, 1);
        tree.nodes[1].instance.unit_universe_hash =
            certified_split_unit_universe_hash(&tree.nodes[1].instance.unit_ids).unwrap();
        tree.tree_id = tree.compute_id().unwrap();
        assert!(matches!(
            verify_certified_bisection_tree_bounded(&tree),
            Err(CertifiedTreeError::ChildInstanceMismatch(_))
                | Err(CertifiedTreeError::Split(
                    CertifiedSplitError::NonCanonicalUnitIds
                ))
        ));
    }

    #[test]
    fn tree_verifier_rejects_missing_leaf() {
        let mut tree = solve_certified_bisection_tree_bounded(path_root(4, 8)).unwrap();
        tree.leaves.pop();
        tree.tree_id = tree.compute_id().unwrap();
        assert_eq!(
            verify_certified_bisection_tree_bounded(&tree),
            Err(CertifiedTreeError::LeafSetMismatch)
        );
    }
}
