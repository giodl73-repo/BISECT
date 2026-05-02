//! SubGraph: a region of the census-tract graph with CSR-formatted data
//! ready for direct METIS FFI calls.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("empty region: cannot build SubGraph from 0 vertices")]
    EmptyRegion,
}

/// A subgraph (contiguous region) ready for METIS partitioning.
///
/// Vertex and edge data are in the METIS CSR format with local (0-based)
/// vertex indices. `global_ids[local]` gives the global tract index.
///
/// Edge weights are scaled: `(meters * EDGE_SCALE_CM).round().max(1)` as i32.
pub struct SubGraph {
    /// Global tract index for each local vertex.
    pub global_ids: Vec<usize>,
    /// CSR row pointers (len = n + 1).
    pub xadj: Vec<i32>,
    /// CSR column indices (local, len = 2 * n_edges).
    pub adjncy: Vec<i32>,
    /// Vertex weights = population (len = n, each ≥ 1).
    pub vwgt: Vec<i32>,
    /// Edge weights in same order as adjncy, or None if unweighted.
    pub adjwgt: Option<Vec<i32>>,
}

/// Scale factor: TIGER boundary lengths are in metres; METIS needs integers.
/// 1 m → 100 cm (centimetre resolution, same as existing bisection_runner).
const EDGE_SCALE_CM: f64 = 100.0;

impl SubGraph {
    /// Build a SubGraph from a global graph view restricted to `vertices`.
    ///
    /// `adjacency[g]` is the list of global neighbours of tract `g`.
    /// `vertex_weights[g]` is the population of tract `g` (i64 ≥ 0).
    /// `edge_weights` maps canonical edge `(min, max)` → boundary length in metres.
    ///   Pass an empty map for unweighted partitioning.
    pub fn build(
        vertices: &[usize],
        adjacency: &[Vec<usize>],
        vertex_weights: &[i64],
        edge_weights: &HashMap<(usize, usize), f64>,
    ) -> Result<Self, GraphError> {
        if vertices.is_empty() {
            return Err(GraphError::EmptyRegion);
        }

        // Sort for deterministic local ordering.
        let mut sorted: Vec<usize> = vertices.to_vec();
        sorted.sort_unstable();

        // Build global→local index map.
        let mut g2l: HashMap<usize, i32> = HashMap::with_capacity(sorted.len());
        for (local, &global) in sorted.iter().enumerate() {
            g2l.insert(global, local as i32);
        }

        let n = sorted.len();
        let has_ew = !edge_weights.is_empty();

        let mut xadj: Vec<i32> = Vec::with_capacity(n + 1);
        let mut adjncy: Vec<i32> = Vec::new();
        let mut adjwgt_vals: Vec<i32> = Vec::new();

        xadj.push(0);
        for &g in &sorted {
            for &nb in &adjacency[g] {
                if let Some(&local_nb) = g2l.get(&nb) {
                    adjncy.push(local_nb);
                    if has_ew {
                        let key = (g.min(nb), g.max(nb));
                        let w = edge_weights.get(&key).copied().unwrap_or(1.0);
                        adjwgt_vals.push((w * EDGE_SCALE_CM).round().max(1.0) as i32);
                    }
                }
            }
            xadj.push(adjncy.len() as i32);
        }

        let vwgt: Vec<i32> = sorted.iter()
            .map(|&g| (vertex_weights[g] as i32).max(1))
            .collect();

        let adjwgt = if has_ew { Some(adjwgt_vals) } else { None };

        Ok(SubGraph { global_ids: sorted, xadj, adjncy, vwgt, adjwgt })
    }

    pub fn n_vertices(&self) -> usize { self.global_ids.len() }
    pub fn n_edges(&self) -> usize { self.adjncy.len() / 2 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> (Vec<Vec<usize>>, Vec<i64>, HashMap<(usize, usize), f64>) {
        // 3-vertex triangle: 0-1, 1-2, 0-2
        let adj = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
        let vw = vec![100i64, 200, 150];
        let ew: HashMap<_, _> = [((0,1), 1000.0), ((1,2), 2000.0), ((0,2), 1500.0)]
            .iter().cloned().collect();
        (adj, vw, ew)
    }

    #[test]
    fn test_full_graph() {
        let (adj, vw, ew) = triangle();
        let sg = SubGraph::build(&[0,1,2], &adj, &vw, &ew).unwrap();
        assert_eq!(sg.n_vertices(), 3);
        assert_eq!(sg.vwgt, vec![100, 200, 150]);
        assert!(sg.adjwgt.is_some());
        // Each of 3 edges appears twice (both directions)
        assert_eq!(sg.adjncy.len(), 6);
    }

    #[test]
    fn test_subgraph_two_vertices() {
        let (adj, vw, ew) = triangle();
        // Only vertices 0 and 1; edge 0-1 stays, edges to 2 drop
        let sg = SubGraph::build(&[0, 1], &adj, &vw, &ew).unwrap();
        assert_eq!(sg.n_vertices(), 2);
        assert_eq!(sg.global_ids, vec![0, 1]);
        // 1 edge × 2 directions = 2 adjncy entries
        assert_eq!(sg.adjncy.len(), 2);
    }

    #[test]
    fn test_unweighted() {
        let adj = vec![vec![1], vec![0]];
        let vw = vec![100i64, 200];
        let ew: HashMap<(usize, usize), f64> = HashMap::new();
        let sg = SubGraph::build(&[0, 1], &adj, &vw, &ew).unwrap();
        assert!(sg.adjwgt.is_none());
    }

    #[test]
    fn test_empty_region_error() {
        let adj: Vec<Vec<usize>> = vec![];
        let vw: Vec<i64> = vec![];
        let ew = HashMap::new();
        assert!(SubGraph::build(&[], &adj, &vw, &ew).is_err());
    }
}
