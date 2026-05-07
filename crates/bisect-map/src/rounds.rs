use std::collections::HashMap;

/// The full bisection hierarchy.
/// `rounds[r]` = list of (region_id, tract_indices_in_region) at round r.
pub struct BisectionTree {
    pub rounds: Vec<Vec<(usize, Vec<usize>)>>,
}

/// Lineage: maps (round, region_id) → parent region_id at round-1.
pub struct Lineage {
    /// (round, region_id) → parent_region_id
    map: HashMap<(usize, usize), usize>,
}

impl Lineage {
    /// Returns the parent region_id of `region` at `round` (round > 0).
    pub fn parent_of(&self, round: usize, region: usize) -> Option<usize> {
        self.map.get(&(round, region)).copied()
    }
}

/// How many final districts will come from `region` at `round`.
pub fn districts_in_region(tree: &BisectionTree, round: usize, region: usize) -> usize {
    // A region's final district count = number of leaf regions that descend from it.
    // Simplest: count how many of the last round's regions are descendants.
    // For well-formed bisection trees (each region splits into 2), the count
    // equals the number of regions in the final round that contain tracts
    // that were in `region` at `round`.
    let Some(regions_at_round) = tree.rounds.get(round) else { return 0 };
    let Some((_, tracts_in_region)) = regions_at_round.iter().find(|(id, _)| *id == region) else {
        return 0;
    };
    let tract_set: std::collections::HashSet<usize> = tracts_in_region.iter().copied().collect();

    let empty = vec![];
    let last_round = tree.rounds.last().unwrap_or(&empty);
    last_round.iter()
        .filter(|(_, tracts)| tracts.iter().any(|t| tract_set.contains(t)))
        .count()
        .max(1)
}

/// Build lineage map: for each (round, region_id), find parent at round-1.
pub fn build_lineage(tree: &BisectionTree) -> Lineage {
    let mut map = HashMap::new();
    for round in 1..tree.rounds.len() {
        let prev = &tree.rounds[round - 1];
        let curr = &tree.rounds[round];
        for (region_id, tracts) in curr {
            if tracts.is_empty() { continue; }
            let sample_tract = tracts[0];
            // Find which parent region contained this tract
            if let Some((parent_id, _)) = prev.iter().find(|(_, pt)| pt.contains(&sample_tract)) {
                map.insert((round, *region_id), *parent_id);
            }
        }
    }
    Lineage { map }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> BisectionTree {
        BisectionTree {
            rounds: vec![
                // round 0: all 4 tracts in one region
                vec![(0, vec![0, 1, 2, 3])],
                // round 1: split into 2 regions
                vec![(0, vec![0, 1]), (1, vec![2, 3])],
                // round 2: each splits again
                vec![(0, vec![0]), (1, vec![1]), (2, vec![2]), (3, vec![3])],
            ],
        }
    }

    #[test]
    fn test_round_region_counts() {
        let tree = sample_tree();
        assert_eq!(districts_in_region(&tree, 0, 0), 4);
        assert_eq!(districts_in_region(&tree, 1, 0), 2);
        assert_eq!(districts_in_region(&tree, 1, 1), 2);
        assert_eq!(districts_in_region(&tree, 2, 0), 1);
    }

    #[test]
    fn test_parent_region_lookup() {
        let tree = sample_tree();
        let lineage = build_lineage(&tree);
        // Round 1 regions 0,1 both came from round 0 region 0
        assert_eq!(lineage.parent_of(1, 0), Some(0));
        assert_eq!(lineage.parent_of(1, 1), Some(0));
        // Round 2: regions 0,1 from round 1 region 0; regions 2,3 from round 1 region 1
        assert_eq!(lineage.parent_of(2, 0), Some(0));
        assert_eq!(lineage.parent_of(2, 2), Some(1));
    }

    #[test]
    fn test_round_label_with_lineage_none_at_round0() {
        use crate::labeler::round_label_with_lineage;
        let lbl = round_label_with_lineage(1, 4, 4, None);
        assert_eq!(lbl.lineage_superscript, None);
    }

    #[test]
    fn test_single_region_tree() {
        let tree = BisectionTree {
            rounds: vec![vec![(0, vec![0, 1, 2])]],
        };
        assert_eq!(districts_in_region(&tree, 0, 0), 1);
    }

    // ── Additional rounds tests ───────────────────────────────────────────────

    #[test]
    fn test_districts_in_region_unknown_round_returns_zero() {
        let tree = sample_tree();
        assert_eq!(districts_in_region(&tree, 99, 0), 0);
    }

    #[test]
    fn test_districts_in_region_unknown_region_returns_zero() {
        let tree = sample_tree();
        assert_eq!(districts_in_region(&tree, 1, 99), 0);
    }

    #[test]
    fn test_lineage_root_has_no_parent() {
        let tree = sample_tree();
        let lineage = build_lineage(&tree);
        // Round 0 has no parent — no entry in map
        assert_eq!(lineage.parent_of(0, 0), None);
    }

    #[test]
    fn test_lineage_round2_region1_parent_is_0() {
        let tree = sample_tree();
        let lineage = build_lineage(&tree);
        // Region 1 at round 2 contains tract 1 which was in region 0 at round 1
        assert_eq!(lineage.parent_of(2, 1), Some(0));
    }

    #[test]
    fn test_lineage_round2_region3_parent_is_1() {
        let tree = sample_tree();
        let lineage = build_lineage(&tree);
        // Region 3 at round 2 contains tract 3 which was in region 1 at round 1
        assert_eq!(lineage.parent_of(2, 3), Some(1));
    }

    #[test]
    fn test_empty_tree_districts_returns_zero() {
        let tree = BisectionTree { rounds: vec![] };
        assert_eq!(districts_in_region(&tree, 0, 0), 0);
    }

    #[test]
    fn test_lineage_empty_region_skipped() {
        // Build a tree where one region at round 1 has no tracts (malformed but should not panic)
        let tree = BisectionTree {
            rounds: vec![
                vec![(0, vec![0, 1])],
                vec![(0, vec![0]), (1, vec![])],   // region 1 is empty
                vec![(0, vec![0]), (1, vec![1])],
            ],
        };
        let lineage = build_lineage(&tree);
        // Region 1 at round 2 contains tract 1 (from round 1 region 0 — because
        // the empty region 1 would not be chosen)
        // Just ensure it doesn't panic
        let _ = lineage.parent_of(1, 1);
        let _ = lineage.parent_of(2, 1);
    }

    #[test]
    fn test_districts_in_region_final_round_is_one() {
        let tree = sample_tree();
        // Every leaf region at the final round should count as 1 district
        for &region in &[0, 1, 2, 3] {
            assert_eq!(districts_in_region(&tree, 2, region), 1,
                "final round region {region} must be 1 district");
        }
    }
}
