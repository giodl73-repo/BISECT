/// BFS-based per-district contiguity check.
///
/// For each district, collects all tract indices belonging to it, then
/// runs BFS restricted to that subset using the adjacency list. If more
/// than one connected component is found the district is non-contiguous.
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of checking contiguity for all districts in a plan.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContiguityResult {
    pub all_contiguous: bool,
    pub districts: Vec<DistrictContiguity>,
}

/// Contiguity result for a single district.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DistrictContiguity {
    pub district: usize,
    pub contiguous: bool,
    pub tract_count: usize,
    pub component_count: usize,
    /// GEOIDs of tracts that are in a non-primary component (isolated).
    pub disconnected_tracts: Vec<String>,
    /// Human-readable county context for each disconnected tract.
    /// Format: "King County, WA" or raw FIPS "53033" if unknown.
    pub disconnected_tract_counties: Vec<String>,
}

/// Check contiguity for every district in `assignments`.
///
/// `adjacency[i]` is a list of tract indices adjacent to tract `i`.
/// `geoids[i]` is the GEOID string for tract index `i`.
/// `num_districts` — how many districts to check (1..=num_districts).
pub fn check_contiguity(
    assignments: &HashMap<String, usize>,
    adjacency: &[Vec<usize>],
    geoids: &[String],
    num_districts: usize,
) -> ContiguityResult {
    // Build GEOID -> index map for fast lookup.
    let geoid_to_idx: HashMap<&str, usize> = geoids
        .iter()
        .enumerate()
        .map(|(i, g)| (g.as_str(), i))
        .collect();

    let mut district_results = Vec::new();
    let mut all_contiguous = true;

    for d in 1..=num_districts {
        // Collect all tract indices for this district.
        let members: HashSet<usize> = assignments
            .iter()
            .filter(|(_, &dist)| dist == d)
            .filter_map(|(geoid, _)| geoid_to_idx.get(geoid.as_str()).copied())
            .collect();

        let tract_count = members.len();

        if tract_count == 0 {
            // Empty district — treat as contiguous (no tracts to be disconnected).
            district_results.push(DistrictContiguity {
                district: d,
                contiguous: true,
                tract_count: 0,
                component_count: 0,
                disconnected_tracts: vec![],
                disconnected_tract_counties: vec![],
            });
            continue;
        }

        let (component_count, components) = bfs_component_count(&members, adjacency);
        let contiguous = component_count == 1;
        if !contiguous {
            all_contiguous = false;
        }

        // Collect GEOIDs of tracts in non-primary components.
        // Primary component = the largest one.
        let primary_size = components.iter().map(|c| c.len()).max().unwrap_or(0);
        let mut disconnected_tracts: Vec<String> = components
            .iter()
            .filter(|c| c.len() < primary_size || component_count == 1)
            .flat_map(|c| {
                if component_count <= 1 {
                    vec![]
                } else if c.len() < primary_size {
                    c.iter()
                        .filter_map(|&idx| geoids.get(idx).cloned())
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect();

        // If multiple components have the same size, include all non-primary ones.
        // Recompute: all components except the first largest.
        if component_count > 1 {
            let mut sorted_components = components.clone();
            sorted_components.sort_by_key(|c| std::cmp::Reverse(c.len()));
            disconnected_tracts = sorted_components[1..]
                .iter()
                .flat_map(|c| c.iter().filter_map(|&idx| geoids.get(idx).cloned()))
                .collect();
        }

        // Build human-readable county context for each disconnected tract.
        let disconnected_tract_counties: Vec<String> = disconnected_tracts
            .iter()
            .map(|geoid| county_context_for_geoid(geoid))
            .collect();

        district_results.push(DistrictContiguity {
            district: d,
            contiguous,
            tract_count,
            component_count,
            disconnected_tracts,
            disconnected_tract_counties,
        });
    }

    ContiguityResult {
        all_contiguous,
        districts: district_results,
    }
}

/// Run BFS on `tract_indices` (restricted to that subset) using `adjacency`.
///
/// Returns `(component_count, vec_of_components)`.
pub fn bfs_component_count(
    tract_indices: &HashSet<usize>,
    adjacency: &[Vec<usize>],
) -> (usize, Vec<HashSet<usize>>) {
    if tract_indices.is_empty() {
        return (0, vec![]);
    }

    let mut visited: HashSet<usize> = HashSet::new();
    let mut components: Vec<HashSet<usize>> = Vec::new();

    for &start in tract_indices {
        if visited.contains(&start) {
            continue;
        }

        // BFS from `start`, restricted to `tract_indices`.
        let mut component = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            component.insert(node);
            if let Some(neighbors) = adjacency.get(node) {
                for &neighbor in neighbors {
                    if tract_indices.contains(&neighbor) && !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        components.push(component);
    }

    (components.len(), components)
}

/// Produce a human-readable county context string for a GEOID.
///
/// Extracts the first 5 characters (county FIPS) and looks up the county name.
/// Returns "King County, WA" style strings, or the raw FIPS if unknown.
pub fn county_context_for_geoid(geoid: &str) -> String {
    if geoid.len() < 5 {
        return geoid.to_string();
    }
    let county_fips = &geoid[..5];
    match crate::county_names::county_name(county_fips) {
        Some(name) => name.to_string(),
        None => county_fips.to_string(),
    }
}

// Re-export for use in nesting tests from plan (Scenario 4).
/// Validate that each upper-chamber district contains whole lower-chamber districts.
pub struct NestingResult {
    pub valid: bool,
    pub violations: Vec<NestingViolation>,
}

pub struct NestingViolation {
    pub senate_district: usize,
    pub house_districts: Vec<usize>,
}

pub fn validate_nesting(
    house: &HashMap<String, usize>,
    senate: &HashMap<String, usize>,
    house_per_senate: usize,
) -> NestingResult {
    // For each senate district, find how many distinct house districts touch it.
    let mut senate_to_house: HashMap<usize, HashSet<usize>> = HashMap::new();
    for (geoid, &s_dist) in senate {
        if let Some(&h_dist) = house.get(geoid) {
            senate_to_house
                .entry(s_dist)
                .or_default()
                .insert(h_dist);
        }
    }

    let mut violations = Vec::new();
    for (s_dist, h_dists) in &senate_to_house {
        if h_dists.len() != house_per_senate {
            violations.push(NestingViolation {
                senate_district: *s_dist,
                house_districts: h_dists.iter().cloned().collect(),
            });
        }
    }

    NestingResult {
        valid: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_assignments(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|&(k, v)| (k.to_string(), v)).collect()
    }

    #[test]
    fn test_contiguity_connected_graph() {
        // 4 tracts in a line: 0-1-2-3, all in district 1 -> contiguous
        let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
        let geoids: Vec<String> = (0..4).map(|i| format!("t{:011}", i)).collect();
        let assignments: HashMap<String, usize> = geoids
            .iter()
            .cloned()
            .zip(vec![1, 1, 1, 1])
            .collect();
        let result = check_contiguity(&assignments, &adj, &geoids, 1);
        assert!(result.all_contiguous);
        assert!(result.districts[0].contiguous);
        assert_eq!(result.districts[0].component_count, 1);
    }

    #[test]
    fn test_contiguity_disconnected_district() {
        // Tracts 0,1 connected. Tract 3 isolated. All in district 1.
        let adj = vec![vec![1usize], vec![0usize], vec![], vec![]];
        let geoids: Vec<String> = (0..4).map(|i| format!("t{:011}", i)).collect();
        let assignments: HashMap<String, usize> = make_assignments(&[
            (&geoids[0], 1),
            (&geoids[1], 1),
            (&geoids[2], 2),
            (&geoids[3], 1), // isolated from 0 and 1
        ]);
        let result = check_contiguity(&assignments, &adj, &geoids, 2);
        assert!(!result.all_contiguous);
        let d1 = result.districts.iter().find(|d| d.district == 1).unwrap();
        assert!(!d1.contiguous);
        assert_eq!(d1.component_count, 2);
        // disconnected_tracts must list tract 3
        assert!(d1.disconnected_tracts.contains(&geoids[3]));
    }

    #[test]
    fn test_contiguity_two_separate_districts_both_connected() {
        // Tracts 0,1 in district 1 (connected). Tracts 2,3 in district 2 (connected).
        let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
        let geoids: Vec<String> = (0..4).map(|i| format!("t{:011}", i)).collect();
        let assignments: HashMap<String, usize> = make_assignments(&[
            (&geoids[0], 1),
            (&geoids[1], 1),
            (&geoids[2], 2),
            (&geoids[3], 2),
        ]);
        let result = check_contiguity(&assignments, &adj, &geoids, 2);
        assert!(result.all_contiguous);
    }

    #[test]
    fn test_bfs_component_count_single_component() {
        let members: HashSet<usize> = [0, 1, 2, 3].iter().cloned().collect();
        let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
        let (count, _) = bfs_component_count(&members, &adj);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_bfs_component_count_two_components() {
        let members: HashSet<usize> = [0, 1, 3].iter().cloned().collect(); // 2 missing; 0-1 connected, 3 isolated
        let adj = vec![vec![1usize], vec![0usize], vec![], vec![]];
        let (count, _) = bfs_component_count(&members, &adj);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_contiguity_single_tract_district_is_contiguous() {
        // A district with exactly one tract is trivially contiguous
        let adj = vec![vec![]];
        let geoids = vec!["t00000000001".to_string()];
        let assignments: HashMap<String, usize> =
            [("t00000000001".to_string(), 1)].iter().cloned().collect();
        let result = check_contiguity(&assignments, &adj, &geoids, 1);
        assert!(result.districts[0].contiguous);
        assert_eq!(result.districts[0].component_count, 1);
    }

    // ── Task 142: County context for disconnected tracts ─────────────────────

    #[test]
    fn test_disconnected_tract_county_context_king_county() {
        // GEOID starting with "53033" is King County, WA
        let ctx = county_context_for_geoid("53033001234");
        assert!(ctx.contains("King"), "should contain 'King', got: {ctx}");
    }

    #[test]
    fn test_disconnected_tract_county_context_unknown_fips() {
        // Unknown FIPS should return the raw FIPS string (first 5 chars)
        let ctx = county_context_for_geoid("99999001234");
        assert_eq!(ctx, "99999", "unknown FIPS should return raw FIPS, got: {ctx}");
    }

    #[test]
    fn test_disconnected_tract_county_context_populated_in_result() {
        // When a district is non-contiguous, disconnected_tract_counties should be populated
        // with one entry per disconnected_tracts entry.
        let adj = vec![vec![1usize], vec![0usize], vec![], vec![]];
        // Use GEOIDs starting with known King County FIPS "53033"
        let geoids: Vec<String> = vec![
            "53033000001".to_string(),
            "53033000002".to_string(),
            "53001000003".to_string(), // Adams County
            "53033000004".to_string(), // isolated King County tract
        ];
        let assignments: HashMap<String, usize> = vec![
            ("53033000001".to_string(), 1),
            ("53033000002".to_string(), 1),
            ("53001000003".to_string(), 2),
            ("53033000004".to_string(), 1), // isolated
        ].into_iter().collect();
        let result = check_contiguity(&assignments, &adj, &geoids, 2);
        let d1 = result.districts.iter().find(|d| d.district == 1).unwrap();
        assert!(!d1.contiguous);
        assert_eq!(
            d1.disconnected_tracts.len(),
            d1.disconnected_tract_counties.len(),
            "county context count must match disconnected tract count"
        );
        // The isolated tract 53033000004 should have King County context
        let king_ctx = d1.disconnected_tract_counties.iter()
            .any(|c| c.contains("King"));
        assert!(king_ctx, "King County tract should get 'King' context, got: {:?}",
            d1.disconnected_tract_counties);
    }

    // ── bfs_component_count edge cases ──────────────────────────────────────

    #[test]
    fn test_bfs_component_count_empty_set_returns_zero() {
        let members: HashSet<usize> = HashSet::new();
        let adj: Vec<Vec<usize>> = vec![];
        let (count, components) = bfs_component_count(&members, &adj);
        assert_eq!(count, 0);
        assert!(components.is_empty());
    }

    #[test]
    fn test_bfs_component_count_single_node() {
        let members: HashSet<usize> = [5].iter().cloned().collect();
        let adj = vec![vec![], vec![], vec![], vec![], vec![], vec![]]; // 6 nodes, no edges
        let (count, comps) = bfs_component_count(&members, &adj);
        assert_eq!(count, 1);
        assert_eq!(comps[0].len(), 1);
    }

    #[test]
    fn test_bfs_component_count_all_isolated() {
        // Three nodes with no edges in common subset
        let members: HashSet<usize> = [0, 2, 4].iter().cloned().collect();
        let adj = vec![vec![], vec![], vec![], vec![], vec![]]; // no edges at all
        let (count, _) = bfs_component_count(&members, &adj);
        assert_eq!(count, 3, "three isolated nodes = 3 components");
    }

    #[test]
    fn test_bfs_component_count_fully_connected_chain() {
        // 5-node chain 0-1-2-3-4, all in subset → single component
        let members: HashSet<usize> = [0, 1, 2, 3, 4].iter().cloned().collect();
        let adj = vec![
            vec![1],
            vec![0, 2],
            vec![1, 3],
            vec![2, 4],
            vec![3],
        ];
        let (count, _) = bfs_component_count(&members, &adj);
        assert_eq!(count, 1);
    }

    // ── check_contiguity edge cases ─────────────────────────────────────────

    #[test]
    fn test_check_contiguity_empty_district_is_contiguous() {
        // District 2 has no tracts → should be marked contiguous with tract_count=0
        let adj = vec![vec![1usize], vec![0usize]];
        let geoids: Vec<String> = vec!["t0".to_string(), "t1".to_string()];
        let assignments: HashMap<String, usize> =
            [("t0".to_string(), 1), ("t1".to_string(), 1)].into_iter().collect();
        let result = check_contiguity(&assignments, &adj, &geoids, 2);
        let d2 = result.districts.iter().find(|d| d.district == 2).unwrap();
        assert!(d2.contiguous, "empty district must be treated as contiguous");
        assert_eq!(d2.tract_count, 0);
        assert_eq!(d2.component_count, 0);
    }

    #[test]
    fn test_check_contiguity_three_components_reports_correct_count() {
        // District 1 has 5 tracts in 3 separate components
        let adj = vec![
            vec![1], vec![0], // component A: 0-1
            vec![3], vec![2], // component B: 2-3
            vec![],           // component C: 4 isolated
        ];
        let geoids: Vec<String> = (0..5).map(|i| format!("t{i:011}")).collect();
        let assignments: HashMap<String, usize> = geoids
            .iter()
            .cloned()
            .zip(std::iter::repeat(1))
            .collect();
        let result = check_contiguity(&assignments, &adj, &geoids, 1);
        let d1 = &result.districts[0];
        assert!(!d1.contiguous);
        assert_eq!(d1.component_count, 3);
        // The primary component has size 2; the other two (size 2 + 1) are disconnected
        assert!(!d1.disconnected_tracts.is_empty());
    }

    #[test]
    fn test_nesting_violation_senate_contains_three_house_districts() {
        let house: HashMap<String, usize> = make_assignments(&[
            ("t0", 1),
            ("t1", 2),
            ("t2", 3),
            ("t3", 3),
            ("t4", 5),
            ("t5", 5),
            ("t6", 6),
            ("t7", 6),
            ("t8", 7),
            ("t9", 7),
        ]);
        let senate: HashMap<String, usize> = make_assignments(&[
            ("t0", 1),
            ("t1", 1),
            ("t2", 3),
            ("t3", 3),
            ("t4", 3),
            ("t5", 3),
            ("t6", 3),
            ("t7", 3),
            ("t8", 3),
            ("t9", 3),
        ]);
        let result = validate_nesting(&house, &senate, 2);
        assert!(!result.valid);
        let v = result.violations.iter().find(|v| v.senate_district == 3).unwrap();
        // Senate 3 contains house districts 3, 5, 6, 7
        assert!(v.house_districts.len() >= 3);
    }
}
