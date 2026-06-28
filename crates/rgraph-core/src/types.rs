use crate::*;

pub(crate) const EPSILON: f64 = 1e-9;

/// Directed edge exposed by a graph adapter.
///
/// `weight` is an abstract non-negative cost. Domain crates own the unit:
/// miles, minutes, population penalty, or any other interpretation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedEdge<E> {
    pub id: E,
    pub target: usize,
    pub weight: f64,
}

/// Minimal directed weighted graph interface for deterministic graph kernels.
pub trait DirectedWeightedGraph {
    type EdgeId: Copy + Debug + Eq + Hash + Ord;

    fn node_count(&self) -> usize;

    fn outgoing_edges(&self, source: usize) -> Vec<WeightedEdge<Self::EdgeId>>;
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum GraphError<E> {
    #[error("node index {node} is out of bounds for graph with {node_count} nodes")]
    NodeOutOfBounds { node: usize, node_count: usize },
    #[error("edge {edge_id:?} from {from} to {target} has invalid weight {weight}")]
    InvalidWeight {
        edge_id: E,
        from: usize,
        target: usize,
        weight: f64,
    },
    #[error("distance from {from} to {target} became non-finite: {distance}")]
    NonFiniteDistance {
        from: usize,
        target: usize,
        distance: f64,
    },
    #[error("shortest-path count for node {node} became non-finite: {count}")]
    NonFinitePathCount { node: usize, count: f64 },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EdgeCutError {
    #[error("assignment length {assignment_len} does not match adjacency length {adjacency_len}")]
    AssignmentLengthMismatch {
        adjacency_len: usize,
        assignment_len: usize,
    },
    #[error("neighbor index {neighbor} from node {node} is out of bounds for graph with {node_count} nodes")]
    NeighborOutOfBounds {
        node: usize,
        neighbor: usize,
        node_count: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryMetrics {
    pub selected_count: usize,
    pub complement_count: usize,
    pub selected_internal_edges: usize,
    pub complement_internal_edges: usize,
    pub boundary_edges: usize,
    pub selected_degree: usize,
    pub complement_degree: usize,
    pub total_edges: usize,
    pub conductance: f64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConnectorPathError {
    #[error("[INPUT] connector sources must not be empty")]
    EmptySources,
    #[error("[INPUT] connector targets must not be empty")]
    EmptyTargets,
    #[error(
        "[INPUT] connector {kind} node {node} is out of bounds for graph with {node_count} nodes"
    )]
    NodeOutOfBounds {
        kind: &'static str,
        node: usize,
        node_count: usize,
    },
    #[error("[INPUT] neighbor index {neighbor} from node {node} is out of bounds for graph with {node_count} nodes")]
    NeighborOutOfBounds {
        node: usize,
        neighbor: usize,
        node_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorPath {
    pub source: usize,
    pub target: usize,
    pub nodes: Vec<usize>,
    pub bridge_nodes: Vec<usize>,
    pub hop_count: usize,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ClusterSummaryError {
    #[error("[INPUT] cluster {cluster_index} must not be empty")]
    EmptyCluster { cluster_index: usize },
    #[error("[INPUT] cluster {cluster_index} node {node} is out of bounds for graph with {node_count} nodes")]
    NodeOutOfBounds {
        cluster_index: usize,
        node: usize,
        node_count: usize,
    },
    #[error(
        "[INPUT] node {node} appears in both cluster {first_cluster} and cluster {second_cluster}"
    )]
    DuplicateClusterNode {
        node: usize,
        first_cluster: usize,
        second_cluster: usize,
    },
    #[error("[INPUT] neighbor index {neighbor} from node {node} is out of bounds for graph with {node_count} nodes")]
    NeighborOutOfBounds {
        node: usize,
        neighbor: usize,
        node_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSummary {
    pub cluster_index: usize,
    pub nodes: Vec<usize>,
    pub representative_node: usize,
    pub internal_edges: usize,
    pub boundary_edges: usize,
    pub volume: usize,
    pub conductance: f64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LabelConnectivityError {
    #[error("assignment length {assignment_len} does not match adjacency length {adjacency_len}")]
    AssignmentLengthMismatch {
        adjacency_len: usize,
        assignment_len: usize,
    },
    #[error("neighbor index {neighbor} from node {node} is out of bounds for graph with {node_count} nodes")]
    NeighborOutOfBounds {
        node: usize,
        neighbor: usize,
        node_count: usize,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SubsetConnectivityError {
    #[error("subset node index {node} is out of bounds for graph with {node_count} nodes")]
    NodeOutOfBounds { node: usize, node_count: usize },
    #[error("neighbor index {neighbor} from node {node} is out of bounds for graph with {node_count} nodes")]
    NeighborOutOfBounds {
        node: usize,
        neighbor: usize,
        node_count: usize,
    },
}

pub trait NodeIndex: Copy {
    fn to_usize(self) -> Option<usize>;
}

impl NodeIndex for usize {
    fn to_usize(self) -> Option<usize> {
        Some(self)
    }
}

impl NodeIndex for u32 {
    fn to_usize(self) -> Option<usize> {
        usize::try_from(self).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Predecessor<E> {
    pub node: usize,
    pub edge_id: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bridge<E> {
    pub source: usize,
    pub target: usize,
    pub edge_id: E,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShortestPathTree<E> {
    pub source: usize,
    pub distances: Vec<Option<f64>>,
    pub predecessors: Vec<Vec<Predecessor<E>>>,
    pub path_counts: Vec<f64>,
    pub visit_order: Vec<usize>,
}

impl<E> ShortestPathTree<E> {
    pub fn distance_to(&self, target: usize) -> Option<f64> {
        self.distances.get(target).copied().flatten()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HeapState {
    pub(crate) cost: f64,
    pub(crate) node: usize,
}

impl PartialEq for HeapState {
    fn eq(&self, other: &Self) -> bool {
        self.cost.total_cmp(&other.cost) == Ordering::Equal && self.node == other.node
    }
}

impl Eq for HeapState {}

impl PartialOrd for HeapState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.node.cmp(&self.node))
    }
}
