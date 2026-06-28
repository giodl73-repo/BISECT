use crate::*;

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

pub(crate) fn undirected_adjacency<G, F>(
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

pub(crate) fn articulation_dfs(
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

pub(crate) fn bridge_dfs(
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

pub(crate) fn ordered_pair(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

pub(crate) fn validate_node<E>(node_count: usize, node: usize) -> Result<(), GraphError<E>> {
    if node >= node_count {
        return Err(GraphError::NodeOutOfBounds { node, node_count });
    }
    Ok(())
}

pub(crate) fn validate_weight<E>(
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

pub(crate) fn validate_path_count<E>(node: usize, count: f64) -> Result<(), GraphError<E>> {
    if !count.is_finite() {
        return Err(GraphError::NonFinitePathCount { node, count });
    }
    Ok(())
}

pub(crate) fn validate_distance<E>(
    from: usize,
    target: usize,
    distance: f64,
) -> Result<(), GraphError<E>> {
    if !distance.is_finite() {
        return Err(GraphError::NonFiniteDistance {
            from,
            target,
            distance,
        });
    }
    Ok(())
}
