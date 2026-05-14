use rgraph_core::{
    bridges, bridges_with_filter, connected_components, connected_components_in_nodes_with_filter,
    edge_betweenness, reachable_nodes_with_filter, shortest_path_distance,
    shortest_path_distance_with_filter, single_source_shortest_paths, Bridge,
    DirectedWeightedGraph, GraphError, WeightedEdge,
};

#[derive(Debug, Clone)]
struct TestGraph {
    edges: Vec<Vec<WeightedEdge<u32>>>,
}

impl TestGraph {
    fn new(node_count: usize) -> Self {
        Self {
            edges: vec![Vec::new(); node_count],
        }
    }

    fn edge(mut self, id: u32, from: usize, to: usize, weight: f64) -> Self {
        self.edges[from].push(WeightedEdge {
            id,
            target: to,
            weight,
        });
        self
    }
}

impl DirectedWeightedGraph for TestGraph {
    type EdgeId = u32;

    fn node_count(&self) -> usize {
        self.edges.len()
    }

    fn outgoing_edges(&self, source: usize) -> Vec<WeightedEdge<Self::EdgeId>> {
        self.edges[source].clone()
    }
}

#[test]
fn l1_weighted_shortest_paths_prefer_lower_cost_indirect_route() {
    let graph = TestGraph::new(4)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 3, 1.0)
        .edge(3, 0, 2, 0.5)
        .edge(4, 2, 3, 10.0)
        .edge(5, 0, 3, 5.0);

    let tree = single_source_shortest_paths(&graph, 0).unwrap();

    assert_eq!(shortest_path_distance(&graph, 0, 3).unwrap(), Some(2.0));
    assert_eq!(tree.distance_to(3), Some(2.0));
    assert_eq!(tree.predecessors[3][0].edge_id, 2);
}

#[test]
fn l1_filter_changes_path_and_reachability() {
    let graph = TestGraph::new(5)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 4, 1.0)
        .edge(3, 0, 2, 1.0)
        .edge(4, 2, 3, 1.0)
        .edge(5, 3, 4, 1.0);

    assert_eq!(shortest_path_distance(&graph, 0, 4).unwrap(), Some(2.0));
    assert_eq!(
        shortest_path_distance_with_filter(&graph, 0, 4, |edge| edge != 2).unwrap(),
        Some(3.0)
    );
    assert_eq!(
        reachable_nodes_with_filter(&graph, 0, |edge| !matches!(edge, 2 | 5)).unwrap(),
        vec![0, 1, 2, 3]
    );
}

#[test]
fn l1_centrality_identifies_bridge_edge() {
    let graph = TestGraph::new(5)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 2, 3, 1.0)
        .edge(4, 3, 4, 1.0)
        .edge(5, 0, 4, 10.0);

    let centrality = edge_betweenness(&graph).unwrap();

    assert!(centrality[&2] >= centrality[&1]);
    assert!(centrality[&3] >= centrality[&4]);
    assert!(centrality[&2] > centrality.get(&5).copied().unwrap_or(0.0));
}

#[test]
fn l1_invalid_target_from_adapter_is_reported() {
    let graph = TestGraph::new(2).edge(9, 0, 3, 1.0);

    let err = shortest_path_distance(&graph, 0, 1).unwrap_err();

    assert_eq!(
        err,
        GraphError::NodeOutOfBounds {
            node: 3,
            node_count: 2
        }
    );
}

#[test]
fn l1_connected_components_support_subset_and_filter() {
    let graph = TestGraph::new(7)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 3, 4, 1.0)
        .edge(4, 4, 5, 1.0)
        .edge(5, 5, 6, 1.0);

    assert_eq!(
        connected_components(&graph).unwrap(),
        vec![vec![0, 1, 2], vec![3, 4, 5, 6]]
    );
    assert_eq!(
        connected_components_in_nodes_with_filter(&graph, &[0, 1, 2, 4, 5, 6], |edge| edge != 5)
            .unwrap(),
        vec![vec![0, 1, 2], vec![4, 5], vec![6]]
    );
}

#[test]
fn l1_bridges_ignore_cycles_and_respect_filters() {
    let graph = TestGraph::new(6)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 2, 0, 1.0)
        .edge(4, 2, 3, 1.0)
        .edge(5, 3, 4, 1.0)
        .edge(6, 4, 5, 1.0)
        .edge(7, 5, 3, 1.0);

    assert_eq!(
        bridges(&graph).unwrap(),
        vec![Bridge {
            source: 2,
            target: 3,
            edge_id: 4
        }]
    );
    assert_eq!(
        bridges_with_filter(&graph, |edge| edge != 7).unwrap(),
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
            },
            Bridge {
                source: 4,
                target: 5,
                edge_id: 6
            }
        ]
    );
}
