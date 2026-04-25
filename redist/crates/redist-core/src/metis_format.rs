/// METIS graph file format: generation and parsing.
///
/// Pure computation — no file I/O. The caller (PyO3 layer or redist-cli) writes
/// the content to disk and invokes gpmetis. This keeps redist-core free of I/O.
///
/// METIS format spec:
///   Line 1: <n_vertices> <n_edges> <fmt> [ncon]
///     fmt 010 = vertex weights only
///     fmt 011 = vertex weights + edge weights
///   Lines 2+: <vw> [<neighbor+1> <ew>]* (1-based vertex indices!)
///   Edge weights: positive integers (we scale meters → centimeters, min 1)
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetisFormatError {
    #[error("empty adjacency list")]
    EmptyGraph,
    #[error("vertex_weights length {0} != adjacency length {1}")]
    WeightLengthMismatch(usize, usize),
    #[error("partition file has {0} lines but expected {1} vertices")]
    PartitionLengthMismatch(usize, usize),
    #[error("invalid partition line {0:?}: {1}")]
    InvalidPartitionLine(String, String),
}

/// Generate METIS .graph file content for a 2-way bisection.
///
/// Edge weights: if provided, only keys present in the map are boosted;
/// missing edges get weight 1 (matching metis_executable.py:304 behaviour).
/// Weights are scaled to centimetres (×100) and clamped to min 1.
pub fn write_metis_graph(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: Option<&HashMap<(usize, usize), f64>>,
) -> Result<String, MetisFormatError> {
    let n = adjacency.len();
    if n == 0 {
        return Err(MetisFormatError::EmptyGraph);
    }
    if vertex_weights.len() != n {
        return Err(MetisFormatError::WeightLengthMismatch(vertex_weights.len(), n));
    }

    let n_edges = adjacency.iter().map(|nbrs| nbrs.len()).sum::<usize>() / 2;
    let has_ew = edge_weights.map(|m| !m.is_empty()).unwrap_or(false);
    let fmt = if has_ew { "011" } else { "010" };

    // Pre-allocate: ~30 chars per vertex line is a reasonable estimate
    let mut out = String::with_capacity(n * 30 + 20);

    // Header
    out.push_str(&format!("{n} {n_edges} {fmt} 1\n"));

    // Vertex lines (1-based neighbor indices)
    for i in 0..n {
        // Vertex weight (convert to positive integer)
        let vw = vertex_weights[i].max(1);
        out.push_str(&vw.to_string());

        for &j in &adjacency[i] {
            // 1-based neighbor index
            out.push(' ');
            out.push_str(&(j + 1).to_string());

            if has_ew {
                let key = (i.min(j), i.max(j));
                let ew_m = edge_weights.unwrap().get(&key).copied().unwrap_or(1.0);
                // Scale to centimetres, clamp to min 1
                let ew_cm = ((ew_m * 100.0).round() as i64).max(1);
                out.push(' ');
                out.push_str(&ew_cm.to_string());
            }
        }
        out.push('\n');
    }

    Ok(out)
}

/// Parse the gpmetis output partition file (one assignment per line, 0-based).
pub fn parse_metis_partition(
    content: &str,
    expected_n: usize,
) -> Result<Vec<usize>, MetisFormatError> {
    let assignments: Result<Vec<usize>, _> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.trim().parse::<usize>().map_err(|e| {
                MetisFormatError::InvalidPartitionLine(l.to_string(), e.to_string())
            })
        })
        .collect();

    let assignments = assignments?;
    if assignments.len() != expected_n {
        return Err(MetisFormatError::PartitionLengthMismatch(assignments.len(), expected_n));
    }
    Ok(assignments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_adj() -> Vec<Vec<usize>> {
        // 0-1-2-3 (linear)
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2]]
    }

    fn simple_weights() -> Vec<i64> {
        vec![1000, 1200, 900, 1100]
    }

    #[test]
    fn test_header_no_edge_weights() {
        let content = write_metis_graph(&simple_adj(), &simple_weights(), None).unwrap();
        let first = content.lines().next().unwrap();
        // 4 vertices, 3 edges (linear graph), fmt 010, ncon 1
        assert_eq!(first, "4 3 010 1");
    }

    #[test]
    fn test_header_with_edge_weights() {
        let mut ew = HashMap::new();
        ew.insert((0, 1), 150.0_f64);
        let content = write_metis_graph(&simple_adj(), &simple_weights(), Some(&ew)).unwrap();
        let first = content.lines().next().unwrap();
        assert_eq!(first, "4 3 011 1");
    }

    #[test]
    fn test_vertex_count() {
        let content = write_metis_graph(&simple_adj(), &simple_weights(), None).unwrap();
        // Header + 4 vertex lines
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_one_based_neighbor_indices() {
        let content = write_metis_graph(&simple_adj(), &simple_weights(), None).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Vertex 0 (line 1): "1000 2" → neighbor 1 becomes index 2 (1-based)
        assert_eq!(lines[1], "1000 2");
        // Vertex 1 (line 2): "1200 1 3" → neighbors 0,2 become 1,3
        assert_eq!(lines[2], "1200 1 3");
    }

    #[test]
    fn test_edge_weights_scaled_to_cm() {
        let mut ew = HashMap::new();
        ew.insert((0, 1), 1.5_f64); // 1.5 meters → 150 cm
        let content = write_metis_graph(&simple_adj(), &simple_weights(), Some(&ew)).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Vertex 0: "1000 2 150" (neighbor 1 at 1.5m → 150cm)
        assert_eq!(lines[1], "1000 2 150");
    }

    #[test]
    fn test_missing_edge_gets_weight_1() {
        let mut ew = HashMap::new();
        ew.insert((0, 1), 200.0_f64); // only boost edge 0-1
        // Edge 1-2 is missing → gets weight 1.0m → 100cm
        let content = write_metis_graph(&simple_adj(), &simple_weights(), Some(&ew)).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Vertex 1 line: "1200 1 20000 3 100" (edge 0-1: 200m→20000cm, edge 1-2: 1.0m→100cm)
        assert_eq!(lines[2], "1200 1 20000 3 100");
    }

    #[test]
    fn test_empty_graph_error() {
        let result = write_metis_graph(&[], &[], None);
        assert!(matches!(result, Err(MetisFormatError::EmptyGraph)));
    }

    #[test]
    fn test_weight_length_mismatch() {
        let result = write_metis_graph(&simple_adj(), &[100, 200], None);
        assert!(matches!(result, Err(MetisFormatError::WeightLengthMismatch(2, 4))));
    }

    #[test]
    fn test_parse_partition_valid() {
        let content = "0\n1\n0\n1\n";
        let parts = parse_metis_partition(content, 4).unwrap();
        assert_eq!(parts, vec![0, 1, 0, 1]);
    }

    #[test]
    fn test_parse_partition_length_mismatch() {
        let content = "0\n1\n0\n";
        let result = parse_metis_partition(content, 4);
        assert!(matches!(result, Err(MetisFormatError::PartitionLengthMismatch(3, 4))));
    }

    #[test]
    fn test_parse_partition_invalid_line() {
        let content = "0\nbad\n1\n1\n";
        let result = parse_metis_partition(content, 4);
        assert!(matches!(result, Err(MetisFormatError::InvalidPartitionLine(_, _))));
    }

    #[test]
    fn test_roundtrip_vertex_count() {
        // Generate graph file and verify it has n+1 lines (header + n vertices)
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let vw = vec![100i64, 200, 150];
        let content = write_metis_graph(&adj, &vw, None).unwrap();
        assert_eq!(content.lines().count(), 4); // header + 3 vertices
    }
}
