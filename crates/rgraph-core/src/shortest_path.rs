use crate::*;

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
            validate_distance::<G::EdgeId>(node, edge.target, next_cost)?;
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
