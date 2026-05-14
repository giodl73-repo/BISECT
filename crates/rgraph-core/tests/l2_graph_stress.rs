use rgraph_core::{
    articulation_points, bridges, connected_components, edge_betweenness, reachable_nodes,
    shortest_path_distance, DirectedWeightedGraph, WeightedEdge,
};

#[derive(Debug, Clone)]
struct GridGraph {
    width: usize,
    height: usize,
}

impl GridGraph {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    fn node(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

impl DirectedWeightedGraph for GridGraph {
    type EdgeId = (usize, usize);

    fn node_count(&self) -> usize {
        self.width * self.height
    }

    fn outgoing_edges(&self, source: usize) -> Vec<WeightedEdge<Self::EdgeId>> {
        let x = source % self.width;
        let y = source / self.width;
        let mut edges = Vec::new();
        if x + 1 < self.width {
            let target = self.node(x + 1, y);
            edges.push(WeightedEdge {
                id: (source, target),
                target,
                weight: 1.0,
            });
        }
        if y + 1 < self.height {
            let target = self.node(x, y + 1);
            edges.push(WeightedEdge {
                id: (source, target),
                target,
                weight: 1.0,
            });
        }
        edges
    }
}

#[test]
#[ignore = "L2 graph stress: larger grid shortest-path and reachability check"]
fn l2_grid_shortest_path_and_reachability_are_stable() {
    let graph = GridGraph::new(80, 80);
    let target = graph.node(79, 79);

    let distance = shortest_path_distance(&graph, 0, target).unwrap();
    let reachable = reachable_nodes(&graph, 0).unwrap();

    assert_eq!(distance, Some(158.0));
    assert_eq!(reachable.len(), 80 * 80);
}

#[test]
#[ignore = "L2 graph stress: Brandes centrality on moderate directed grid"]
fn l2_grid_edge_betweenness_remains_bounded() {
    let graph = GridGraph::new(18, 18);

    let centrality = edge_betweenness(&graph).unwrap();

    assert!(!centrality.is_empty());
    assert!(centrality.values().all(|v| (0.0..=1.0).contains(v)));
    assert!(centrality.values().any(|v| *v > 0.5));
}

#[test]
#[ignore = "L2 graph stress: connected components on larger disjoint grids"]
fn l2_disjoint_grid_components_are_stable() {
    let graph = GridGraph::new(80, 80);

    let components = connected_components(&graph).unwrap();

    assert_eq!(components.len(), 1);
    assert_eq!(components[0].len(), 80 * 80);
}

#[test]
#[ignore = "L2 graph stress: bridge detection on larger cyclic grid"]
fn l2_grid_bridge_detection_remains_stable() {
    let graph = GridGraph::new(40, 40);

    let bridges = bridges(&graph).unwrap();

    assert!(bridges.is_empty());
}

#[test]
#[ignore = "L2 graph stress: articulation detection on larger cyclic grid"]
fn l2_grid_articulation_detection_remains_stable() {
    let graph = GridGraph::new(40, 40);

    let articulations = articulation_points(&graph).unwrap();

    assert!(articulations.is_empty());
}
