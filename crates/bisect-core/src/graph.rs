use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("vertex_weights must be 1D (got 2D — multi-constraint mode not supported)")]
    MultiDimensionalWeights,
    #[error("empty graph: n_vertices must be > 0")]
    EmptyGraph,
    #[error("adjacency list length {0} does not match n_vertices {1}")]
    AdjacencyLengthMismatch(usize, usize),
}

/// CSR-format adjacency graph. Vertices are census tracts; edges connect
/// geographically adjacent tracts. Vertex weights are tract populations (1D).
#[derive(Debug, Clone)]
pub struct Graph {
    /// adjacency[i] = list of neighbor vertex indices
    pub adjacency: Vec<Vec<usize>>,
    /// vertex_weights[i] = population of tract i
    pub vertex_weights: Vec<i64>,
    pub n_vertices: usize,
}

impl Graph {
    pub fn new(
        adjacency: Vec<Vec<usize>>,
        vertex_weights: Vec<i64>,
    ) -> Result<Self, GraphError> {
        let n = adjacency.len();
        if n == 0 {
            return Err(GraphError::EmptyGraph);
        }
        if vertex_weights.len() != n {
            return Err(GraphError::AdjacencyLengthMismatch(vertex_weights.len(), n));
        }
        Ok(Self { adjacency, vertex_weights, n_vertices: n })
    }

    pub fn n_vertices(&self) -> usize {
        self.n_vertices
    }

    /// Number of undirected edges. Each edge (u,v) is stored once in CSR.
    pub fn n_edges(&self) -> usize {
        // sum of all neighbor lists / 2 (each edge appears in both u and v's list)
        self.adjacency.iter().map(|nbrs| nbrs.len()).sum::<usize>() / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_graph() -> Graph {
        // 0--1--2
        // |     |
        // 3--4--+
        Graph::new(
            vec![
                vec![1, 3],  // 0
                vec![0, 2],  // 1
                vec![1, 4],  // 2
                vec![0, 4],  // 3
                vec![3, 2],  // 4
            ],
            vec![1000, 1200, 900, 1100, 800],
        ).unwrap()
    }

    #[test]
    fn test_n_vertices() {
        assert_eq!(small_graph().n_vertices(), 5);
    }

    #[test]
    fn test_n_edges() {
        assert_eq!(small_graph().n_edges(), 5);
    }

    #[test]
    fn test_empty_graph_error() {
        let result = Graph::new(vec![], vec![]);
        assert!(matches!(result, Err(GraphError::EmptyGraph)));
    }

    #[test]
    fn test_adjacency_length_mismatch() {
        // 2 adjacency entries but 3 weight entries
        let result = Graph::new(vec![vec![1], vec![0]], vec![100, 200, 300]);
        assert!(matches!(result, Err(GraphError::AdjacencyLengthMismatch(3, 2))));
    }

    #[test]
    fn test_isolated_vertex() {
        // A graph with one vertex and no edges
        let g = Graph::new(vec![vec![]], vec![5000]).unwrap();
        assert_eq!(g.n_vertices(), 1);
        assert_eq!(g.n_edges(), 0);
    }

    #[test]
    fn test_clone_is_independent() {
        let g1 = small_graph();
        let mut g2 = g1.clone();
        g2.vertex_weights[0] = 9999;
        assert_eq!(g1.vertex_weights[0], 1000); // original unchanged
    }
}
