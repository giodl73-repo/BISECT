//! Independent regression for prepared weak-connectivity queries and scratch history.
use rgraph_core::{node_subset_connected, PreparedSubsetConnectivity};

// Scan the original arcs in either direction rather than using the production
// adjacency normalizer or traversal implementation.
fn independent_connected(graph: &[Vec<usize>], nodes: &[usize]) -> bool {
    let Some(&start) = nodes.first() else {
        return true;
    };
    let mut reached = vec![start];
    let mut cursor = 0;
    while cursor < reached.len() {
        let from = reached[cursor];
        cursor += 1;
        for &to in nodes {
            if !reached.contains(&to) && (graph[from].contains(&to) || graph[to].contains(&from)) {
                reached.push(to);
            }
        }
    }
    nodes.iter().all(|node| reached.contains(node))
}

#[test]
fn exhaustive_directed_graphs_preserve_connectivity_and_query_history() {
    for n in 0usize..=4 {
        let arcs: Vec<_> = (0..n)
            .flat_map(|from| {
                (0..n)
                    .filter(move |&to| to != from)
                    .map(move |to| (from, to))
            })
            .collect();
        for graph_bits in 0usize..(1 << arcs.len()) {
            let mut graph = vec![vec![]; n];
            for (bit, &(from, to)) in arcs.iter().enumerate() {
                if graph_bits & (1 << bit) != 0 {
                    graph[from].push(to);
                }
            }
            let mut prepared = PreparedSubsetConnectivity::new(&graph).unwrap();
            for subset_bits in 0usize..(1 << n) {
                let nodes: Vec<_> = (0..n)
                    .filter(|node| subset_bits & (1 << node) != 0)
                    .collect();
                let expected = independent_connected(&graph, &nodes);
                let reversed: Vec<_> = nodes.iter().copied().rev().collect();
                let duplicated: Vec<_> = reversed.iter().chain(&nodes).copied().collect();
                for query in [&nodes, &reversed, &duplicated] {
                    assert_eq!(node_subset_connected(&graph, query).unwrap(), expected);
                    assert_eq!(prepared.node_subset_connected(query).unwrap(), expected);
                    // Leave partial membership behind on an error, then issue
                    // an empty query and replay the valid subset.
                    let mut invalid = query.clone();
                    invalid.push(n);
                    assert!(prepared.node_subset_connected(&invalid).is_err());
                    assert!(prepared.node_subset_connected::<usize>(&[]).unwrap());
                    assert_eq!(prepared.node_subset_connected(query).unwrap(), expected);
                }
            }
        }
    }
}

#[test]
fn prepared_topology_is_an_owned_snapshot_with_weak_connectivity() {
    let mut graph = vec![vec![1, 1, 0], vec![], vec![1]];
    let original = graph.clone();
    let mut prepared = PreparedSubsetConnectivity::new(&graph).unwrap();
    assert!(prepared.node_subset_connected(&[2usize, 1, 0, 2]).unwrap());
    assert!(!prepared.node_subset_connected(&[0usize, 2]).unwrap());
    graph[0].push(2);
    assert!(node_subset_connected(&graph, &[0usize, 2]).unwrap());
    assert!(!prepared.node_subset_connected(&[0usize, 2]).unwrap());
    assert_eq!(
        prepared.node_subset_connected(&[0usize, 2]).unwrap(),
        independent_connected(&original, &[0, 2])
    );
    assert!(PreparedSubsetConnectivity::new(&graph)
        .unwrap()
        .node_subset_connected(&[0usize, 2])
        .unwrap());
}

#[test]
fn invalid_constructor_and_typed_query_boundaries_remain_explicit() {
    assert!(PreparedSubsetConnectivity::new(&[vec![1usize]]).is_err());
    let graph = vec![vec![1u32], vec![]];
    let mut prepared = PreparedSubsetConnectivity::new(&graph).unwrap();
    assert!(prepared.node_subset_connected(&[1u32, 0, 1]).unwrap());
    assert!(prepared.node_subset_connected(&[0u32, u32::MAX]).is_err());
    assert!(prepared.node_subset_connected::<u32>(&[]).unwrap());
    assert!(prepared.node_subset_connected(&[0u32, 1]).unwrap());
}
