use crate::*;

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

pub(crate) fn undirected_index_adjacency_for_labels<I>(
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

pub(crate) fn undirected_index_adjacency_for_subset<I>(
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

pub(crate) fn undirected_index_adjacency_for_connectors<I>(
    adjacency: &[Vec<I>],
) -> Result<Vec<Vec<usize>>, ConnectorPathError>
where
    I: NodeIndex,
{
    let node_count = adjacency.len();
    let mut undirected = vec![Vec::new(); node_count];
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            let Some(neighbor) = neighbor.to_usize() else {
                return Err(ConnectorPathError::NeighborOutOfBounds {
                    node,
                    neighbor: usize::MAX,
                    node_count,
                });
            };
            if neighbor >= node_count {
                return Err(ConnectorPathError::NeighborOutOfBounds {
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

pub(crate) fn validate_assignment_adjacency<I, D>(
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
