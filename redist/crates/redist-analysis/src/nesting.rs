/// Chamber adjacency graph construction and nesting validation.
/// Spec 5 — board amendments R3 applied.
use std::collections::{HashMap, HashSet, VecDeque};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Chamber adjacency
// ---------------------------------------------------------------------------

/// Build an adjacency graph where nodes = house districts (0-indexed),
/// edges = shared tract boundaries.
///
/// Uses PRIMARY COMPONENT ONLY of each house district (largest connected
/// component by tract count). Tracts in secondary components are excluded.
///
/// Board amendment (MERIDIAN): tie-break between equal-size components by
/// minimum GEOID lexicographic order.
pub fn build_chamber_adjacency(
    house_assignments: &HashMap<String, usize>,
    tract_adjacency: &[Vec<usize>],
    tract_ids: &[String],
    num_house_districts: usize,
) -> Vec<Vec<usize>> {
    // Group tract indices by house district (1-indexed → 0-indexed node in output)
    let mut district_tracts: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, geoid) in tract_ids.iter().enumerate() {
        if let Some(&dist) = house_assignments.get(geoid) {
            district_tracts.entry(dist).or_default().push(idx);
        }
    }

    // For each district, find primary component (largest; tie-break by min GEOID)
    let mut primary_tract_sets: Vec<HashSet<usize>> = vec![HashSet::new(); num_house_districts];

    for dist in 1..=num_house_districts {
        let tracts = match district_tracts.get(&dist) {
            Some(t) => t,
            None => continue,
        };
        let tract_set: HashSet<usize> = tracts.iter().cloned().collect();

        // BFS to find connected components within this district
        let mut visited: HashSet<usize> = HashSet::new();
        let mut components: Vec<Vec<usize>> = Vec::new();

        for &start in tracts {
            if visited.contains(&start) {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited.insert(start);
            while let Some(node) = queue.pop_front() {
                component.push(node);
                for &neighbor in &tract_adjacency[node] {
                    if !visited.contains(&neighbor) && tract_set.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
            components.push(component);
        }

        if components.is_empty() {
            continue;
        }

        // Pick primary: largest component. Tie-break: min GEOID lexicographic.
        components.sort_by(|a, b| {
            let size_cmp = b.len().cmp(&a.len()); // larger first
            if size_cmp != std::cmp::Ordering::Equal {
                return size_cmp;
            }
            // Tie-break: component with minimum GEOID wins (board amendment MERIDIAN)
            let min_a = a.iter().map(|&i| &tract_ids[i]).min().map(|s| s.as_str()).unwrap_or("");
            let min_b = b.iter().map(|&i| &tract_ids[i]).min().map(|s| s.as_str()).unwrap_or("");
            min_a.cmp(min_b) // lexicographically smaller geoid comes first
        });

        let primary: HashSet<usize> = components[0].iter().cloned().collect();

        if components.len() > 1 {
            let secondary_count: usize = components[1..].iter().map(|c| c.len()).sum();
            eprintln!(
                "WARNING: house district {dist} has {} disconnected component(s) with {secondary_count} secondary tract(s); \
                 using primary component for adjacency",
                components.len()
            );
        }

        primary_tract_sets[dist - 1] = primary;
    }

    // Build house-district adjacency: two districts adjacent iff any primary-component
    // tract of one is adjacent to any primary-component tract of the other.
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); num_house_districts];

    for (dist_a_idx, primary_a) in primary_tract_sets.iter().enumerate() {
        for &tract_a in primary_a {
            for &neighbor in &tract_adjacency[tract_a] {
                // Find which district this neighbor belongs to
                if let Some(geoid) = tract_ids.get(neighbor) {
                    if let Some(&dist_b) = house_assignments.get(geoid) {
                        let dist_b_idx = dist_b - 1; // convert to 0-indexed
                        // Verify neighbor is in that district's primary component
                        if dist_b_idx != dist_a_idx
                            && dist_b_idx < num_house_districts
                            && primary_tract_sets[dist_b_idx].contains(&neighbor)
                        {
                            adj[dist_a_idx].insert(dist_b_idx);
                            adj[dist_b_idx].insert(dist_a_idx);
                        }
                    }
                }
            }
        }
    }

    adj.into_iter()
        .map(|set| {
            let mut v: Vec<usize> = set.into_iter().collect();
            v.sort();
            v
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Nesting validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct NestingViolation {
    pub senate_district: usize,
    pub house_districts: Vec<usize>,
    pub expected_count: usize,
}

#[derive(Debug, Serialize)]
pub struct NestingValidation {
    pub valid: bool,
    pub violations: Vec<NestingViolation>,
    pub senate_to_house_map: HashMap<usize, Vec<usize>>,
}

/// Validate that every senate district contains exactly `required_ratio` whole
/// house districts (no house district spans multiple senate districts).
pub fn validate_nesting(
    house_assignments: &HashMap<String, usize>,
    senate_assignments: &HashMap<String, usize>,
    required_ratio: usize,
) -> NestingValidation {
    // Build senate_district → set of house districts
    let mut senate_to_house: HashMap<usize, HashSet<usize>> = HashMap::new();
    // Also track house_district → set of senate districts (to catch spanning)
    let mut house_to_senate: HashMap<usize, HashSet<usize>> = HashMap::new();

    for (geoid, &senate_dist) in senate_assignments {
        if let Some(&house_dist) = house_assignments.get(geoid) {
            senate_to_house.entry(senate_dist).or_default().insert(house_dist);
            house_to_senate.entry(house_dist).or_default().insert(senate_dist);
        }
    }

    let mut violations: Vec<NestingViolation> = Vec::new();

    for (&senate_dist, house_dists) in &senate_to_house {
        let mut house_list: Vec<usize> = house_dists.iter().cloned().collect();
        house_list.sort();
        if house_list.len() != required_ratio {
            violations.push(NestingViolation {
                senate_district: senate_dist,
                house_districts: house_list,
                expected_count: required_ratio,
            });
        }
    }

    // Also flag house districts that span multiple senate districts
    for (&house_dist, senate_dists) in &house_to_senate {
        if senate_dists.len() > 1 {
            // Find senate districts that contain this house district
            for &senate_dist in senate_dists {
                let already_listed = violations.iter().any(|v| v.senate_district == senate_dist);
                if !already_listed {
                    if let Some(house_set) = senate_to_house.get(&senate_dist) {
                        let mut house_list: Vec<usize> = house_set.iter().cloned().collect();
                        house_list.sort();
                        violations.push(NestingViolation {
                            senate_district: senate_dist,
                            house_districts: house_list,
                            expected_count: required_ratio,
                        });
                    }
                }
            }
            eprintln!(
                "WARNING: house district {house_dist} spans {} senate districts — nesting violation",
                senate_dists.len()
            );
        }
    }

    violations.sort_by_key(|v| v.senate_district);
    violations.dedup_by_key(|v| v.senate_district);

    let senate_to_house_map: HashMap<usize, Vec<usize>> = senate_to_house
        .into_iter()
        .map(|(k, set)| {
            let mut v: Vec<usize> = set.into_iter().collect();
            v.sort();
            (k, v)
        })
        .collect();

    NestingValidation {
        valid: violations.is_empty(),
        violations,
        senate_to_house_map,
    }
}

/// Compute ratio: num_house / num_senate. Returns None if not evenly divisible.
pub fn compute_nest_ratio(num_house: usize, num_senate: usize) -> Option<usize> {
    if num_senate == 0 || num_house % num_senate != 0 {
        None
    } else {
        Some(num_house / num_senate)
    }
}

/// Bitfield exit code: bit 0=balance, bit 1=contiguity, bit 2=nesting.
/// Kept here for backward compatibility; prefer `exit_codes::compute_exit_code`.
pub fn compute_exit_code(
    balance_violation: bool,
    contiguity_violation: bool,
    nesting_violation: bool,
    _reserved: bool,
) -> u8 {
    (balance_violation as u8)
        | ((contiguity_violation as u8) << 1)
        | ((nesting_violation as u8) << 2)
}

/// Alias for the nesting-specific exit code check (bit 2 = 4).
pub fn nesting_exit_code(nesting_violation: bool) -> u8 {
    if nesting_violation { 4 } else { 0 }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_chamber_adjacency_simple() {
        let house_asgn: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 2), ("t3".into(), 2),
        ].into();
        // Linear chain: t0-t1, t1-t2, t2-t3
        let tract_adj: Vec<Vec<usize>> = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
        let tract_ids: Vec<String> = vec!["t0".into(), "t1".into(), "t2".into(), "t3".into()];
        let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
        assert!(house_adj[0].contains(&1), "house dist 0 must be adjacent to house dist 1");
        assert!(house_adj[1].contains(&0), "adjacency must be symmetric");
    }

    #[test]
    fn test_build_chamber_adjacency_no_cross_adjacency() {
        let house_asgn: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 2), ("t3".into(), 2),
        ].into();
        let tract_adj: Vec<Vec<usize>> = vec![vec![1], vec![0], vec![3], vec![2]]; // no t1-t2 edge
        let tract_ids: Vec<String> = vec!["t0".into(), "t1".into(), "t2".into(), "t3".into()];
        let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
        assert!(house_adj[0].is_empty(), "no cross-district boundary -> no adjacency");
        assert!(house_adj[1].is_empty());
    }

    #[test]
    fn test_build_chamber_adjacency_primary_component_only() {
        // House district 1 has two disconnected components: t0 (isolated) and t3 (adjacent to t2 in dist2)
        let house_asgn: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 2), ("t2".into(), 2), ("t3".into(), 1),
        ].into();
        let tract_adj: Vec<Vec<usize>> = vec![
            vec![],       // t0: isolated
            vec![2],      // t1: adj to t2
            vec![1, 3],   // t2: adj to t1, t3
            vec![2],      // t3: adj to t2 (secondary component of dist1)
        ];
        let tract_ids: Vec<String> = vec!["t0".into(), "t1".into(), "t2".into(), "t3".into()];
        let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
        // Must return valid-length vec without panicking
        assert_eq!(house_adj.len(), 2);
        // Both components have size 1; tie-break by GEOID: "t0" < "t3" → primary = t0
        // t0 is isolated → no cross-district edges from primary → dist1 and dist2 NOT adjacent
        assert!(
            house_adj[0].is_empty(),
            "primary component t0 has no neighbors in dist2, so dist1-dist2 must not be adjacent"
        );
    }

    #[test]
    fn test_chamber_adjacency_tie_break_by_min_geoid() {
        // Two components of size 1: geoid "53001" (isolated) and geoid "53002" (touches dist2)
        // MERIDIAN amendment: "53001" < "53002" → "53001" is primary → no phantom edge
        let house_asgn: HashMap<String, usize> = [
            ("53001".into(), 1),
            ("53002".into(), 1),
            ("53003".into(), 2),
        ].into();
        let tract_adj: Vec<Vec<usize>> = vec![
            vec![],      // "53001": isolated
            vec![2],     // "53002": adj to "53003" (dist2)
            vec![1],     // "53003": adj to "53002"
        ];
        let tract_ids: Vec<String> = vec!["53001".into(), "53002".into(), "53003".into()];
        let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
        // Primary of dist1 = "53001" (min GEOID). "53001" has no edge to "53003" → no adjacency.
        assert!(
            house_adj[0].is_empty(),
            "GEOID tie-break: primary='53001' (no neighbors) → dist1 not adjacent to dist2"
        );
    }

    #[test]
    fn test_nesting_validation_perfect() {
        let house: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 2), ("t3".into(), 2),
            ("t4".into(), 3), ("t5".into(), 3), ("t6".into(), 4), ("t7".into(), 4),
        ].into();
        let senate: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 1), ("t3".into(), 1),
            ("t4".into(), 2), ("t5".into(), 2), ("t6".into(), 2), ("t7".into(), 2),
        ].into();
        let result = validate_nesting(&house, &senate, 2);
        assert!(result.valid);
        assert!(result.violations.is_empty());
        assert_eq!(result.senate_to_house_map[&1].len(), 2);
    }

    #[test]
    fn test_nesting_validation_violation() {
        let house: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 2), ("t2".into(), 3),
        ].into();
        let senate: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 1),
        ].into();
        let result = validate_nesting(&house, &senate, 2);
        assert!(!result.valid);
        assert!(!result.violations.is_empty());
        let v = &result.violations[0];
        assert_eq!(v.senate_district, 1);
        assert_eq!(v.house_districts.len(), 3);
    }

    #[test]
    fn test_nesting_ratio_computed() {
        assert_eq!(compute_nest_ratio(98, 49), Some(2));
        assert_eq!(compute_nest_ratio(98, 48), None);
        assert_eq!(compute_nest_ratio(99, 33), Some(3));
    }

    #[test]
    fn test_nesting_violation_senate_contains_many_house_districts() {
        // Senate 3 contains tracts from house districts 3, 5, 6, 7 (4 total).
        // With required_ratio=2, this is a violation regardless of count.
        let house: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 2), ("t2".into(), 3), ("t3".into(), 3),
            ("t4".into(), 5), ("t5".into(), 5), ("t6".into(), 6), ("t7".into(), 6),
            ("t8".into(), 7), ("t9".into(), 7),
        ].into();
        let senate: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1),
            ("t2".into(), 3), ("t3".into(), 3), ("t4".into(), 3), ("t5".into(), 3),
            ("t6".into(), 3), ("t7".into(), 3), ("t8".into(), 3), ("t9".into(), 3),
        ].into();
        let result = validate_nesting(&house, &senate, 2);
        assert!(!result.valid, "senate 3 spans multiple house districts — must be invalid");
        let v = result.violations.iter().find(|v| v.senate_district == 3).unwrap();
        // Senate 3 contains house districts 3, 5, 6, 7 → 4 districts, not the required 2
        assert!(v.house_districts.len() > 2,
            "violation must show more than required_ratio (2) house districts, got {}",
            v.house_districts.len());
        assert_eq!(v.expected_count, 2, "expected_count must equal required_ratio");
    }

    #[test]
    fn test_nesting_exit_code_is_bit2() {
        assert_eq!(compute_exit_code(false, false, true, false), 4);
    }

    #[test]
    fn test_balance_and_nesting_exit_code() {
        assert_eq!(compute_exit_code(true, false, true, false), 5);
    }

    #[test]
    fn test_exit_code_zero_when_no_violations() {
        assert_eq!(compute_exit_code(false, false, false, false), 0);
    }

    #[test]
    fn test_all_violations_exit_code() {
        assert_eq!(compute_exit_code(true, true, true, false), 7);
    }

    // ── compute_nest_ratio edge cases ───────────────────────────────────────

    #[test]
    fn test_nest_ratio_zero_senate_returns_none() {
        assert_eq!(compute_nest_ratio(100, 0), None);
    }

    #[test]
    fn test_nest_ratio_one_to_one() {
        assert_eq!(compute_nest_ratio(50, 50), Some(1));
    }

    #[test]
    fn test_nest_ratio_not_divisible_returns_none() {
        assert_eq!(compute_nest_ratio(101, 50), None);
    }

    #[test]
    fn test_nest_ratio_large_values() {
        assert_eq!(compute_nest_ratio(400, 100), Some(4));
    }

    #[test]
    fn test_nest_ratio_house_zero_returns_none() {
        // 0 / 5 = 0, which is divisible, but Some(0) is a valid result
        assert_eq!(compute_nest_ratio(0, 5), Some(0));
    }

    // ── nesting_exit_code ───────────────────────────────────────────────────

    #[test]
    fn test_nesting_exit_code_true() {
        assert_eq!(nesting_exit_code(true), 4);
    }

    #[test]
    fn test_nesting_exit_code_false() {
        assert_eq!(nesting_exit_code(false), 0);
    }

    // ── contiguity exit code bit 1 ──────────────────────────────────────────

    #[test]
    fn test_contiguity_only_exit_code() {
        assert_eq!(compute_exit_code(false, true, false, false), 2);
    }

    // ── validate_nesting with ratio 3 ──────────────────────────────────────

    #[test]
    fn test_nesting_validation_perfect_ratio_3() {
        // Senate 1 contains house districts 1, 2, 3 (ratio = 3)
        let house: HashMap<String, usize> = [
            ("a".into(), 1), ("b".into(), 1),
            ("c".into(), 2), ("d".into(), 2),
            ("e".into(), 3), ("f".into(), 3),
        ].into();
        let senate: HashMap<String, usize> = [
            ("a".into(), 1), ("b".into(), 1),
            ("c".into(), 1), ("d".into(), 1),
            ("e".into(), 1), ("f".into(), 1),
        ].into();
        let result = validate_nesting(&house, &senate, 3);
        assert!(result.valid, "all 3 house districts nested in senate 1 — must be valid");
    }

    #[test]
    fn test_nesting_senate_to_house_map_populated() {
        let house: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 2),
        ].into();
        let senate: HashMap<String, usize> = [
            ("t0".into(), 1), ("t1".into(), 1),
        ].into();
        let result = validate_nesting(&house, &senate, 2);
        assert!(result.senate_to_house_map.contains_key(&1));
        assert_eq!(result.senate_to_house_map[&1].len(), 2);
    }
}
