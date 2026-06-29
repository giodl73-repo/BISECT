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
fn shortest_path_distance_overflow_is_rejected() {
    let mut graph = TinyGraph::new(3);
    graph.add_edge(1, 0, 1, f64::MAX);
    graph.add_edge(2, 1, 2, f64::MAX);

    match single_source_shortest_paths(&graph, 0) {
        Err(GraphError::NonFiniteDistance {
            from,
            target,
            distance,
        }) => {
            assert_eq!((from, target), (1, 2));
            assert!(distance.is_infinite());
        }
        other => panic!("expected distance overflow error, got {other:?}"),
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
fn undirected_boundary_metrics_scores_selected_set() {
    let adjacency = vec![
        vec![1_usize, 2],
        vec![0, 2, 3],
        vec![0, 1, 3],
        vec![1, 2, 4],
        vec![3],
    ];
    let selected = vec![true, true, true, false, false];

    assert_eq!(
        undirected_boundary_metrics(&adjacency, &selected).unwrap(),
        BoundaryMetrics {
            selected_count: 3,
            complement_count: 2,
            selected_internal_edges: 3,
            complement_internal_edges: 1,
            boundary_edges: 2,
            selected_degree: 8,
            complement_degree: 4,
            total_edges: 6,
            conductance: 0.5,
        }
    );
}

#[test]
fn undirected_boundary_metrics_counts_asymmetric_edges_once() {
    let adjacency = vec![vec![1_usize, 1], vec![], vec![1]];
    let selected = vec![true, false, false];

    assert_eq!(
        undirected_boundary_metrics(&adjacency, &selected).unwrap(),
        BoundaryMetrics {
            selected_count: 1,
            complement_count: 2,
            selected_internal_edges: 0,
            complement_internal_edges: 1,
            boundary_edges: 1,
            selected_degree: 1,
            complement_degree: 3,
            total_edges: 2,
            conductance: 1.0,
        }
    );
}

#[test]
fn undirected_boundary_metrics_rejects_length_mismatch() {
    let adjacency = vec![vec![1_usize], vec![0]];
    let selected = vec![true];

    assert_eq!(
        undirected_boundary_metrics(&adjacency, &selected),
        Err(EdgeCutError::AssignmentLengthMismatch {
            adjacency_len: 2,
            assignment_len: 1
        })
    );
}

#[test]
fn undirected_boundary_metrics_rejects_out_of_bounds_neighbor() {
    let adjacency = vec![vec![2_usize], vec![0]];
    let selected = vec![true, false];

    assert_eq!(
        undirected_boundary_metrics(&adjacency, &selected),
        Err(EdgeCutError::NeighborOutOfBounds {
            node: 0,
            neighbor: 2,
            node_count: 2
        })
    );
}

#[test]
fn shortest_connector_path_recovers_missing_bridge_node() {
    let adjacency = vec![
        vec![1_usize],
        vec![0, 2],
        vec![1, 3],
        vec![2],
        vec![5],
        vec![4],
    ];

    assert_eq!(
        shortest_connector_path(&adjacency, &[0], &[3]).unwrap(),
        Some(ConnectorPath {
            source: 0,
            target: 3,
            nodes: vec![0, 1, 2, 3],
            bridge_nodes: vec![1, 2],
            hop_count: 3,
        })
    );
}

#[test]
fn shortest_connector_path_uses_deterministic_tie_breaking() {
    let adjacency = vec![
        vec![2_usize, 1],
        vec![0, 4],
        vec![0, 4],
        vec![4],
        vec![1, 2, 3],
    ];

    assert_eq!(
        shortest_connector_path(&adjacency, &[0, 3], &[4]).unwrap(),
        Some(ConnectorPath {
            source: 3,
            target: 4,
            nodes: vec![3, 4],
            bridge_nodes: vec![],
            hop_count: 1,
        })
    );
}

#[test]
fn shortest_connector_path_returns_none_for_disconnected_sets() {
    let adjacency = vec![vec![1_usize], vec![0], vec![3], vec![2]];

    assert_eq!(
        shortest_connector_path(&adjacency, &[0], &[3]).unwrap(),
        None
    );
}

#[test]
fn shortest_connector_path_rejects_empty_sources() {
    let adjacency = vec![vec![1_usize], vec![0]];

    assert_eq!(
        shortest_connector_path(&adjacency, &[], &[1]),
        Err(ConnectorPathError::EmptySources)
    );
}

#[test]
fn shortest_connector_path_rejects_out_of_bounds_target() {
    let adjacency = vec![vec![1_usize], vec![0]];

    assert_eq!(
        shortest_connector_path(&adjacency, &[0], &[2]),
        Err(ConnectorPathError::NodeOutOfBounds {
            kind: "target",
            node: 2,
            node_count: 2,
        })
    );
}

#[test]
fn shortest_connector_path_rejects_out_of_bounds_neighbor() {
    let adjacency = vec![vec![2_usize], vec![0]];

    assert_eq!(
        shortest_connector_path(&adjacency, &[0], &[1]),
        Err(ConnectorPathError::NeighborOutOfBounds {
            node: 0,
            neighbor: 2,
            node_count: 2,
        })
    );
}

#[test]
fn undirected_cluster_summaries_score_neighborhoods() {
    let adjacency = vec![
        vec![1_usize, 2],
        vec![0, 2, 3],
        vec![0, 1],
        vec![1, 4],
        vec![3],
    ];
    let clusters = vec![vec![0_usize, 1, 2], vec![3, 4]];

    assert_eq!(
        undirected_cluster_summaries(&adjacency, &clusters).unwrap(),
        vec![
            ClusterSummary {
                cluster_index: 0,
                nodes: vec![0, 1, 2],
                representative_node: 0,
                internal_edges: 3,
                boundary_edges: 1,
                volume: 7,
                conductance: 1.0 / 7.0,
            },
            ClusterSummary {
                cluster_index: 1,
                nodes: vec![3, 4],
                representative_node: 3,
                internal_edges: 1,
                boundary_edges: 1,
                volume: 3,
                conductance: 1.0 / 3.0,
            },
        ]
    );
}

#[test]
fn undirected_cluster_summaries_count_edges_to_unclustered_nodes() {
    let adjacency = vec![vec![1_usize], vec![0, 2], vec![1]];
    let clusters = vec![vec![0_usize, 1]];

    assert_eq!(
        undirected_cluster_summaries(&adjacency, &clusters).unwrap(),
        vec![ClusterSummary {
            cluster_index: 0,
            nodes: vec![0, 1],
            representative_node: 0,
            internal_edges: 1,
            boundary_edges: 1,
            volume: 3,
            conductance: 1.0 / 3.0,
        }]
    );
}

#[test]
fn undirected_cluster_summaries_reject_empty_cluster() {
    let adjacency = vec![vec![1_usize], vec![0]];
    let clusters = vec![vec![0_usize], vec![]];

    assert_eq!(
        undirected_cluster_summaries(&adjacency, &clusters),
        Err(ClusterSummaryError::EmptyCluster { cluster_index: 1 })
    );
}

#[test]
fn undirected_cluster_summaries_reject_duplicate_cluster_node() {
    let adjacency = vec![vec![1_usize], vec![0]];
    let clusters = vec![vec![0_usize, 1], vec![1]];

    assert_eq!(
        undirected_cluster_summaries(&adjacency, &clusters),
        Err(ClusterSummaryError::DuplicateClusterNode {
            node: 1,
            first_cluster: 0,
            second_cluster: 1,
        })
    );
}

#[test]
fn undirected_cluster_summaries_reject_out_of_bounds_neighbor() {
    let adjacency = vec![vec![2_usize], vec![0]];
    let clusters = vec![vec![0_usize]];

    assert_eq!(
        undirected_cluster_summaries(&adjacency, &clusters),
        Err(ClusterSummaryError::NeighborOutOfBounds {
            node: 0,
            neighbor: 2,
            node_count: 2,
        })
    );
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
