//! Canonical-form assignment helpers (State Staff Interop plan Task 8).
//!
//! Round-trip equality definition (spec §6): for an upstream tool T and a plan
//! P, `T(redist(T(P))) == P` iff for every geometry ID g in T's source format,
//! the destination district assigned to g is identical, AFTER canonicalizing
//! district labels (district numbers may be permuted by the tool, but the
//! partition must be the same).
//!
//! Canonicalization rule: re-number districts in increasing order of their
//! lowest-GEOID member. Tie-break is impossible because GEOIDs are unique.
//! Returns a `BTreeMap` so equality is order-independent.

use std::collections::{BTreeMap, HashMap};

/// Canonicalize an assignment map by re-labeling districts so that:
/// - District 1 contains the GEOID with the lexicographically smallest value
///   among all districts;
/// - District 2 contains the GEOID with the smallest value among all OTHER
///   districts; etc.
///
/// This collapses pairs of assignments that differ only by district-label
/// permutation into the same canonical form.
pub fn canonicalize_assignments(
    assignments: &HashMap<String, usize>,
) -> BTreeMap<String, usize> {
    if assignments.is_empty() {
        return BTreeMap::new();
    }
    // For each existing district label, find the lexicographically smallest GEOID.
    let mut smallest_geoid_per_district: HashMap<usize, &str> = HashMap::new();
    for (geoid, &dist) in assignments {
        let entry = smallest_geoid_per_district.entry(dist).or_insert(geoid);
        if geoid.as_str() < *entry {
            *entry = geoid;
        }
    }
    // Sort districts by their smallest GEOID; assign new labels 1..=n.
    let mut order: Vec<(usize, &str)> = smallest_geoid_per_district
        .iter()
        .map(|(d, g)| (*d, *g))
        .collect();
    order.sort_by(|a, b| a.1.cmp(b.1));
    let label_map: HashMap<usize, usize> = order
        .iter()
        .enumerate()
        .map(|(i, (d, _))| (*d, i + 1))
        .collect();
    // Build the canonical BTreeMap.
    let mut out = BTreeMap::new();
    for (geoid, &dist) in assignments {
        out.insert(geoid.clone(), label_map[&dist]);
    }
    out
}

/// Structured diff produced when two assignments are NOT canonically equal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentDiff {
    /// GEOIDs present in `a` but missing from `b`.
    pub missing_in_b: Vec<String>,
    /// GEOIDs present in `b` but missing from `a`.
    pub missing_in_a: Vec<String>,
    /// GEOIDs present in both but assigned to different (canonicalized)
    /// districts. Each entry is `(geoid, a_canonical_district, b_canonical_district)`.
    pub differing: Vec<(String, usize, usize)>,
}

impl AssignmentDiff {
    pub fn is_empty(&self) -> bool {
        self.missing_in_a.is_empty() && self.missing_in_b.is_empty() && self.differing.is_empty()
    }
}

/// Compute the canonical-form diff between two assignment maps.
/// Returns an empty diff iff the two are canonically equal.
pub fn diff_assignments(
    a: &HashMap<String, usize>,
    b: &HashMap<String, usize>,
) -> AssignmentDiff {
    let ca = canonicalize_assignments(a);
    let cb = canonicalize_assignments(b);
    let mut missing_in_a: Vec<String> = Vec::new();
    let mut missing_in_b: Vec<String> = Vec::new();
    let mut differing: Vec<(String, usize, usize)> = Vec::new();
    for (g, da) in &ca {
        match cb.get(g) {
            None => missing_in_b.push(g.clone()),
            Some(db) if db != da => differing.push((g.clone(), *da, *db)),
            _ => {}
        }
    }
    for g in cb.keys() {
        if !ca.contains_key(g) {
            missing_in_a.push(g.clone());
        }
    }
    missing_in_a.sort();
    missing_in_b.sort();
    differing.sort_by(|x, y| x.0.cmp(&y.0));
    AssignmentDiff {
        missing_in_a,
        missing_in_b,
        differing,
    }
}

/// Assert canonical equality with a structured error message on failure.
/// Returns `Ok(())` iff `diff_assignments(a, b).is_empty()`.
pub fn assert_canonical_equal(
    a: &HashMap<String, usize>,
    b: &HashMap<String, usize>,
) -> Result<(), String> {
    let diff = diff_assignments(a, b);
    if diff.is_empty() {
        return Ok(());
    }
    let mut s = String::from("[INPUT] assignments are not canonically equal:\n");
    if !diff.missing_in_b.is_empty() {
        s.push_str(&format!(
            "  GEOIDs in A but missing from B ({}): {}\n",
            diff.missing_in_b.len(),
            diff.missing_in_b
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if !diff.missing_in_a.is_empty() {
        s.push_str(&format!(
            "  GEOIDs in B but missing from A ({}): {}\n",
            diff.missing_in_a.len(),
            diff.missing_in_a
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if !diff.differing.is_empty() {
        s.push_str(&format!(
            "  GEOIDs assigned to different districts ({}; first 5): {}\n",
            diff.differing.len(),
            diff.differing
                .iter()
                .take(5)
                .map(|(g, da, db)| format!("{g} (A={da}, B={db})"))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    Err(s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(g, d)| (g.to_string(), *d)).collect()
    }

    #[test]
    fn test_empty_assignments_canonicalize_to_empty() {
        let m = HashMap::new();
        assert!(canonicalize_assignments(&m).is_empty());
    }

    #[test]
    fn test_canonicalize_relabels_by_min_geoid() {
        // District 7 has min GEOID "01001"; district 3 has min GEOID "02002".
        // After canonicalization: district 7 -> 1, district 3 -> 2.
        let m = map(&[("01001", 7), ("01002", 7), ("02002", 3), ("02003", 3)]);
        let c = canonicalize_assignments(&m);
        assert_eq!(c["01001"], 1);
        assert_eq!(c["01002"], 1);
        assert_eq!(c["02002"], 2);
        assert_eq!(c["02003"], 2);
    }

    #[test]
    fn test_canonicalize_collapses_label_permutation() {
        // Two assignments differ only by a 1<->2 swap; canonical forms must match.
        let a = map(&[("01001", 1), ("01002", 1), ("02002", 2), ("02003", 2)]);
        let b = map(&[("01001", 2), ("01002", 2), ("02002", 1), ("02003", 1)]);
        let ca = canonicalize_assignments(&a);
        let cb = canonicalize_assignments(&b);
        assert_eq!(ca, cb, "label-permuted assignments must canonicalize to the same map");
    }

    #[test]
    fn test_canonicalize_collapses_three_way_permutation() {
        let a = map(&[
            ("01001", 1), ("02002", 2), ("03003", 3),
        ]);
        // Permute (1->3, 2->1, 3->2)
        let b = map(&[
            ("01001", 3), ("02002", 1), ("03003", 2),
        ]);
        assert_eq!(canonicalize_assignments(&a), canonicalize_assignments(&b));
    }

    #[test]
    fn test_canonicalize_distinguishes_different_partitions() {
        // Different partitions (NOT just relabeling) must canonicalize to different forms.
        let a = map(&[("01001", 1), ("01002", 1), ("02002", 2)]);
        let b = map(&[("01001", 1), ("01002", 2), ("02002", 2)]);
        assert_ne!(canonicalize_assignments(&a), canonicalize_assignments(&b));
    }

    #[test]
    fn test_diff_empty_when_canonically_equal() {
        let a = map(&[("01001", 1), ("02002", 2)]);
        let b = map(&[("01001", 2), ("02002", 1)]);
        let diff = diff_assignments(&a, &b);
        assert!(diff.is_empty(), "label-permuted should diff to empty");
    }

    #[test]
    fn test_diff_reports_missing_in_a() {
        let a = map(&[("01001", 1)]);
        let b = map(&[("01001", 1), ("02002", 2)]);
        let diff = diff_assignments(&a, &b);
        assert!(!diff.is_empty());
        assert_eq!(diff.missing_in_a, vec!["02002".to_string()]);
        assert!(diff.missing_in_b.is_empty());
    }

    #[test]
    fn test_diff_reports_differing_district() {
        let a = map(&[("01001", 1), ("02002", 2)]);
        // Same GEOIDs but the partition differs (01001+02002 in same district).
        let b = map(&[("01001", 1), ("02002", 1)]);
        let diff = diff_assignments(&a, &b);
        assert!(!diff.is_empty());
        assert_eq!(diff.differing.len(), 1);
        assert_eq!(diff.differing[0].0, "02002");
    }

    #[test]
    fn test_assert_canonical_equal_ok() {
        let a = map(&[("01001", 1), ("02002", 2)]);
        let b = map(&[("01001", 99), ("02002", 100)]);  // arbitrary labels
        assert!(assert_canonical_equal(&a, &b).is_ok());
    }

    #[test]
    fn test_assert_canonical_equal_err_includes_input_prefix() {
        let a = map(&[("01001", 1)]);
        let b = map(&[("01001", 1), ("02002", 2)]);
        let err = assert_canonical_equal(&a, &b).unwrap_err();
        assert!(err.starts_with("[INPUT]"), "error must use [INPUT] category: {err}");
        assert!(err.contains("02002"), "error must name the offending GEOID: {err}");
    }
}
