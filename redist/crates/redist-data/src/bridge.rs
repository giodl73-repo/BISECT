/// Island bridging: connect disconnected graph components.
///
/// Census tracts on islands (Hawaii, Alaska) may not share land boundaries
/// with the mainland, producing disconnected adjacency graphs. This module
/// finds the nearest mainland tract (within the same county if possible) and
/// adds bridge edges. This matches adjacency_county_bridge.py logic.
///
/// Algorithm:
/// 1. Union-Find to identify connected components
/// 2. Sort components by size (largest = main component)
/// 3. For each isolated tract: find nearest same-county tract in main component
///    (fallback: any main component tract if no same-county match)
/// 4. Return new (i, j) edge pairs to add

/// Extract county code (first 5 chars) from 11-char tract GEOID: SSCCCTTTTTT
pub fn county_from_geoid(geoid: &str) -> &str {
    &geoid[..geoid.len().min(5)]
}

/// Union-Find structure for connected component detection.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]); // path compression
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry { return; }
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
        } else if self.rank[rx] > self.rank[ry] {
            self.parent[ry] = rx;
        } else {
            self.parent[ry] = rx;
            self.rank[rx] += 1;
        }
    }
}

/// Find connected components from adjacency list.
/// Returns Vec of sets (each set = one component), largest first.
fn find_components(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adjacency.len();
    let mut uf = UnionFind::new(n);
    for (i, nbrs) in adjacency.iter().enumerate() {
        for &j in nbrs {
            uf.union(i, j);
        }
    }
    // Group vertices by root
    let mut component_map: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = uf.find(i);
        component_map.entry(root).or_default().push(i);
    }
    let mut components: Vec<Vec<usize>> = component_map.into_values().collect();
    // Largest first
    components.sort_by(|a, b| b.len().cmp(&a.len()));
    components
}

/// Euclidean distance squared between two 2D points.
#[inline]
fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx * dx + dy * dy
}

/// Find bridge edges to connect all disconnected island components to the main component.
pub fn connect_island_components(
    adjacency: &[Vec<usize>],
    centroids: &[(f64, f64)],
    geoids: &[String],
) -> Vec<(usize, usize)> {
    let components = find_components(adjacency);
    if components.len() <= 1 {
        return Vec::new(); // already connected
    }

    let main_component = &components[0];
    let counties: Vec<&str> = geoids.iter().map(|g| county_from_geoid(g)).collect();
    // HashSet for O(1) deduplication (vs O(n²) Vec::contains for island-heavy states)
    let mut new_edges_set: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    for component in &components[1..] {
        for &tract_idx in component {
            let tract_county = counties[tract_idx];
            let tract_centroid = centroids[tract_idx];

            // Find same-county candidates in main component
            let same_county: Vec<usize> = main_component.iter()
                .copied()
                .filter(|&idx| counties[idx] == tract_county)
                .collect();

            let candidates = if same_county.is_empty() {
                // Fallback: any main component tract
                main_component.as_slice()
            } else {
                same_county.as_slice()
            };

            // Find nearest candidate by Euclidean distance
            let closest = candidates.iter()
                .copied()
                .min_by(|&a, &b| {
                    dist2(centroids[a], tract_centroid)
                        .partial_cmp(&dist2(centroids[b], tract_centroid))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap(); // candidates is never empty

            let edge = (tract_idx.min(closest), tract_idx.max(closest));
            new_edges_set.insert(edge);
        }
    }

    let mut edges: Vec<_> = new_edges_set.into_iter().collect();
    edges.sort_unstable(); // deterministic output order
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_county_from_geoid() {
        assert_eq!(county_from_geoid("50005957100"), "50005");
        assert_eq!(county_from_geoid("15003010400"), "15003"); // Hawaii county
        assert_eq!(county_from_geoid("02068000100"), "02068"); // Alaska county
    }

    #[test]
    fn test_county_from_short_geoid() {
        // Handles shorter strings gracefully
        assert_eq!(county_from_geoid("50005"), "50005");
    }

    #[test]
    fn test_union_find_basic() {
        let mut uf = UnionFind::new(5);
        uf.union(0, 1);
        uf.union(1, 2);
        // 0, 1, 2 should be same component
        assert_eq!(uf.find(0), uf.find(1));
        assert_eq!(uf.find(1), uf.find(2));
        // 3, 4 different
        assert_ne!(uf.find(0), uf.find(3));
        assert_ne!(uf.find(3), uf.find(4));
    }

    #[test]
    fn test_find_components_connected() {
        // Linear chain 0-1-2 → one component
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let comps = find_components(&adj);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].len(), 3);
    }

    #[test]
    fn test_find_components_disconnected() {
        // 0-1 and 2-3 disconnected
        let adj = vec![vec![1], vec![0], vec![3], vec![2]];
        let comps = find_components(&adj);
        assert_eq!(comps.len(), 2);
        // Both have size 2; largest first (tied)
        assert_eq!(comps[0].len(), 2);
    }

    #[test]
    fn test_find_components_island() {
        // 0-1-2 main, 3 isolated
        let adj = vec![vec![1], vec![0, 2], vec![1], vec![]];
        let comps = find_components(&adj);
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].len(), 3); // main
        assert_eq!(comps[1].len(), 1); // island
    }

    #[test]
    fn test_connect_island_same_county() {
        // Main: tracts 0, 1 (county 50001)
        // Island: tract 2 (county 50001) — should connect to nearest in main
        let adj = vec![vec![1], vec![0], vec![]]; // tract 2 is isolated
        let centroids = vec![
            (1_000_000.0, 1_000_000.0),  // 0
            (1_001_000.0, 1_000_000.0),  // 1
            (1_000_500.0, 1_010_000.0),  // 2 — closer to 0 than 1
        ];
        let geoids = vec![
            "50001000100".to_string(),
            "50001000200".to_string(),
            "50001000300".to_string(),
        ];
        let edges = connect_island_components(&adj, &centroids, &geoids);
        assert_eq!(edges.len(), 1);
        // Tract 2 should connect to tract 0 (closer centroid)
        let (u, v) = edges[0];
        assert!(u == 0 || v == 0, "bridge should connect to tract 0 (nearest)");
    }

    #[test]
    fn test_connect_island_cross_county_fallback() {
        // Main: tracts 0, 1 (county 50001)
        // Island: tract 2 (county 50002) — no same-county match → fallback to nearest
        let adj = vec![vec![1], vec![0], vec![]];
        let centroids = vec![
            (1_000_000.0, 1_000_000.0),
            (1_001_000.0, 1_000_000.0),
            (1_000_500.0, 1_010_000.0),
        ];
        let geoids = vec![
            "50001000100".to_string(),
            "50001000200".to_string(),
            "50002000100".to_string(), // different county
        ];
        let edges = connect_island_components(&adj, &centroids, &geoids);
        // Should still connect despite different county
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_already_connected_returns_empty() {
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let centroids = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let geoids: Vec<String> = (0..3).map(|i| format!("5000100{:04}", i)).collect();
        let edges = connect_island_components(&adj, &centroids, &geoids);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_bridge_edges_canonical_order() {
        // Island tract 0, main tract 1 — edge should be (0,1) not (1,0)
        let adj = vec![vec![], vec![2], vec![1]];
        let centroids = vec![(0.0, 0.0), (5.0, 0.0), (6.0, 0.0)];
        let geoids = vec![
            "50001000100".to_string(),
            "50001000200".to_string(),
            "50001000300".to_string(),
        ];
        let edges = connect_island_components(&adj, &centroids, &geoids);
        for (u, v) in &edges {
            assert!(u < v, "edge ({u},{v}) not canonical");
        }
    }

    #[test]
    fn test_no_duplicate_bridge_edges() {
        // Two isolated tracts connecting to same main tract
        let adj = vec![vec![1], vec![0], vec![], vec![]];
        let centroids = vec![
            (0.0, 0.0), (1.0, 0.0),
            (0.5, 5.0), // isolates close to tract 0
            (0.5, 6.0), // also closest to tract 0
        ];
        let geoids: Vec<String> = (0..4).map(|i| format!("5000100{:04}", i)).collect();
        let edges = connect_island_components(&adj, &centroids, &geoids);
        // Tract 2 → 0 and tract 3 → 0 → two edges, but (0,2) and (0,3) are different
        let edge_set: std::collections::HashSet<_> = edges.iter().collect();
        assert_eq!(edge_set.len(), edges.len(), "no duplicate bridge edges");
    }
}
