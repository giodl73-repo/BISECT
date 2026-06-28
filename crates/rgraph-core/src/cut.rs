use crate::*;

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

pub fn undirected_boundary_metrics<I>(
    adjacency: &[Vec<I>],
    selected: &[bool],
) -> Result<BoundaryMetrics, EdgeCutError>
where
    I: NodeIndex,
{
    if adjacency.len() != selected.len() {
        return Err(EdgeCutError::AssignmentLengthMismatch {
            adjacency_len: adjacency.len(),
            assignment_len: selected.len(),
        });
    }

    let selected_count = selected.iter().filter(|&&is_selected| is_selected).count();
    let complement_count = selected.len() - selected_count;
    let mut seen_edges = HashSet::new();
    let mut selected_internal_edges = 0usize;
    let mut complement_internal_edges = 0usize;
    let mut boundary_edges = 0usize;

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
            if node == neighbor {
                continue;
            }
            if !seen_edges.insert(ordered_pair(node, neighbor)) {
                continue;
            }

            match (selected[node], selected[neighbor]) {
                (true, true) => selected_internal_edges += 1,
                (false, false) => complement_internal_edges += 1,
                _ => boundary_edges += 1,
            }
        }
    }

    let selected_degree = selected_internal_edges * 2 + boundary_edges;
    let complement_degree = complement_internal_edges * 2 + boundary_edges;
    let denominator = selected_degree.min(complement_degree);
    let conductance = if denominator == 0 {
        0.0
    } else {
        boundary_edges as f64 / denominator as f64
    };

    Ok(BoundaryMetrics {
        selected_count,
        complement_count,
        selected_internal_edges,
        complement_internal_edges,
        boundary_edges,
        selected_degree,
        complement_degree,
        total_edges: selected_internal_edges + complement_internal_edges + boundary_edges,
        conductance,
    })
}

pub fn shortest_connector_path<I>(
    adjacency: &[Vec<I>],
    sources: &[usize],
    targets: &[usize],
) -> Result<Option<ConnectorPath>, ConnectorPathError>
where
    I: NodeIndex,
{
    if sources.is_empty() {
        return Err(ConnectorPathError::EmptySources);
    }
    if targets.is_empty() {
        return Err(ConnectorPathError::EmptyTargets);
    }

    let node_count = adjacency.len();
    let mut sorted_sources = sources.to_vec();
    sorted_sources.sort_unstable();
    sorted_sources.dedup();
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_unstable();
    sorted_targets.dedup();

    for &source in &sorted_sources {
        if source >= node_count {
            return Err(ConnectorPathError::NodeOutOfBounds {
                kind: "source",
                node: source,
                node_count,
            });
        }
    }
    for &target in &sorted_targets {
        if target >= node_count {
            return Err(ConnectorPathError::NodeOutOfBounds {
                kind: "target",
                node: target,
                node_count,
            });
        }
    }

    let undirected = undirected_index_adjacency_for_connectors(adjacency)?;
    let target_set: HashSet<usize> = sorted_targets.iter().copied().collect();
    let mut predecessor = vec![None; node_count];
    let mut source_for = vec![None; node_count];
    let mut seen = vec![false; node_count];
    let mut queue = VecDeque::new();

    for &source in &sorted_sources {
        seen[source] = true;
        source_for[source] = Some(source);
        queue.push_back(source);
    }

    while let Some(node) = queue.pop_front() {
        if target_set.contains(&node) {
            let source = source_for[node].expect("visited connector node has source");
            let mut nodes = Vec::new();
            let mut cursor = node;
            nodes.push(cursor);
            while cursor != source {
                cursor = predecessor[cursor].expect("connector path has predecessor");
                nodes.push(cursor);
            }
            nodes.reverse();
            let bridge_nodes = if nodes.len() <= 2 {
                Vec::new()
            } else {
                nodes[1..nodes.len() - 1].to_vec()
            };
            return Ok(Some(ConnectorPath {
                source,
                target: node,
                hop_count: nodes.len().saturating_sub(1),
                nodes,
                bridge_nodes,
            }));
        }

        for &neighbor in &undirected[node] {
            if !seen[neighbor] {
                seen[neighbor] = true;
                predecessor[neighbor] = Some(node);
                source_for[neighbor] = source_for[node];
                queue.push_back(neighbor);
            }
        }
    }

    Ok(None)
}

pub fn undirected_cluster_summaries<I, D>(
    adjacency: &[Vec<I>],
    clusters: &[Vec<D>],
) -> Result<Vec<ClusterSummary>, ClusterSummaryError>
where
    I: NodeIndex,
    D: NodeIndex,
{
    let node_count = adjacency.len();
    let mut membership = vec![None; node_count];
    let mut normalized_clusters = Vec::with_capacity(clusters.len());

    for (cluster_index, cluster) in clusters.iter().enumerate() {
        if cluster.is_empty() {
            return Err(ClusterSummaryError::EmptyCluster { cluster_index });
        }
        let mut nodes = Vec::with_capacity(cluster.len());
        for &node in cluster {
            let Some(node) = node.to_usize() else {
                return Err(ClusterSummaryError::NodeOutOfBounds {
                    cluster_index,
                    node: usize::MAX,
                    node_count,
                });
            };
            if node >= node_count {
                return Err(ClusterSummaryError::NodeOutOfBounds {
                    cluster_index,
                    node,
                    node_count,
                });
            }
            if let Some(first_cluster) = membership[node] {
                return Err(ClusterSummaryError::DuplicateClusterNode {
                    node,
                    first_cluster,
                    second_cluster: cluster_index,
                });
            }
            membership[node] = Some(cluster_index);
            nodes.push(node);
        }
        nodes.sort_unstable();
        normalized_clusters.push(nodes);
    }

    let mut internal_edges = vec![0usize; clusters.len()];
    let mut boundary_edges = vec![0usize; clusters.len()];
    let mut internal_degree = vec![vec![0usize; node_count]; clusters.len()];
    let mut seen_edges = HashSet::new();

    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            let Some(neighbor) = neighbor.to_usize() else {
                return Err(ClusterSummaryError::NeighborOutOfBounds {
                    node,
                    neighbor: usize::MAX,
                    node_count,
                });
            };
            if neighbor >= node_count {
                return Err(ClusterSummaryError::NeighborOutOfBounds {
                    node,
                    neighbor,
                    node_count,
                });
            }
            if node == neighbor || !seen_edges.insert(ordered_pair(node, neighbor)) {
                continue;
            }

            match (membership[node], membership[neighbor]) {
                (Some(left), Some(right)) if left == right => {
                    internal_edges[left] += 1;
                    internal_degree[left][node] += 1;
                    internal_degree[left][neighbor] += 1;
                }
                (Some(left), Some(right)) => {
                    boundary_edges[left] += 1;
                    boundary_edges[right] += 1;
                }
                (Some(cluster), None) | (None, Some(cluster)) => boundary_edges[cluster] += 1,
                (None, None) => {}
            }
        }
    }

    normalized_clusters
        .into_iter()
        .enumerate()
        .map(|(cluster_index, nodes)| {
            let representative_node = nodes
                .iter()
                .copied()
                .max_by(|&left, &right| {
                    internal_degree[cluster_index][left]
                        .cmp(&internal_degree[cluster_index][right])
                        .then_with(|| right.cmp(&left))
                })
                .expect("empty clusters were rejected");
            let volume = internal_edges[cluster_index] * 2 + boundary_edges[cluster_index];
            let conductance = if volume == 0 {
                0.0
            } else {
                boundary_edges[cluster_index] as f64 / volume as f64
            };
            Ok(ClusterSummary {
                cluster_index,
                nodes,
                representative_node,
                internal_edges: internal_edges[cluster_index],
                boundary_edges: boundary_edges[cluster_index],
                volume,
                conductance,
            })
        })
        .collect()
}
