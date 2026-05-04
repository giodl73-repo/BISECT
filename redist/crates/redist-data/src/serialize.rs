/// Adjacency graph binary serialization (.adj.bin format).
///
/// Fast Rust-to-Rust format. Not pickle — Python uses the companion
/// conversion script to produce .pkl for backward compat.
///
/// Format v2 (little-endian):
///   [0..4]   magic bytes: b"RADJ"
///   [4..8]   format version: u32 = 2
///   [8..12]  n_vertices: u32
///   [12..16] n_edges: u32
///   [16..]   vertex_weights section:
///              for each vertex i (0..n_vertices): weight: i64
///   then adjacency section:
///              for each vertex i (0..n_vertices):
///                n_neighbors: u32
///                neighbor_0: u32, ..., neighbor_{n-1}: u32
///   then edge_weights section:
///              n_weights: u32
///              for each weight:
///                u: u32, v: u32, weight: f64  (u < v always)
///
/// Format v3 (superset of v2 — adds CompactBisect geometry):
///   Same as v2, then:
///   has_geometry: u32 (0 = no geometry, 1 = geometry follows)
///   if has_geometry == 1:
///     vertex_ext_perimeters section:
///       for each vertex i (0..n_vertices): ext_perimeter: f64
///     vertex_areas section:
///       for each vertex i (0..n_vertices): area: f64
///   v2 files are read correctly (has_geometry defaults to 0).
use std::io::{self, Read, Write};
use thiserror::Error;
use crate::adjacency::AdjacencyGraph;

const MAGIC: &[u8; 4] = b"RADJ";
const FORMAT_VERSION: u32 = 3;
const FORMAT_VERSION_V2: u32 = 2;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid magic bytes: expected RADJ")]
    InvalidMagic,
    #[error("unsupported format version: {0} (expected {FORMAT_VERSION})")]
    UnsupportedVersion(u32),
    #[error("data truncated at {context}")]
    Truncated { context: &'static str },
    #[error("n_edges header ({header}) does not match actual edge weight count ({actual})")]
    EdgeCountMismatch { header: usize, actual: usize },
}

/// Serialize an adjacency graph to binary bytes (format v2 with vertex_weights).
pub fn serialize_adjacency(graph: &AdjacencyGraph) -> Vec<u8> {
    let mut buf = Vec::with_capacity(
        16 + graph.n_vertices * 16 + graph.n_edges * 16
    );

    // Header
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    buf.extend_from_slice(&(graph.n_vertices as u32).to_le_bytes());
    buf.extend_from_slice(&(graph.n_edges as u32).to_le_bytes());

    // Vertex weights section (v2 addition)
    for &w in &graph.vertex_weights {
        buf.extend_from_slice(&w.to_le_bytes());
    }

    // Adjacency section
    for nbrs in &graph.adjacency {
        buf.extend_from_slice(&(nbrs.len() as u32).to_le_bytes());
        for &n in nbrs {
            buf.extend_from_slice(&(n as u32).to_le_bytes());
        }
    }

    // Edge weights section
    // Validate canonical key order before writing — non-canonical keys (u>v) would
    // deserialize as-is and silently produce duplicate or wrong edges downstream.
    for &(u, v) in graph.edge_weights.keys() {
        assert!(u < v,
            "edge_weights key ({u},{v}) is not canonical (u must be < v); \
             build_adjacency_graph always produces canonical keys");
    }
    buf.extend_from_slice(&(graph.edge_weights.len() as u32).to_le_bytes());
    // Sort for deterministic output
    let mut weights: Vec<_> = graph.edge_weights.iter().collect();
    weights.sort_by_key(|&(&(u, v), _)| (u, v));
    for (&(u, v), &w) in &weights {
        buf.extend_from_slice(&(u as u32).to_le_bytes());
        buf.extend_from_slice(&(v as u32).to_le_bytes());
        buf.extend_from_slice(&w.to_le_bytes());
    }

    // v3: CompactBisect geometry (has_geometry flag + ext_perimeters + areas)
    let has_geometry = !graph.vertex_ext_perimeters.is_empty()
                    && !graph.vertex_areas.is_empty();
    buf.extend_from_slice(&(has_geometry as u32).to_le_bytes());
    if has_geometry {
        for &p in &graph.vertex_ext_perimeters {
            buf.extend_from_slice(&p.to_le_bytes());
        }
        for &a in &graph.vertex_areas {
            buf.extend_from_slice(&a.to_le_bytes());
        }
    }

    buf
}

/// Deserialize an adjacency graph from binary bytes.
pub fn deserialize_adjacency(data: &[u8]) -> Result<AdjacencyGraph, SerializeError> {
    let mut pos = 0usize;

    macro_rules! read_u32 {
        ($ctx:expr) => {{
            if pos + 4 > data.len() {
                return Err(SerializeError::Truncated { context: $ctx });
            }
            let v = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
            pos += 4;
            v
        }};
    }

    macro_rules! read_f64 {
        ($ctx:expr) => {{
            if pos + 8 > data.len() {
                return Err(SerializeError::Truncated { context: $ctx });
            }
            let v = f64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
            pos += 8;
            v
        }};
    }

    // Magic
    if data.len() < 4 || &data[0..4] != MAGIC {
        return Err(SerializeError::InvalidMagic);
    }
    pos = 4;

    // Version — accept v2 (legacy, no geometry) and v3 (with geometry)
    let version = read_u32!("version");
    if version != FORMAT_VERSION && version != FORMAT_VERSION_V2 {
        return Err(SerializeError::UnsupportedVersion(version));
    }

    let n_vertices = read_u32!("n_vertices") as usize;
    let n_edges_hdr = read_u32!("n_edges") as usize;

    // Vertex weights (v2+ addition; v1 files have none — backward compat below)
    let vertex_weights: Vec<i64> = if version >= 2 {
        (0..n_vertices).map(|_| {
            if pos + 8 > data.len() { return 1i64; }
            let v = i64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
            pos += 8;
            v
        }).collect()
    } else {
        vec![1i64; n_vertices] // v1 files: default to 1 (no population data)
    };

    // Adjacency
    let mut adjacency: Vec<Vec<usize>> = Vec::with_capacity(n_vertices);
    for _ in 0..n_vertices {
        let n_nbrs = read_u32!("n_neighbors") as usize;
        let mut nbrs = Vec::with_capacity(n_nbrs);
        for _ in 0..n_nbrs {
            nbrs.push(read_u32!("neighbor") as usize);
        }
        adjacency.push(nbrs);
    }

    // Edge weights
    let n_weights = read_u32!("n_weights") as usize;
    // Validate header n_edges matches actual weight count
    if n_weights != n_edges_hdr {
        return Err(SerializeError::EdgeCountMismatch {
            header: n_edges_hdr,
            actual: n_weights,
        });
    }
    let mut edge_weights = std::collections::HashMap::with_capacity(n_weights);
    for _ in 0..n_weights {
        let u = read_u32!("edge_u") as usize;
        let v = read_u32!("edge_v") as usize;
        let w = read_f64!("edge_weight");
        // Enforce canonical order on read (defensive)
        let key = (u.min(v), u.max(v));
        edge_weights.insert(key, w);
    }

    // v3: CompactBisect geometry (optional — v2 files have none)
    let (vertex_ext_perimeters, vertex_areas) = if version >= 3 && pos < data.len() {
        let has_geometry = read_u32!("has_geometry");
        if has_geometry == 1 {
            let mut ext_perims = Vec::with_capacity(n_vertices);
            for _ in 0..n_vertices { ext_perims.push(read_f64!("ext_perimeter")); }
            let mut areas = Vec::with_capacity(n_vertices);
            for _ in 0..n_vertices { areas.push(read_f64!("vertex_area")); }
            (ext_perims, areas)
        } else {
            (Vec::new(), Vec::new())
        }
    } else {
        (Vec::new(), Vec::new()) // v2 files: no geometry
    };

    Ok(AdjacencyGraph {
        adjacency,
        vertex_weights,
        edge_weights,
        n_vertices,
        n_edges: n_weights,
        vertex_ext_perimeters,
        vertex_areas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_graph() -> AdjacencyGraph {
        // Linear 0-1-2
        let adjacency = vec![vec![1], vec![0, 2], vec![1]];
        let mut edge_weights = HashMap::new();
        edge_weights.insert((0, 1), 500.5_f64);
        edge_weights.insert((1, 2), 300.0_f64);
        AdjacencyGraph {
            adjacency, vertex_weights: vec![1000, 1200, 900],
            edge_weights, n_vertices: 3, n_edges: 2,
            vertex_areas: Vec::new(), vertex_ext_perimeters: Vec::new(),
        }
    }

    #[test]
    fn test_roundtrip() {
        let g = make_graph();
        let bytes = serialize_adjacency(&g);
        let g2 = deserialize_adjacency(&bytes).unwrap();
        assert_eq!(g.n_vertices, g2.n_vertices);
        assert_eq!(g.n_edges, g2.n_edges);
        assert_eq!(g.adjacency, g2.adjacency);
        assert_eq!(g.edge_weights, g2.edge_weights);
    }

    #[test]
    fn test_magic_header() {
        let g = make_graph();
        let bytes = serialize_adjacency(&g);
        assert_eq!(&bytes[0..4], b"RADJ");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 3u32); // format v3 (with geometry)
    }

    #[test]
    fn test_invalid_magic_error() {
        let mut bytes = serialize_adjacency(&make_graph());
        bytes[0] = b'X'; // corrupt magic
        let result = deserialize_adjacency(&bytes);
        assert!(matches!(result, Err(SerializeError::InvalidMagic)));
    }

    #[test]
    fn test_truncated_data_error() {
        let bytes = serialize_adjacency(&make_graph());
        let short = &bytes[..8]; // only magic + version
        let result = deserialize_adjacency(short);
        assert!(matches!(result, Err(SerializeError::Truncated { .. })));
    }

    #[test]
    fn test_edge_weights_sorted_in_output() {
        // Edge weights must be written in sorted (u,v) order
        let g = make_graph();
        let bytes = serialize_adjacency(&g);
        let g2 = deserialize_adjacency(&bytes).unwrap();
        // All keys canonical
        for &(u, v) in g2.edge_weights.keys() {
            assert!(u < v, "edge ({u},{v}) not canonical");
        }
    }

    #[test]
    fn test_empty_graph_roundtrip() {
        let g = AdjacencyGraph {
            adjacency: vec![vec![], vec![]],
            vertex_weights: vec![500, 700],
            edge_weights: HashMap::new(),
            n_vertices: 2,
            n_edges: 0,
            vertex_areas: Vec::new(),
            vertex_ext_perimeters: Vec::new(),
        };
        let bytes = serialize_adjacency(&g);
        let g2 = deserialize_adjacency(&bytes).unwrap();
        assert_eq!(g2.n_vertices, 2);
        assert_eq!(g2.n_edges, 0);
        assert!(g2.edge_weights.is_empty());
    }

    #[test]
    fn test_edge_count_mismatch_error() {
        // Construct bytes with mismatched n_edges header vs actual weight count
        let g = make_graph();
        let mut bytes = serialize_adjacency(&g);
        // Header n_edges is at offset 12..16 (after magic, version, n_vertices)
        // Change it to 999
        bytes[12] = 0xe7; bytes[13] = 0x03; bytes[14] = 0x00; bytes[15] = 0x00; // 999 LE
        let result = deserialize_adjacency(&bytes);
        assert!(matches!(result, Err(SerializeError::EdgeCountMismatch { .. })),
            "mismatched n_edges should be caught");
    }

    #[test]
    #[should_panic(expected = "not canonical")]
    fn test_serialize_panics_on_non_canonical_key() {
        let mut g = make_graph();
        g.edge_weights.insert((2, 0), 100.0); // non-canonical: 2 > 0
        serialize_adjacency(&g); // must panic
    }

    #[test]
    fn test_large_graph_roundtrip() {
        // 100 nodes in a cycle with edge weights
        let n = 100usize;
        let adjacency: Vec<Vec<usize>> = (0..n)
            .map(|i| vec![(i + n - 1) % n, (i + 1) % n])
            .collect();
        let edge_weights: HashMap<(usize, usize), f64> = (0..n)
            .map(|i| ((i, (i + 1) % n), 100.0 + i as f64))
            .filter(|&((u, v), _)| u < v)
            .collect();
        let n_edges = edge_weights.len();
        let vertex_weights = vec![1000i64; n];
        let g = AdjacencyGraph { adjacency, vertex_weights, edge_weights, n_vertices: n, n_edges,
                                vertex_areas: Vec::new(), vertex_ext_perimeters: Vec::new() };
        let bytes = serialize_adjacency(&g);
        let g2 = deserialize_adjacency(&bytes).unwrap();
        assert_eq!(g.adjacency, g2.adjacency);
        assert_eq!(g.edge_weights.len(), g2.edge_weights.len());
    }
}
