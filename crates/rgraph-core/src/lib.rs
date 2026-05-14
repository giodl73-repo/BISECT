use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use thiserror::Error;

const EPSILON: f64 = 1e-9;

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
struct HeapState {
    cost: f64,
    node: usize,
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

pub fn shortest_path_distance<G>(
    graph: &G,
    source: usize,
    target: usize,
) -> Result<Option<f64>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    shortest_path_distance_with_filter(graph, source, target, |_| true)
}

pub fn shortest_path_distance_with_filter<G, F>(
    graph: &G,
    source: usize,
    target: usize,
    edge_filter: F,
) -> Result<Option<f64>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool,
{
    validate_node(graph.node_count(), target)?;
    Ok(single_source_shortest_paths_with_filter(graph, source, edge_filter)?.distance_to(target))
}

pub fn single_source_shortest_paths<G>(
    graph: &G,
    source: usize,
) -> Result<ShortestPathTree<G::EdgeId>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    single_source_shortest_paths_with_filter(graph, source, |_| true)
}

pub fn single_source_shortest_paths_with_filter<G, F>(
    graph: &G,
    source: usize,
    edge_filter: F,
) -> Result<ShortestPathTree<G::EdgeId>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool,
{
    let node_count = graph.node_count();
    validate_node::<G::EdgeId>(node_count, source)?;

    let mut distances = vec![None; node_count];
    let mut predecessors = vec![Vec::new(); node_count];
    let mut path_counts = vec![0.0; node_count];
    let mut visit_order = Vec::new();
    let mut heap = BinaryHeap::new();

    distances[source] = Some(0.0);
    path_counts[source] = 1.0;
    heap.push(HeapState {
        cost: 0.0,
        node: source,
    });

    while let Some(HeapState { cost, node }) = heap.pop() {
        if let Some(best) = distances[node] {
            if cost > best + EPSILON {
                continue;
            }
        }

        visit_order.push(node);

        let mut edges = graph.outgoing_edges(node);
        edges.sort_by(|a, b| a.target.cmp(&b.target).then_with(|| a.id.cmp(&b.id)));

        for edge in edges {
            if !edge_filter(edge.id) {
                continue;
            }
            validate_weight(edge.id, node, edge.target, edge.weight)?;
            validate_node::<G::EdgeId>(node_count, edge.target)?;

            let next_cost = cost + edge.weight;
            let previous = distances[edge.target];

            match previous {
                None => {
                    validate_path_count::<G::EdgeId>(edge.target, path_counts[node])?;
                    distances[edge.target] = Some(next_cost);
                    predecessors[edge.target] = vec![Predecessor {
                        node,
                        edge_id: edge.id,
                    }];
                    path_counts[edge.target] = path_counts[node];
                    heap.push(HeapState {
                        cost: next_cost,
                        node: edge.target,
                    });
                }
                Some(prev_cost) if next_cost < prev_cost - EPSILON => {
                    validate_path_count::<G::EdgeId>(edge.target, path_counts[node])?;
                    distances[edge.target] = Some(next_cost);
                    predecessors[edge.target] = vec![Predecessor {
                        node,
                        edge_id: edge.id,
                    }];
                    path_counts[edge.target] = path_counts[node];
                    heap.push(HeapState {
                        cost: next_cost,
                        node: edge.target,
                    });
                }
                Some(prev_cost) if (next_cost - prev_cost).abs() <= EPSILON => {
                    predecessors[edge.target].push(Predecessor {
                        node,
                        edge_id: edge.id,
                    });
                    predecessors[edge.target].sort_by(|a, b| {
                        a.node.cmp(&b.node).then_with(|| a.edge_id.cmp(&b.edge_id))
                    });
                    path_counts[edge.target] += path_counts[node];
                    validate_path_count::<G::EdgeId>(edge.target, path_counts[edge.target])?;
                }
                Some(_) => {}
            }
        }
    }

    Ok(ShortestPathTree {
        source,
        distances,
        predecessors,
        path_counts,
        visit_order,
    })
}

pub fn reachable_nodes<G>(graph: &G, source: usize) -> Result<Vec<usize>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    reachable_nodes_with_filter(graph, source, |_| true)
}

pub fn reachable_nodes_with_filter<G, F>(
    graph: &G,
    source: usize,
    edge_filter: F,
) -> Result<Vec<usize>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool,
{
    let tree = single_source_shortest_paths_with_filter(graph, source, edge_filter)?;
    Ok(tree
        .distances
        .iter()
        .enumerate()
        .filter_map(|(node, distance)| distance.map(|_| node))
        .collect())
}

pub fn undirected_edge_cut<I, D>(
    adjacency: &[Vec<I>],
    assignment: &[D],
) -> Result<usize, EdgeCutError>
where
    I: NodeIndex,
    D: Eq,
{
    if adjacency.len() != assignment.len() {
        return Err(EdgeCutError::AssignmentLengthMismatch {
            adjacency_len: adjacency.len(),
            assignment_len: assignment.len(),
        });
    }

    undirected_edge_cut_by(adjacency, |node| &assignment[node])
}

pub fn undirected_edge_cut_by<I, D, F>(
    adjacency: &[Vec<I>],
    mut label_of: F,
) -> Result<usize, EdgeCutError>
where
    I: NodeIndex,
    D: Eq,
    F: FnMut(usize) -> D,
{
    let mut cut_edges = std::collections::HashSet::new();
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            let Some(neighbor) = neighbor.to_usize() else {
                return Err(EdgeCutError::NeighborOutOfBounds {
                    node,
                    neighbor: usize::MAX,
                    node_count: adjacency.len(),
                });
            };
            if neighbor >= adjacency.len() {
                return Err(EdgeCutError::NeighborOutOfBounds {
                    node,
                    neighbor,
                    node_count: adjacency.len(),
                });
            }
            if node != neighbor && label_of(node) != label_of(neighbor) {
                cut_edges.insert(ordered_pair(node, neighbor));
            }
        }
    }
    Ok(cut_edges.len())
}

pub fn assignment_label_connected<I, D>(
    adjacency: &[Vec<I>],
    assignment: &[D],
    label: D,
) -> Result<bool, LabelConnectivityError>
where
    I: NodeIndex,
    D: Eq + Copy,
{
    validate_assignment_adjacency(adjacency, assignment)?;
    let undirected = undirected_index_adjacency_for_labels(adjacency)?;

    let Some(start) = assignment.iter().position(|&assigned| assigned == label) else {
        return Ok(false);
    };
    let member_count = assignment
        .iter()
        .filter(|&&assigned| assigned == label)
        .count();

    let mut seen = vec![false; assignment.len()];
    let mut stack = vec![start];
    seen[start] = true;
    let mut reached = 0usize;
    while let Some(node) = stack.pop() {
        reached += 1;
        for &neighbor in &undirected[node] {
            if assignment[neighbor] == label && !seen[neighbor] {
                seen[neighbor] = true;
                stack.push(neighbor);
            }
        }
    }

    Ok(reached == member_count)
}

pub fn assignment_labels_connected<I, D, L>(
    adjacency: &[Vec<I>],
    assignment: &[D],
    labels: L,
) -> Result<bool, LabelConnectivityError>
where
    I: NodeIndex,
    D: Eq + Copy,
    L: IntoIterator<Item = D>,
{
    for label in labels {
        if !assignment_label_connected(adjacency, assignment, label)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn node_subset_connected<I, N>(
    adjacency: &[Vec<I>],
    nodes: &[N],
) -> Result<bool, SubsetConnectivityError>
where
    I: NodeIndex,
    N: NodeIndex,
{
    if nodes.is_empty() {
        return Ok(true);
    }

    let node_count = adjacency.len();
    let mut in_subset = vec![false; node_count];
    let mut unique_nodes = Vec::new();
    for &node in nodes {
        let Some(node) = node.to_usize() else {
            return Err(SubsetConnectivityError::NodeOutOfBounds {
                node: usize::MAX,
                node_count,
            });
        };
        if node >= node_count {
            return Err(SubsetConnectivityError::NodeOutOfBounds { node, node_count });
        }
        if !in_subset[node] {
            in_subset[node] = true;
            unique_nodes.push(node);
        }
    }

    let undirected = undirected_index_adjacency_for_subset(adjacency)?;
    let mut seen = vec![false; node_count];
    let mut stack = vec![unique_nodes[0]];
    seen[unique_nodes[0]] = true;
    let mut reached = 0usize;
    while let Some(node) = stack.pop() {
        reached += 1;
        for &neighbor in &undirected[node] {
            if in_subset[neighbor] && !seen[neighbor] {
                seen[neighbor] = true;
                stack.push(neighbor);
            }
        }
    }

    Ok(reached == unique_nodes.len())
}

fn undirected_index_adjacency_for_labels<I>(
    adjacency: &[Vec<I>],
) -> Result<Vec<Vec<usize>>, LabelConnectivityError>
where
    I: NodeIndex,
{
    let node_count = adjacency.len();
    let mut undirected = vec![Vec::new(); node_count];
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            let neighbor = neighbor
                .to_usize()
                .expect("assignment adjacency was already validated");
            if node != neighbor {
                undirected[node].push(neighbor);
                undirected[neighbor].push(node);
            }
        }
    }
    for neighbors in &mut undirected {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    Ok(undirected)
}

fn undirected_index_adjacency_for_subset<I>(
    adjacency: &[Vec<I>],
) -> Result<Vec<Vec<usize>>, SubsetConnectivityError>
where
    I: NodeIndex,
{
    let node_count = adjacency.len();
    let mut undirected = vec![Vec::new(); node_count];
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            let Some(neighbor) = neighbor.to_usize() else {
                return Err(SubsetConnectivityError::NeighborOutOfBounds {
                    node,
                    neighbor: usize::MAX,
                    node_count,
                });
            };
            if neighbor >= node_count {
                return Err(SubsetConnectivityError::NeighborOutOfBounds {
                    node,
                    neighbor,
                    node_count,
                });
            }
            if node != neighbor {
                undirected[node].push(neighbor);
                undirected[neighbor].push(node);
            }
        }
    }
    for neighbors in &mut undirected {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    Ok(undirected)
}

fn validate_assignment_adjacency<I, D>(
    adjacency: &[Vec<I>],
    assignment: &[D],
) -> Result<(), LabelConnectivityError>
where
    I: NodeIndex,
{
    if adjacency.len() != assignment.len() {
        return Err(LabelConnectivityError::AssignmentLengthMismatch {
            adjacency_len: adjacency.len(),
            assignment_len: assignment.len(),
        });
    }
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            let Some(neighbor) = neighbor.to_usize() else {
                return Err(LabelConnectivityError::NeighborOutOfBounds {
                    node,
                    neighbor: usize::MAX,
                    node_count: adjacency.len(),
                });
            };
            if neighbor >= assignment.len() {
                return Err(LabelConnectivityError::NeighborOutOfBounds {
                    node,
                    neighbor,
                    node_count: adjacency.len(),
                });
            }
        }
    }
    Ok(())
}

pub fn connected_components<G>(graph: &G) -> Result<Vec<Vec<usize>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    connected_components_with_filter(graph, |_| true)
}

pub fn connected_components_with_filter<G, F>(
    graph: &G,
    edge_filter: F,
) -> Result<Vec<Vec<usize>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool + Copy,
{
    let nodes: Vec<usize> = (0..graph.node_count()).collect();
    connected_components_in_nodes_with_filter(graph, &nodes, edge_filter)
}

pub fn connected_components_in_nodes<G>(
    graph: &G,
    nodes: &[usize],
) -> Result<Vec<Vec<usize>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    connected_components_in_nodes_with_filter(graph, nodes, |_| true)
}

pub fn connected_components_in_nodes_with_filter<G, F>(
    graph: &G,
    nodes: &[usize],
    edge_filter: F,
) -> Result<Vec<Vec<usize>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool + Copy,
{
    let node_count = graph.node_count();
    let mut starts = nodes.to_vec();
    starts.sort_unstable();
    starts.dedup();
    for &node in &starts {
        validate_node::<G::EdgeId>(node_count, node)?;
    }

    let adjacency = undirected_adjacency(graph, edge_filter)?;
    let allowed: std::collections::HashSet<usize> = starts.iter().copied().collect();
    let mut visited = vec![false; node_count];
    let mut components = Vec::new();

    for start in starts {
        if visited[start] {
            continue;
        }

        let mut component = Vec::new();
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(node) = stack.pop() {
            component.push(node);
            for &target in &adjacency[node] {
                if !allowed.contains(&target) || visited[target] {
                    continue;
                }
                visited[target] = true;
                stack.push(target);
            }
        }
        component.sort_unstable();
        components.push(component);
    }

    Ok(components)
}

pub fn bridges<G>(graph: &G) -> Result<Vec<Bridge<G::EdgeId>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    bridges_with_filter(graph, |_| true)
}

pub fn bridges_with_filter<G, F>(
    graph: &G,
    edge_filter: F,
) -> Result<Vec<Bridge<G::EdgeId>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool + Copy,
{
    let node_count = graph.node_count();
    if node_count == 0 {
        return Ok(Vec::new());
    }

    let mut adjacency = vec![Vec::new(); node_count];
    let mut pair_edges: HashMap<(usize, usize), Vec<Bridge<G::EdgeId>>> = HashMap::new();
    let mut directed_counts: HashMap<(usize, usize), usize> = HashMap::new();

    for source in 0..node_count {
        let mut edges = graph.outgoing_edges(source);
        edges.sort_by(|a, b| a.target.cmp(&b.target).then_with(|| a.id.cmp(&b.id)));
        for edge in edges {
            if !edge_filter(edge.id) {
                continue;
            }
            validate_weight(edge.id, source, edge.target, edge.weight)?;
            validate_node::<G::EdgeId>(node_count, edge.target)?;
            if source == edge.target {
                continue;
            }

            let pair = ordered_pair(source, edge.target);
            let entries = pair_edges.entry(pair).or_default();
            if entries.is_empty() {
                adjacency[pair.0].push(pair.1);
                adjacency[pair.1].push(pair.0);
            }
            entries.push(Bridge {
                source,
                target: edge.target,
                edge_id: edge.id,
            });
            *directed_counts.entry((source, edge.target)).or_insert(0) += 1;
        }
    }

    for neighbors in &mut adjacency {
        neighbors.sort_unstable();
    }

    let mut discovery = vec![None; node_count];
    let mut low = vec![0; node_count];
    let mut parent = vec![None; node_count];
    let mut time = 0usize;
    let mut bridge_pairs = Vec::new();

    for node in 0..node_count {
        if discovery[node].is_none() {
            bridge_dfs(
                node,
                &adjacency,
                &mut discovery,
                &mut low,
                &mut parent,
                &mut time,
                &mut bridge_pairs,
            );
        }
    }

    let mut out = Vec::new();
    for pair in bridge_pairs {
        if let Some(entries) = pair_edges.get(&pair) {
            let has_parallel_same_direction = entries.iter().any(|entry| {
                directed_counts
                    .get(&(entry.source, entry.target))
                    .copied()
                    .unwrap_or(0)
                    > 1
            });
            if !has_parallel_same_direction {
                out.extend(entries.iter().copied());
            }
        }
    }
    out.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.edge_id.cmp(&b.edge_id))
    });
    Ok(out)
}

pub fn articulation_points<G>(graph: &G) -> Result<Vec<usize>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    articulation_points_with_filter(graph, |_| true)
}

pub fn articulation_points_with_filter<G, F>(
    graph: &G,
    edge_filter: F,
) -> Result<Vec<usize>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool + Copy,
{
    let adjacency = undirected_adjacency(graph, edge_filter)?;
    let node_count = adjacency.len();
    if node_count == 0 {
        return Ok(Vec::new());
    }

    let mut discovery = vec![None; node_count];
    let mut low = vec![0; node_count];
    let mut parent = vec![None; node_count];
    let mut time = 0usize;
    let mut is_articulation = vec![false; node_count];

    for node in 0..node_count {
        if discovery[node].is_none() {
            articulation_dfs(
                node,
                &adjacency,
                &mut discovery,
                &mut low,
                &mut parent,
                &mut time,
                &mut is_articulation,
            );
        }
    }

    Ok(is_articulation
        .iter()
        .enumerate()
        .filter_map(|(node, is_cut)| is_cut.then_some(node))
        .collect())
}

pub fn edge_betweenness<G>(graph: &G) -> Result<HashMap<G::EdgeId, f64>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
{
    edge_betweenness_with_filter(graph, |_| true)
}

pub fn edge_betweenness_with_filter<G, F>(
    graph: &G,
    edge_filter: F,
) -> Result<HashMap<G::EdgeId, f64>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool + Copy,
{
    let node_count = graph.node_count();
    if node_count == 0 {
        return Ok(HashMap::new());
    }

    let mut raw = HashMap::new();

    for source in 0..node_count {
        let tree = single_source_shortest_paths_with_filter(graph, source, edge_filter)?;
        let mut dependency = vec![0.0; node_count];

        for &w in tree.visit_order.iter().rev() {
            let sigma_w = tree.path_counts[w];
            if sigma_w <= 0.0 {
                continue;
            }

            let delta_w = dependency[w];
            for predecessor in &tree.predecessors[w] {
                let sigma_v = tree.path_counts[predecessor.node];
                let contribution = (sigma_v / sigma_w) * (1.0 + delta_w);
                dependency[predecessor.node] += contribution;
                *raw.entry(predecessor.edge_id).or_insert(0.0) += contribution;
            }
        }
    }

    let max = raw.values().copied().fold(0.0_f64, f64::max);
    if max > 0.0 {
        for value in raw.values_mut() {
            *value /= max;
        }
    }

    Ok(raw)
}

fn undirected_adjacency<G, F>(
    graph: &G,
    edge_filter: F,
) -> Result<Vec<Vec<usize>>, GraphError<G::EdgeId>>
where
    G: DirectedWeightedGraph,
    F: Fn(G::EdgeId) -> bool + Copy,
{
    let node_count = graph.node_count();
    let mut adjacency = vec![Vec::new(); node_count];
    for source in 0..node_count {
        let mut edges = graph.outgoing_edges(source);
        edges.sort_by(|a, b| a.target.cmp(&b.target).then_with(|| a.id.cmp(&b.id)));
        for edge in edges {
            if !edge_filter(edge.id) {
                continue;
            }
            validate_weight(edge.id, source, edge.target, edge.weight)?;
            validate_node::<G::EdgeId>(node_count, edge.target)?;
            if source == edge.target {
                continue;
            }
            adjacency[source].push(edge.target);
            adjacency[edge.target].push(source);
        }
    }

    for neighbors in &mut adjacency {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    Ok(adjacency)
}

fn articulation_dfs(
    node: usize,
    adjacency: &[Vec<usize>],
    discovery: &mut [Option<usize>],
    low: &mut [usize],
    parent: &mut [Option<usize>],
    time: &mut usize,
    is_articulation: &mut [bool],
) {
    discovery[node] = Some(*time);
    low[node] = *time;
    *time += 1;
    let mut child_count = 0usize;

    for &neighbor in &adjacency[node] {
        if discovery[neighbor].is_none() {
            child_count += 1;
            parent[neighbor] = Some(node);
            articulation_dfs(
                neighbor,
                adjacency,
                discovery,
                low,
                parent,
                time,
                is_articulation,
            );
            low[node] = low[node].min(low[neighbor]);

            if parent[node].is_none() && child_count > 1 {
                is_articulation[node] = true;
            }
            if parent[node].is_some()
                && low[neighbor] >= discovery[node].expect("visited node has discovery time")
            {
                is_articulation[node] = true;
            }
        } else if parent[node] != Some(neighbor) {
            low[node] = low[node].min(discovery[neighbor].expect("visited neighbor"));
        }
    }
}

fn bridge_dfs(
    node: usize,
    adjacency: &[Vec<usize>],
    discovery: &mut [Option<usize>],
    low: &mut [usize],
    parent: &mut [Option<usize>],
    time: &mut usize,
    bridge_pairs: &mut Vec<(usize, usize)>,
) {
    discovery[node] = Some(*time);
    low[node] = *time;
    *time += 1;

    for &neighbor in &adjacency[node] {
        if discovery[neighbor].is_none() {
            parent[neighbor] = Some(node);
            bridge_dfs(
                neighbor,
                adjacency,
                discovery,
                low,
                parent,
                time,
                bridge_pairs,
            );
            low[node] = low[node].min(low[neighbor]);
            if low[neighbor] > discovery[node].expect("visited node has discovery time") {
                bridge_pairs.push(ordered_pair(node, neighbor));
            }
        } else if parent[node] != Some(neighbor) {
            low[node] = low[node].min(discovery[neighbor].expect("visited neighbor"));
        }
    }
}

fn ordered_pair(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn validate_node<E>(node_count: usize, node: usize) -> Result<(), GraphError<E>> {
    if node >= node_count {
        return Err(GraphError::NodeOutOfBounds { node, node_count });
    }
    Ok(())
}

fn validate_weight<E>(
    edge_id: E,
    source: usize,
    target: usize,
    weight: f64,
) -> Result<(), GraphError<E>> {
    if !weight.is_finite() || weight < 0.0 {
        return Err(GraphError::InvalidWeight {
            edge_id,
            from: source,
            target,
            weight,
        });
    }
    Ok(())
}

fn validate_path_count<E>(node: usize, count: f64) -> Result<(), GraphError<E>> {
    if !count.is_finite() {
        return Err(GraphError::NonFinitePathCount { node, count });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TinyGraph {
        node_count: usize,
        edges: Vec<Vec<WeightedEdge<usize>>>,
    }

    impl TinyGraph {
        fn new(node_count: usize) -> Self {
            Self {
                node_count,
                edges: vec![Vec::new(); node_count],
            }
        }

        fn add_edge(&mut self, id: usize, source: usize, target: usize, weight: f64) {
            self.edges[source].push(WeightedEdge { id, target, weight });
        }
    }

    impl DirectedWeightedGraph for TinyGraph {
        type EdgeId = usize;

        fn node_count(&self) -> usize {
            self.node_count
        }

        fn outgoing_edges(&self, source: usize) -> Vec<WeightedEdge<Self::EdgeId>> {
            self.edges[source].clone()
        }
    }

    #[test]
    fn equal_shortest_paths_preserve_predecessors_and_counts() {
        let mut graph = TinyGraph::new(4);
        graph.add_edge(10, 0, 1, 1.0);
        graph.add_edge(11, 1, 3, 1.0);
        graph.add_edge(20, 0, 2, 1.0);
        graph.add_edge(21, 2, 3, 1.0);

        let tree = single_source_shortest_paths(&graph, 0).unwrap();

        assert_eq!(tree.distance_to(3), Some(2.0));
        assert_eq!(tree.path_counts[3], 2.0);
        assert_eq!(
            tree.predecessors[3],
            vec![
                Predecessor {
                    node: 1,
                    edge_id: 11
                },
                Predecessor {
                    node: 2,
                    edge_id: 21
                }
            ]
        );
    }

    #[test]
    fn shortest_path_count_overflow_is_rejected() {
        let mut graph = TinyGraph::new(1100);
        let mut edge_id = 0usize;
        for source in 0..1099 {
            graph.add_edge(edge_id, source, source + 1, 1.0);
            edge_id += 1;
            graph.add_edge(edge_id, source, source + 1, 1.0);
            edge_id += 1;
        }

        match single_source_shortest_paths(&graph, 0) {
            Err(GraphError::NonFinitePathCount { node, count }) => {
                assert!(node > 0);
                assert!(count.is_infinite());
            }
            other => panic!("expected path-count overflow error, got {other:?}"),
        }
    }

    #[test]
    fn edge_filter_can_disconnect_target() {
        let mut graph = TinyGraph::new(3);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);

        let distance = shortest_path_distance_with_filter(&graph, 0, 2, |edge_id| edge_id != 2)
            .expect("filtered shortest path should not fail");

        assert_eq!(distance, None);
        assert_eq!(
            reachable_nodes_with_filter(&graph, 0, |edge_id| edge_id != 2).unwrap(),
            vec![0, 1]
        );
    }

    #[test]
    fn one_node_graph_reaches_itself() {
        let graph = TinyGraph::new(1);

        let tree = single_source_shortest_paths(&graph, 0).unwrap();

        assert_eq!(tree.distance_to(0), Some(0.0));
        assert_eq!(tree.path_counts[0], 1.0);
        assert_eq!(reachable_nodes(&graph, 0).unwrap(), vec![0]);
    }

    #[test]
    fn connected_components_are_sorted_and_deterministic() {
        let mut graph = TinyGraph::new(5);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 0, 1.0);
        graph.add_edge(3, 3, 4, 1.0);

        assert_eq!(
            connected_components(&graph).unwrap(),
            vec![vec![0, 1], vec![2], vec![3, 4]]
        );
    }

    #[test]
    fn connected_components_treat_directed_adapter_as_weak_components() {
        let mut graph = TinyGraph::new(4);
        graph.add_edge(1, 1, 0, 1.0);
        graph.add_edge(2, 2, 1, 1.0);

        assert_eq!(
            connected_components(&graph).unwrap(),
            vec![vec![0, 1, 2], vec![3]]
        );
    }

    #[test]
    fn connected_components_can_be_restricted_to_node_subset() {
        let mut graph = TinyGraph::new(6);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);
        graph.add_edge(3, 3, 4, 1.0);

        assert_eq!(
            connected_components_in_nodes(&graph, &[4, 3, 1, 0]).unwrap(),
            vec![vec![0, 1], vec![3, 4]]
        );
    }

    #[test]
    fn connected_components_filter_can_remove_bridge() {
        let mut graph = TinyGraph::new(3);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);

        assert_eq!(
            connected_components_with_filter(&graph, |edge| edge != 2).unwrap(),
            vec![vec![0, 1], vec![2]]
        );
    }

    #[test]
    fn bridges_identify_tree_edges_and_ignore_cycle_edges() {
        let mut graph = TinyGraph::new(5);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);
        graph.add_edge(3, 2, 0, 1.0);
        graph.add_edge(4, 2, 3, 1.0);
        graph.add_edge(5, 3, 4, 1.0);

        let bridges = bridges(&graph).unwrap();

        assert_eq!(
            bridges,
            vec![
                Bridge {
                    source: 2,
                    target: 3,
                    edge_id: 4
                },
                Bridge {
                    source: 3,
                    target: 4,
                    edge_id: 5
                }
            ]
        );
    }

    #[test]
    fn bridges_return_reciprocal_adapter_edges_for_one_undirected_bridge() {
        let mut graph = TinyGraph::new(2);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 0, 1.0);

        let bridges = bridges(&graph).unwrap();

        assert_eq!(
            bridges,
            vec![
                Bridge {
                    source: 0,
                    target: 1,
                    edge_id: 1
                },
                Bridge {
                    source: 1,
                    target: 0,
                    edge_id: 2
                }
            ]
        );
    }

    #[test]
    fn bridges_ignore_parallel_same_direction_edges() {
        let mut graph = TinyGraph::new(2);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 0, 1, 1.0);

        assert!(bridges(&graph).unwrap().is_empty());
    }

    #[test]
    fn bridges_filter_can_create_bridge() {
        let mut graph = TinyGraph::new(3);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);
        graph.add_edge(3, 0, 2, 1.0);

        assert_eq!(
            bridges_with_filter(&graph, |edge| edge != 3).unwrap(),
            vec![
                Bridge {
                    source: 0,
                    target: 1,
                    edge_id: 1
                },
                Bridge {
                    source: 1,
                    target: 2,
                    edge_id: 2
                }
            ]
        );
    }

    #[test]
    fn articulation_points_identify_cut_vertices_and_ignore_cycle_vertices() {
        let mut graph = TinyGraph::new(6);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);
        graph.add_edge(3, 2, 0, 1.0);
        graph.add_edge(4, 2, 3, 1.0);
        graph.add_edge(5, 3, 4, 1.0);
        graph.add_edge(6, 4, 5, 1.0);
        graph.add_edge(7, 5, 3, 1.0);

        assert_eq!(articulation_points(&graph).unwrap(), vec![2, 3]);
    }

    #[test]
    fn articulation_points_handle_root_with_multiple_children() {
        let mut graph = TinyGraph::new(4);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 0, 2, 1.0);
        graph.add_edge(3, 0, 3, 1.0);

        assert_eq!(articulation_points(&graph).unwrap(), vec![0]);
    }

    #[test]
    fn articulation_points_filter_can_create_cut_vertex() {
        let mut graph = TinyGraph::new(3);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);
        graph.add_edge(3, 0, 2, 1.0);

        assert_eq!(
            articulation_points_with_filter(&graph, |edge| edge != 3).unwrap(),
            vec![1]
        );
    }

    #[test]
    fn invalid_source_node_is_rejected() {
        let graph = TinyGraph::new(1);

        let err = single_source_shortest_paths(&graph, 2).unwrap_err();

        assert_eq!(
            err,
            GraphError::NodeOutOfBounds {
                node: 2,
                node_count: 1
            }
        );
    }

    #[test]
    fn invalid_target_node_is_rejected() {
        let graph = TinyGraph::new(1);

        let err = shortest_path_distance(&graph, 0, 2).unwrap_err();

        assert_eq!(
            err,
            GraphError::NodeOutOfBounds {
                node: 2,
                node_count: 1
            }
        );
    }

    #[test]
    fn negative_weight_is_rejected() {
        let mut graph = TinyGraph::new(2);
        graph.add_edge(7, 0, 1, -1.0);

        let err = shortest_path_distance(&graph, 0, 1).unwrap_err();

        assert_eq!(
            err,
            GraphError::InvalidWeight {
                edge_id: 7,
                from: 0,
                target: 1,
                weight: -1.0
            }
        );
    }

    #[test]
    fn non_finite_weight_is_rejected() {
        let mut graph = TinyGraph::new(2);
        graph.add_edge(8, 0, 1, f64::INFINITY);

        let err = shortest_path_distance(&graph, 0, 1).unwrap_err();

        assert_eq!(
            err,
            GraphError::InvalidWeight {
                edge_id: 8,
                from: 0,
                target: 1,
                weight: f64::INFINITY
            }
        );
    }

    #[test]
    fn equal_shortest_paths_split_edge_betweenness() {
        let mut graph = TinyGraph::new(4);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 3, 1.0);
        graph.add_edge(3, 0, 2, 1.0);
        graph.add_edge(4, 2, 3, 1.0);

        let centrality = edge_betweenness(&graph).unwrap();

        let upper = centrality[&1] + centrality[&2];
        let lower = centrality[&3] + centrality[&4];
        assert!((upper - lower).abs() < 1e-9);
        assert!(centrality[&1] > 0.0);
    }

    #[test]
    fn non_shortest_direct_edge_receives_no_betweenness() {
        let mut graph = TinyGraph::new(3);
        graph.add_edge(1, 0, 1, 1.0);
        graph.add_edge(2, 1, 2, 1.0);
        graph.add_edge(3, 0, 2, 10.0);

        let centrality = edge_betweenness(&graph).unwrap();

        assert!(centrality[&1] > centrality.get(&3).copied().unwrap_or(0.0));
        assert!(centrality[&2] > centrality.get(&3).copied().unwrap_or(0.0));
    }

    #[test]
    fn empty_graph_has_empty_edge_betweenness() {
        let graph = TinyGraph::new(0);

        assert!(edge_betweenness(&graph).unwrap().is_empty());
    }

    #[test]
    fn undirected_edge_cut_counts_each_crossing_once() {
        let adjacency = vec![vec![1_usize, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let assignment = vec![0, 0, 1, 1];

        assert_eq!(undirected_edge_cut(&adjacency, &assignment).unwrap(), 2);
    }

    #[test]
    fn undirected_edge_cut_supports_u32_adjacency_and_assignment() {
        let adjacency = vec![vec![1_u32], vec![0, 2], vec![1]];
        let assignment = vec![1_u32, 2, 2];

        assert_eq!(undirected_edge_cut(&adjacency, &assignment).unwrap(), 1);
    }

    #[test]
    fn undirected_edge_cut_by_supports_map_defaults() {
        let adjacency = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];
        let assignment = std::collections::HashMap::from([(0usize, 0usize), (3, 1)]);

        assert_eq!(
            undirected_edge_cut_by(&adjacency, |node| assignment
                .get(&node)
                .copied()
                .unwrap_or(0))
            .unwrap(),
            1
        );
    }

    #[test]
    fn undirected_edge_cut_by_supports_set_membership() {
        let adjacency = vec![vec![1_usize, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let left = std::collections::HashSet::from([0usize, 1]);

        assert_eq!(
            undirected_edge_cut_by(&adjacency, |node| left.contains(&node)).unwrap(),
            2
        );
    }

    #[test]
    fn undirected_edge_cut_counts_asymmetric_adjacency_once() {
        let adjacency = vec![vec![], vec![0_usize, 0], vec![1]];
        let assignment = vec![0_usize, 1, 1];

        assert_eq!(undirected_edge_cut(&adjacency, &assignment).unwrap(), 1);
    }

    #[test]
    fn undirected_edge_cut_rejects_length_mismatch() {
        let adjacency = vec![vec![1_usize], vec![0]];
        let assignment = vec![0];

        assert_eq!(
            undirected_edge_cut(&adjacency, &assignment),
            Err(EdgeCutError::AssignmentLengthMismatch {
                adjacency_len: 2,
                assignment_len: 1
            })
        );
    }

    #[test]
    fn undirected_edge_cut_rejects_out_of_bounds_neighbor() {
        let adjacency = vec![vec![2_usize], vec![0]];
        let assignment = vec![0, 1];

        assert_eq!(
            undirected_edge_cut(&adjacency, &assignment),
            Err(EdgeCutError::NeighborOutOfBounds {
                node: 0,
                neighbor: 2,
                node_count: 2
            })
        );
    }

    #[test]
    fn node_subset_connected_accepts_contiguous_subset() {
        let adjacency = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];

        assert!(node_subset_connected(&adjacency, &[1_usize, 2, 3]).unwrap());
        assert!(node_subset_connected(&adjacency, &[2_usize]).unwrap());
        assert!(node_subset_connected(&adjacency, &[] as &[usize]).unwrap());
    }

    #[test]
    fn node_subset_connected_rejects_disconnected_subset() {
        let adjacency = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];

        assert!(!node_subset_connected(&adjacency, &[0_usize, 3]).unwrap());
    }

    #[test]
    fn node_subset_connected_treats_duplicate_nodes_as_one_subset_member() {
        let adjacency = vec![vec![1_usize], vec![0]];

        assert!(node_subset_connected(&adjacency, &[0_usize, 0, 1]).unwrap());
    }

    #[test]
    fn node_subset_connected_treats_adjacency_as_undirected() {
        let adjacency = vec![vec![], vec![0_usize], vec![1]];

        assert!(node_subset_connected(&adjacency, &[0_usize, 1, 2]).unwrap());
    }

    #[test]
    fn node_subset_connected_rejects_out_of_bounds_node() {
        let adjacency = vec![vec![1_usize], vec![0]];

        assert_eq!(
            node_subset_connected(&adjacency, &[0_usize, 2]),
            Err(SubsetConnectivityError::NodeOutOfBounds {
                node: 2,
                node_count: 2
            })
        );
    }

    #[test]
    fn node_subset_connected_rejects_out_of_bounds_neighbor() {
        let adjacency = vec![vec![2_usize], vec![0]];

        assert_eq!(
            node_subset_connected(&adjacency, &[0_usize, 1]),
            Err(SubsetConnectivityError::NeighborOutOfBounds {
                node: 0,
                neighbor: 2,
                node_count: 2
            })
        );
    }

    #[test]
    fn assignment_label_connected_accepts_contiguous_label() {
        let adjacency = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];
        let assignment = vec![0, 0, 1, 1];

        assert!(assignment_label_connected(&adjacency, &assignment, 0).unwrap());
        assert!(assignment_label_connected(&adjacency, &assignment, 1).unwrap());
        assert!(assignment_labels_connected(&adjacency, &assignment, 0..2).unwrap());
    }

    #[test]
    fn assignment_label_connected_rejects_disconnected_label() {
        let adjacency = vec![vec![1_usize], vec![0, 2], vec![1]];
        let assignment = vec![0, 1, 0];

        assert!(!assignment_label_connected(&adjacency, &assignment, 0).unwrap());
        assert!(!assignment_labels_connected(&adjacency, &assignment, 0..2).unwrap());
    }

    #[test]
    fn assignment_label_connected_treats_adjacency_as_undirected() {
        let adjacency = vec![vec![], vec![0_usize], vec![1]];
        let assignment = vec![7_usize, 7, 7];

        assert!(assignment_label_connected(&adjacency, &assignment, 7).unwrap());
    }

    #[test]
    fn assignment_label_connected_returns_false_for_missing_label() {
        let adjacency = vec![vec![1_usize], vec![0]];
        let assignment = vec![0, 0];

        assert!(!assignment_label_connected(&adjacency, &assignment, 1).unwrap());
    }

    #[test]
    fn assignment_label_connected_rejects_length_mismatch() {
        let adjacency = vec![vec![1_usize], vec![0]];
        let assignment = vec![0];

        assert_eq!(
            assignment_label_connected(&adjacency, &assignment, 0),
            Err(LabelConnectivityError::AssignmentLengthMismatch {
                adjacency_len: 2,
                assignment_len: 1
            })
        );
    }

    #[test]
    fn assignment_label_connected_rejects_out_of_bounds_neighbor() {
        let adjacency = vec![vec![2_usize], vec![0]];
        let assignment = vec![0, 0];

        assert_eq!(
            assignment_label_connected(&adjacency, &assignment, 0),
            Err(LabelConnectivityError::NeighborOutOfBounds {
                node: 0,
                neighbor: 2,
                node_count: 2
            })
        );
    }
}
