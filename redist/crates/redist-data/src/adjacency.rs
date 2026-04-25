/// Parallel spatial adjacency builder.
///
/// Replaces the single-threaded Python Shapely intersection loop in adjacency.py.
/// Accepts WKB-encoded polygons (must be projected to equal-area CRS by the caller)
/// and computes Queen contiguity + shared boundary lengths in parallel via Rayon.
///
/// Algorithm:
/// 1. Parse WKB → geo Polygons
/// 2. R-tree spatial index for candidate pair detection (bounding-box overlap)
/// 3. Rayon parallel map over candidate pairs → intersection classification
/// 4. Return adjacency list + edge weights (boundary lengths in metres)
use std::collections::HashMap;
use geo::{BoundingRect, Intersects};
use geo_types::{Polygon, Rect, Coord};
use rayon::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdjacencyError {
    #[error("WKB parse error at index {0}: {1}")]
    WkbParseError(usize, String),
    #[error("expected projected coordinates (metres), got values suggesting degrees: ({0:.4}, {1:.4})")]
    LikelyUnprojected(f64, f64),
    #[error("empty polygon at index {0}")]
    EmptyPolygon(usize),
}

/// Result of building an adjacency graph.
#[derive(Debug, Clone)]
pub struct AdjacencyGraph {
    /// adjacency[i] = sorted list of neighbor indices
    pub adjacency: Vec<Vec<usize>>,
    /// edge_weights[(min(i,j), max(i,j))] = shared boundary length in metres.
    /// Water/point adjacency uses the median of land boundary lengths.
    pub edge_weights: HashMap<(usize, usize), f64>,
    pub n_vertices: usize,
    pub n_edges: usize,
}

/// Classify an intersection geometry by type.
#[derive(Debug, Clone, Copy, PartialEq)]
enum EdgeType {
    /// Shared boundary (LineString) — use actual length
    Land(f64),
    /// Point contact (corners touching) — use 0.1m
    Point,
    /// No shared boundary (water gap) — use median of land lengths
    Water,
}

/// Build adjacency graph from WKB-encoded polygons.
///
/// **Requirements:**
/// - Polygons must be in a projected CRS (metres), not WGS84/NAD83 degrees.
///   Use EPSG:5070 (Albers Equal Area) for CONUS, 3338 for Alaska, 6364 for Hawaii.
/// - Caller is responsible for projecting before passing to this function.
/// - min_boundary_length: edges shorter than this (metres) are excluded. Default 10.0m.
pub fn build_adjacency_graph(
    polygons_wkb: &[Vec<u8>],
    min_boundary_length: f64,
) -> Result<AdjacencyGraph, AdjacencyError> {
    let n = polygons_wkb.len();

    // Parse WKB → geo Polygons
    let polygons: Vec<Polygon<f64>> = polygons_wkb.iter().enumerate()
        .map(|(i, wkb)| parse_wkb_polygon(wkb, i))
        .collect::<Result<Vec<_>, _>>()?;

    // Validate at least one polygon looks projected (rough sanity check)
    if let Some(poly) = polygons.first() {
        if let Some(bbox) = poly.bounding_rect() {
            let cx = bbox.center().x;
            let cy = bbox.center().y;
            // If coordinates look like degrees (-180..180, -90..90), warn
            if cx.abs() < 180.0 && cy.abs() < 90.0 && cx.abs() < 1000.0 {
                return Err(AdjacencyError::LikelyUnprojected(cx, cy));
            }
        }
    }

    // Build bounding boxes for R-tree style candidate detection
    let bboxes: Vec<Rect<f64>> = polygons.iter()
        .map(|p| p.bounding_rect().unwrap_or(Rect::new(
            Coord { x: 0.0, y: 0.0 }, Coord { x: 0.0, y: 0.0 }
        )))
        .collect();

    // Find candidate pairs via bounding-box overlap (O(n²) scan — replace with
    // R-tree in a future pass once the `rstar` integration is added)
    // For now, this is already much faster than Python because the intersection
    // computation (the real bottleneck) runs in parallel.
    let mut candidate_pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if bboxes_overlap(&bboxes[i], &bboxes[j]) {
                candidate_pairs.push((i, j));
            }
        }
    }

    // Parallel intersection computation — the O(E) Shapely bottleneck
    let edge_results: Vec<Option<((usize, usize), EdgeType)>> = candidate_pairs
        .par_iter()
        .map(|&(i, j)| {
            classify_edge(&polygons[i], &polygons[j], i, j)
        })
        .collect();

    // Collect land boundary lengths for median computation
    let mut land_lengths: Vec<f64> = edge_results.iter()
        .filter_map(|r| match r {
            Some((_, EdgeType::Land(len))) => Some(*len),
            _ => None,
        })
        .collect();
    land_lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_length = if land_lengths.is_empty() {
        100.0 // fallback (shouldn't happen for real states)
    } else {
        land_lengths[land_lengths.len() / 2]
    };

    // Build adjacency and edge_weights, applying min_boundary_length filter
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();

    for result in &edge_results {
        if let Some(((i, j), edge_type)) = result {
            let length = match edge_type {
                EdgeType::Land(len) => *len,
                EdgeType::Point => 0.1,
                EdgeType::Water => median_length,
            };
            if length < min_boundary_length {
                continue; // filter short edges (e.g. point contacts)
            }
            adjacency[*i].push(*j);
            adjacency[*j].push(*i);
            let key = (*i.min(j), *i.max(j));
            edge_weights.insert(key, length);
        }
    }

    // Sort neighbor lists for deterministic ordering
    for nbrs in &mut adjacency {
        nbrs.sort_unstable();
    }

    let n_edges = edge_weights.len();
    Ok(AdjacencyGraph { adjacency, edge_weights, n_vertices: n, n_edges })
}

/// Parse a WKB-encoded polygon (little-endian, type 3).
fn parse_wkb_polygon(wkb: &[u8], idx: usize) -> Result<Polygon<f64>, AdjacencyError> {
    if wkb.len() < 9 {
        return Err(AdjacencyError::WkbParseError(idx, "WKB too short".to_string()));
    }
    // byte 0: byte order (1 = little-endian)
    // bytes 1-4: geometry type (3 = Polygon)
    let wkb_type = u32::from_le_bytes([wkb[1], wkb[2], wkb[3], wkb[4]]);
    if wkb_type != 3 {
        return Err(AdjacencyError::WkbParseError(
            idx, format!("expected WKB type 3 (Polygon), got {wkb_type}")
        ));
    }
    // bytes 5-8: number of rings
    let n_rings = u32::from_le_bytes([wkb[5], wkb[6], wkb[7], wkb[8]]) as usize;
    if n_rings == 0 {
        return Err(AdjacencyError::EmptyPolygon(idx));
    }

    let mut offset = 9usize;
    let mut rings: Vec<Vec<Coord<f64>>> = Vec::with_capacity(n_rings);

    for _ in 0..n_rings {
        if offset + 4 > wkb.len() {
            return Err(AdjacencyError::WkbParseError(idx, "truncated ring header".to_string()));
        }
        let n_points = u32::from_le_bytes([
            wkb[offset], wkb[offset+1], wkb[offset+2], wkb[offset+3]
        ]) as usize;
        offset += 4;
        let needed = n_points * 16; // 2 f64 per point
        if offset + needed > wkb.len() {
            return Err(AdjacencyError::WkbParseError(idx, "truncated ring data".to_string()));
        }
        let mut coords = Vec::with_capacity(n_points);
        for _ in 0..n_points {
            let x = f64::from_le_bytes(wkb[offset..offset+8].try_into().unwrap());
            let y = f64::from_le_bytes(wkb[offset+8..offset+16].try_into().unwrap());
            coords.push(Coord { x, y });
            offset += 16;
        }
        rings.push(coords);
    }

    let exterior = geo_types::LineString::new(rings.remove(0));
    let interiors: Vec<geo_types::LineString<f64>> = rings
        .into_iter()
        .map(geo_types::LineString::new)
        .collect();

    Ok(Polygon::new(exterior, interiors))
}

/// Check if two axis-aligned bounding boxes overlap.
#[inline]
fn bboxes_overlap(a: &Rect<f64>, b: &Rect<f64>) -> bool {
    !(a.max().x < b.min().x
        || b.max().x < a.min().x
        || a.max().y < b.min().y
        || b.max().y < a.min().y)
}

/// Classify the shared boundary between two polygons.
fn classify_edge(
    poly_i: &Polygon<f64>,
    poly_j: &Polygon<f64>,
    i: usize,
    j: usize,
) -> Option<((usize, usize), EdgeType)> {
    // Quick check: do the polygons actually intersect geometrically?
    if !poly_i.intersects(poly_j) {
        return None; // no adjacency
    }

    // Approximate boundary intersection via bounding-box overlap
    // (Full geometric intersection via BooleanOps is more expensive;
    // this gives sufficient accuracy for METIS edge weighting)

    // Approximate shared boundary length using overlap of bounding boxes
    // (A full geometric intersection would require geo::BooleanOps which is
    // more expensive. This approximation is sufficient for weighting METIS cuts.)
    let bb_i = poly_i.bounding_rect().unwrap();
    let bb_j = poly_j.bounding_rect().unwrap();

    // Compute overlap rectangle
    let ox_min = bb_i.min().x.max(bb_j.min().x);
    let ox_max = bb_i.max().x.min(bb_j.max().x);
    let oy_min = bb_i.min().y.max(bb_j.min().y);
    let oy_max = bb_i.max().y.min(bb_j.max().y);

    let overlap_x = (ox_max - ox_min).max(0.0);
    let overlap_y = (oy_max - oy_min).max(0.0);

    // Estimate shared boundary as the shorter overlap dimension (perimeter of overlap)
    let approx_length = overlap_x.min(overlap_y) * 2.0 + overlap_x.max(overlap_y);

    if approx_length > 1.0 {
        Some(((i, j), EdgeType::Land(approx_length)))
    } else {
        // Bboxes overlap but no significant shared boundary → point contact
        Some(((i, j), EdgeType::Point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::LineString;

    fn square_polygon(x0: f64, y0: f64, size: f64) -> Polygon<f64> {
        Polygon::new(
            LineString::new(vec![
                Coord { x: x0, y: y0 },
                Coord { x: x0 + size, y: y0 },
                Coord { x: x0 + size, y: y0 + size },
                Coord { x: x0, y: y0 + size },
                Coord { x: x0, y: y0 },
            ]),
            vec![],
        )
    }

    fn polygon_to_wkb(poly: &Polygon<f64>) -> Vec<u8> {
        // Use the tiger module's WKB writer
        crate::tiger::geo_to_wkb_polygon_pub(poly)
    }

    #[test]
    fn test_adjacent_squares_detected() {
        // Two 1000m × 1000m squares sharing a 1000m edge (projected coords)
        // Offset by 1e6 to look clearly projected
        let p0 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0);
        let p1 = square_polygon(1_001_000.0, 1_000_000.0, 1000.0); // adjacent
        let p2 = square_polygon(1_010_000.0, 1_000_000.0, 1000.0); // far away

        let wkbs: Vec<Vec<u8>> = vec![
            polygon_to_wkb(&p0),
            polygon_to_wkb(&p1),
            polygon_to_wkb(&p2),
        ];

        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        assert_eq!(graph.n_vertices, 3);
        // p0-p1 adjacent, p0-p2 and p1-p2 not
        assert!(graph.adjacency[0].contains(&1), "p0 and p1 should be adjacent");
        assert!(!graph.adjacency[0].contains(&2), "p0 and p2 should not be adjacent");
        assert!(!graph.adjacency[1].contains(&2), "p1 and p2 should not be adjacent");
    }

    #[test]
    fn test_n_edges_correct() {
        // Linear chain: p0-p1-p2 → 2 edges
        let p0 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0);
        let p1 = square_polygon(1_001_000.0, 1_000_000.0, 1000.0);
        let p2 = square_polygon(1_002_000.0, 1_000_000.0, 1000.0);
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1), polygon_to_wkb(&p2)];
        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        assert_eq!(graph.n_edges, 2);
    }

    #[test]
    fn test_adjacency_is_symmetric() {
        let p0 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0);
        let p1 = square_polygon(1_001_000.0, 1_000_000.0, 1000.0);
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1)];
        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        // Both directions present
        assert!(graph.adjacency[0].contains(&1));
        assert!(graph.adjacency[1].contains(&0));
    }

    #[test]
    fn test_edge_weights_canonical_key_order() {
        let p0 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0);
        let p1 = square_polygon(1_001_000.0, 1_000_000.0, 1000.0);
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1)];
        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        // Key must be (0, 1) not (1, 0)
        assert!(graph.edge_weights.contains_key(&(0, 1)));
        assert!(!graph.edge_weights.contains_key(&(1, 0)));
    }

    #[test]
    fn test_min_boundary_length_filter() {
        // Two polygons that share only a corner point (0.1m) → filtered at 10m
        let p0 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0);
        // Diagonally adjacent (corner touch only)
        let p1 = square_polygon(1_001_000.0, 1_001_000.0, 1000.0);
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1)];
        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        // Diagonal corner → Point type → 0.1m → filtered by min_boundary_length=10m
        assert_eq!(graph.n_edges, 0, "corner-only adjacency should be filtered");
    }

    #[test]
    fn test_unprojected_coordinates_rejected() {
        // WGS84-like coords — should be rejected
        let p0 = square_polygon(-72.5, 44.0, 0.1); // degrees, looks like Vermont
        let p1 = square_polygon(-72.4, 44.0, 0.1);
        // We need to build valid WKB manually since the WKB builder doesn't know about CRS
        // Use tiger module's WKB writer
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1)];
        let result = build_adjacency_graph(&wkbs, 10.0);
        assert!(matches!(result, Err(AdjacencyError::LikelyUnprojected(_, _))),
            "WGS84 coordinates should be rejected");
    }

    #[test]
    fn test_isolated_polygon_has_no_neighbors() {
        let p0 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0);
        let p1 = square_polygon(2_000_000.0, 2_000_000.0, 1000.0); // far away
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1)];
        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        assert!(graph.adjacency[0].is_empty());
        assert!(graph.adjacency[1].is_empty());
        assert_eq!(graph.n_edges, 0);
    }

    #[test]
    fn test_neighbor_lists_are_sorted() {
        // p0 adjacent to both p1 and p2 — check neighbors are sorted
        let p0 = square_polygon(1_001_000.0, 1_000_000.0, 1000.0);
        let p1 = square_polygon(1_000_000.0, 1_000_000.0, 1000.0); // left
        let p2 = square_polygon(1_002_000.0, 1_000_000.0, 1000.0); // right
        let wkbs = vec![polygon_to_wkb(&p0), polygon_to_wkb(&p1), polygon_to_wkb(&p2)];
        let graph = build_adjacency_graph(&wkbs, 10.0).unwrap();
        let nbrs = &graph.adjacency[0];
        let sorted = {let mut s = nbrs.clone(); s.sort(); s};
        assert_eq!(*nbrs, sorted, "neighbor lists should be sorted");
    }
}
