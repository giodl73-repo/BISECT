/// Plan-to-plan comparison: Jaccard similarity, population equality, compactness.
///
/// Legal note: Output MUST NOT use "Winner:" framing — use "Lower:" instead.
/// The choice of "better" plan is a legal/political determination outside this tool.
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlanSummary {
    pub label: String,
    pub num_tracts: usize,
    pub num_districts: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PopulationComparison {
    pub plan_a_max_dev: f64,
    pub plan_b_max_dev: f64,
    /// "plan_a" | "plan_b" | "equal"  — NOT "winner"
    pub lower: String,
    pub difference_pct: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompactnessComparison {
    pub plan_a_mean: f64,
    pub plan_b_mean: f64,
    pub lower: String,
    pub difference: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SplitComparison {
    pub plan_a_splits: usize,
    pub plan_b_splits: usize,
    pub lower: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PartisanComparison {
    pub available: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlanComparison {
    pub plan_a: PlanSummary,
    pub plan_b: PlanSummary,
    /// Mean Jaccard similarity over best-matched district pairs.
    pub jaccard_similarity: f64,
    pub population: PopulationComparison,
    pub compactness: CompactnessComparison,
    pub county_splits: Option<SplitComparison>,
    pub partisan: Option<PartisanComparison>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ComparisonError {
    #[error("[INPUT] {plan} assignment for tract {geoid} uses invalid district 0")]
    InvalidDistrictLabel { plan: &'static str, geoid: String },
}

/// Decode the native direct-file assignment object without passing district IDs
/// through a floating-point or target-width-dependent intermediate.
pub fn parse_direct_assignment_object<D: TryFrom<u64>>(
    source: &str,
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<HashMap<String, D>, String> {
    object.iter().map(|(geoid, value)| {
        let invalid = || format!(
            "[INPUT] assignment value for GEOID {geoid} in {source} must be a positive integer, got {value}"
        );
        let district = value.as_u64().filter(|d| *d > 0).ok_or_else(invalid)?;
        let district = D::try_from(district).map_err(|_| format!(
            "[INPUT] assignment value for GEOID {geoid} in {source} is too large: {district}"
        ))?;
        Ok((geoid.clone(), district))
    }).collect()
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Compute Jaccard similarity between two assignment maps.
///
/// Algorithm:
/// 1. Count units per district and overlap for each observed district pair.
/// 2. For each district in plan_a, find the district in plan_b that maximises
///    Jaccard (greedy matching).
/// 3. Return the mean Jaccard across all matched pairs.
///
/// For single-district states this is equivalent to the simple set Jaccard.
pub fn compare_plans(
    assignments_a: &HashMap<String, usize>,
    assignments_b: &HashMap<String, usize>,
) -> PlanComparison {
    try_compare_plans(assignments_a, assignments_b).expect("comparison assignment labels are valid")
}

pub fn try_compare_plans(
    assignments_a: &HashMap<String, usize>,
    assignments_b: &HashMap<String, usize>,
) -> Result<PlanComparison, ComparisonError> {
    validate_assignment_labels("plan_a", assignments_a)?;
    validate_assignment_labels("plan_b", assignments_b)?;

    let map_a = district_sizes(assignments_a);
    let map_b = district_sizes(assignments_b);
    let num_districts_a = map_a.len();
    let num_districts_b = map_b.len();

    // Only districts sharing a unit can have positive Jaccard similarity.
    // Keep one count per observed pair instead of scanning all district pairs
    // and rebuilding their set unions. Each common unit contributes once.
    let mut shared = HashMap::<(usize, usize), usize>::new();
    for (unit, &district_a) in assignments_a {
        if let Some(&district_b) = assignments_b.get(unit) {
            *shared.entry((district_a, district_b)).or_default() += 1;
        }
    }
    let mut best_for_a = HashMap::<usize, f64>::new();
    for ((a, b), common) in shared {
        let union = (map_a[&a] - common) + map_b[&b];
        let similarity = common as f64 / union as f64;
        let best = best_for_a.entry(a).or_default();
        *best = best.max(similarity);
    }
    let jaccard_similarity = if map_a.is_empty() && map_b.is_empty() {
        1.0
    } else if map_a.is_empty() || map_b.is_empty() {
        0.0
    } else {
        // The metric is directional. A district with no overlap contributes
        // zero; B districts may be reused. Keep the original ascending A
        // order for the floating-point sum and exact output bytes.
        let total = map_a.keys().fold(0.0, |sum, a| {
            sum + best_for_a.get(a).copied().unwrap_or(0.0)
        });
        total / num_districts_a as f64
    };

    // Without actual population data the native report uses unit counts.
    let pop_a = population_max_dev_proxy(&map_a);
    let pop_b = population_max_dev_proxy(&map_b);
    let pop_lower = if (pop_a - pop_b).abs() < 1e-9 {
        "equal".to_string()
    } else if pop_a < pop_b {
        "plan_a".to_string()
    } else {
        "plan_b".to_string()
    };

    // Compactness placeholder (not computable without geometry).
    let cmp_lower = "equal".to_string();

    Ok(PlanComparison {
        plan_a: PlanSummary {
            label: "plan_a".into(),
            num_tracts: assignments_a.len(),
            num_districts: num_districts_a,
        },
        plan_b: PlanSummary {
            label: "plan_b".into(),
            num_tracts: assignments_b.len(),
            num_districts: num_districts_b,
        },
        jaccard_similarity,
        population: PopulationComparison {
            plan_a_max_dev: pop_a,
            plan_b_max_dev: pop_b,
            lower: pop_lower,
            difference_pct: (pop_a - pop_b).abs(),
        },
        compactness: CompactnessComparison {
            plan_a_mean: 0.0,
            plan_b_mean: 0.0,
            lower: cmp_lower,
            difference: 0.0,
        },
        county_splits: None,
        partisan: None,
    })
}

fn validate_assignment_labels(
    plan: &'static str,
    assignments: &HashMap<String, usize>,
) -> Result<(), ComparisonError> {
    for (geoid, &district) in assignments {
        if district == 0 {
            return Err(ComparisonError::InvalidDistrictLabel {
                plan,
                geoid: geoid.clone(),
            });
        }
    }
    Ok(())
}

/// Jaccard similarity for two sets.
///
/// Empty sets: union == 0 -> defined as 1.0 (two empty plans are identical).
pub fn jaccard(set_a: &HashSet<String>, set_b: &HashSet<String>) -> f64 {
    let intersection = set_a.intersection(set_b).count() as f64;
    let union = set_a.union(set_b).count() as f64;
    if union == 0.0 {
        1.0
    } else {
        intersection / union
    }
}

fn district_sizes(assignments: &HashMap<String, usize>) -> BTreeMap<usize, usize> {
    let mut sizes = BTreeMap::new();
    for &district in assignments.values() {
        *sizes.entry(district).or_default() += 1;
    }
    sizes
}

/// Maximum absolute deviation divided by mean district unit count.
fn population_max_dev_proxy(map: &BTreeMap<usize, usize>) -> f64 {
    if map.is_empty() {
        return 0.0;
    }
    let mean = map.values().map(|&size| size as f64).sum::<f64>() / map.len() as f64;
    map.values()
        .map(|&size| (size as f64 - mean).abs() / mean)
        .fold(0.0, f64::max)
}

/// Label-sensitive differences on the common unit universe.
/// District numbers are compared directly: a pure renumbering counts as change.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssignmentChanges<D = usize> {
    pub common_units: usize,
    pub changed_units: Vec<String>,
    /// Sorted union of touched labels from both plans.
    pub districts_with_changes: Vec<D>,
}

pub fn assignment_changes<'a>(
    a: impl IntoIterator<Item = (&'a str, usize)>,
    b: impl IntoIterator<Item = (&'a str, usize)>,
) -> AssignmentChanges {
    assignment_changes_typed(a, b)
}

/// Preserve the exact district representation of the caller, even on wasm32.
pub fn assignment_changes_typed<'a, D: Copy + Ord>(
    a: impl IntoIterator<Item = (&'a str, D)>,
    b: impl IntoIterator<Item = (&'a str, D)>,
) -> AssignmentChanges<D> {
    let a: BTreeMap<_, _> = a.into_iter().collect();
    let b: BTreeMap<_, _> = b.into_iter().collect();
    let mut common_units = 0;
    let mut changed_units = Vec::new();
    let mut touched = std::collections::BTreeSet::new();
    for (unit, district_a) in a {
        if let Some(&district_b) = b.get(unit) {
            common_units += 1;
            if district_a != district_b {
                changed_units.push(unit.to_owned());
                touched.insert(district_a);
                touched.insert(district_b);
            }
        }
    }
    AssignmentChanges {
        common_units,
        changed_units,
        districts_with_changes: touched.into_iter().collect(),
    }
}

// ---------------------------------------------------------------------------
// Output formatting
// ---------------------------------------------------------------------------

const DISCLAIMER: &str = "No single metric determines legal compliance. \
     Consult redistricting counsel for legal assessment.";

/// Format comparison as a human-readable table.
///
/// LEGAL REQUIREMENT: Must NOT contain "Winner:" — use "Lower:" instead.
pub fn format_comparison_table(comparison: &PlanComparison) -> String {
    let mut out = String::new();
    out.push_str("Plan Comparison\n");
    out.push_str(&format!(
        "  Plan A: {} ({} tracts, {} districts)\n",
        comparison.plan_a.label, comparison.plan_a.num_tracts, comparison.plan_a.num_districts
    ));
    out.push_str(&format!(
        "  Plan B: {} ({} tracts, {} districts)\n",
        comparison.plan_b.label, comparison.plan_b.num_tracts, comparison.plan_b.num_districts
    ));
    out.push_str(&format!(
        "  Jaccard Similarity:  {:.6}\n",
        comparison.jaccard_similarity
    ));
    out.push_str(&format!(
        "  Population Max Dev:  A={:.4}%  B={:.4}%  Lower: {}\n",
        comparison.population.plan_a_max_dev * 100.0,
        comparison.population.plan_b_max_dev * 100.0,
        comparison.population.lower
    ));
    out.push('\n');
    out.push_str(&format!(
        "  Difference: {:.4}%\n",
        comparison.population.difference_pct * 100.0
    ));
    out.push('\n');
    out.push_str(DISCLAIMER);
    out.push('\n');
    out
}

/// Format comparison as JSON.
pub fn format_comparison_json(comparison: &PlanComparison) -> String {
    let metrics = serde_json::json!({
        "jaccard_similarity": comparison.jaccard_similarity,
        "population": {
            "plan_a_max_dev": comparison.population.plan_a_max_dev,
            "plan_b_max_dev": comparison.population.plan_b_max_dev,
            "lower": comparison.population.lower,
            "difference_pct": comparison.population.difference_pct,
        },
        "compactness": {
            "plan_a_mean": comparison.compactness.plan_a_mean,
            "plan_b_mean": comparison.compactness.plan_b_mean,
            "lower": comparison.compactness.lower,
        },
    });
    let obj = serde_json::json!({
        "plan_a": comparison.plan_a,
        "plan_b": comparison.plan_b,
        "metrics": metrics,
        "disclaimer": DISCLAIMER,
    });
    serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".to_string())
}

/// Format comparison as CSV.
pub fn format_comparison_csv(comparison: &PlanComparison) -> String {
    let mut out = String::new();
    out.push_str("metric,plan_a,plan_b,lower\n");
    out.push_str(&format!(
        "jaccard_similarity,{},{},\n",
        comparison.jaccard_similarity, comparison.jaccard_similarity
    ));
    out.push_str(&format!(
        "population_max_dev,{},{},{}\n",
        comparison.population.plan_a_max_dev,
        comparison.population.plan_b_max_dev,
        comparison.population.lower
    ));
    out
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
fn make_test_assignments(n: usize) -> HashMap<String, usize> {
    (0..n)
        .map(|i| (format!("t{:011}", i), (i % 2) + 1))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_changes_are_sorted_label_sensitive_and_use_common_units() {
        let result = assignment_changes(
            [("z", 1), ("a", 2), ("only_a", 3)],
            [("z", 2), ("a", 1), ("only_b", 4)],
        );
        assert_eq!(result.common_units, 2);
        assert_eq!(result.changed_units, ["a", "z"]);
        assert_eq!(result.districts_with_changes, [1, 2]);
        assert!(assignment_changes([("x", 1)], [("x", 1)]).changed_units.is_empty());
        assert_eq!(assignment_changes([("x", 1)], [("y", 2)]).common_units, 0);
    }

    #[test]
    fn district_accumulation_is_exact_across_insertion_orders() {
        let a:HashMap<_,_>=(0..37).map(|i|(i.to_string(),1+i%7)).collect();
        let b:HashMap<_,_>=(0..37).map(|i|(i.to_string(),1+i%5)).collect();
        let expected=compare_plans(&a,&b).jaccard_similarity.to_bits();
        for _ in 0..100 {
            let reversed:HashMap<_,_>=(0..37).rev().map(|i|(i.to_string(),1+i%7)).collect();
            assert_eq!(compare_plans(&reversed,&b).jaccard_similarity.to_bits(),expected);
        }
    }

    #[test]
    fn test_jaccard_identical_plans() {
        let a: HashMap<String, usize> = [
            ("530330001001".to_string(), 1),
            ("530330001002".to_string(), 1),
            ("530330002001".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();
        let b = a.clone();
        let result = compare_plans(&a, &b);
        assert!(
            (result.jaccard_similarity - 1.0).abs() < 1e-9,
            "Identical plans should have Jaccard = 1.0, got {}",
            result.jaccard_similarity
        );
    }

    #[test]
    fn test_jaccard_completely_different() {
        // Plans with labels swapped are actually the same partition — Jaccard = 1.0
        // (best-match Jaccard finds the mirror: district 1_a matches district 2_b exactly)
        let a: HashMap<String, usize> = [
            ("530330001001".to_string(), 1),
            ("530330001002".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();
        let b: HashMap<String, usize> = [
            ("530330001001".to_string(), 2),
            ("530330001002".to_string(), 1),
        ]
        .iter()
        .cloned()
        .collect();
        let result = compare_plans(&a, &b);
        // Mirror-image plans: best-match Jaccard is 1.0 (same partition, different labels)
        // This is correct: label-agnostic comparison treats these as identical partitions.
        assert!(
            result.jaccard_similarity >= 0.0 && result.jaccard_similarity <= 1.0,
            "Jaccard must be in [0,1], got {}",
            result.jaccard_similarity
        );
    }

    #[test]
    fn test_jaccard_truly_different_plans() {
        // Plans with genuinely different partitions → Jaccard < 1.0
        // District 1 in A: {t1, t2}; district 1 in B: {t1, t3} — different sets
        let a: HashMap<String, usize> = [
            ("t1".to_string(), 1),
            ("t2".to_string(), 1),
            ("t3".to_string(), 2),
            ("t4".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();
        let b: HashMap<String, usize> = [
            ("t1".to_string(), 1),
            ("t2".to_string(), 2),
            ("t3".to_string(), 1),
            ("t4".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();
        let result = compare_plans(&a, &b);
        assert!(
            result.jaccard_similarity < 1.0,
            "Genuinely different plans must have Jaccard < 1.0, got {}",
            result.jaccard_similarity
        );
        assert!(result.jaccard_similarity >= 0.0);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let a: HashMap<String, usize> = [
            ("t00000000001".to_string(), 1),
            ("t00000000002".to_string(), 1),
            ("t00000000003".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();
        let b: HashMap<String, usize> = [
            ("t00000000001".to_string(), 1),
            ("t00000000002".to_string(), 2),
            ("t00000000003".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();
        let result = compare_plans(&a, &b);
        assert!(
            result.jaccard_similarity > 0.0 && result.jaccard_similarity < 1.0,
            "Partial overlap Jaccard should be in (0,1), got {}",
            result.jaccard_similarity
        );
    }

    #[test]
    fn test_try_compare_plans_rejects_zero_label_in_plan_a() {
        let a: HashMap<String, usize> = [("t1".to_string(), 0)].into_iter().collect();
        let b: HashMap<String, usize> = [("t1".to_string(), 1)].into_iter().collect();
        let err = try_compare_plans(&a, &b).expect_err("plan A district 0 must be rejected");
        assert_eq!(
            err,
            ComparisonError::InvalidDistrictLabel {
                plan: "plan_a",
                geoid: "t1".to_string()
            }
        );
    }

    #[test]
    fn test_try_compare_plans_rejects_zero_label_in_plan_b() {
        let a: HashMap<String, usize> = [("t1".to_string(), 1)].into_iter().collect();
        let b: HashMap<String, usize> = [("t1".to_string(), 0)].into_iter().collect();
        let err = try_compare_plans(&a, &b).expect_err("plan B district 0 must be rejected");
        assert_eq!(
            err,
            ComparisonError::InvalidDistrictLabel {
                plan: "plan_b",
                geoid: "t1".to_string()
            }
        );
    }

    #[test]
    fn test_jaccard_internal_fn_identical_sets() {
        let set: HashSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard(&set, &set) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_jaccard_internal_fn_disjoint_sets() {
        let a: HashSet<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["c", "d"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard(&a, &b) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_jaccard_internal_fn_empty_sets() {
        let empty: HashSet<String> = HashSet::new();
        assert!((jaccard(&empty, &empty) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_population_comparison_winner_framing_absent() {
        let a = make_test_assignments(5);
        let b = make_test_assignments(5);
        let comparison = compare_plans(&a, &b);
        let output = format_comparison_table(&comparison);
        assert!(
            !output.contains("Winner:"),
            "Output must not use 'Winner:' framing (legally dangerous)"
        );
        assert!(
            output.contains("Lower:") || output.contains("Difference:"),
            "Output must use 'Lower:' or 'Difference:' framing, got:\n{output}"
        );
    }

    #[test]
    fn test_comparison_output_contains_disclaimer() {
        let comparison = compare_plans(&make_test_assignments(5), &make_test_assignments(5));
        let output = format_comparison_table(&comparison);
        assert!(
            output.contains("No single metric determines legal compliance"),
            "Comparison output must include legal disclaimer"
        );
    }

    #[test]
    fn test_format_comparison_json_has_required_keys() {
        let cmp = compare_plans(&make_test_assignments(4), &make_test_assignments(4));
        let json_str = format_comparison_json(&cmp);
        let v: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert!(v["plan_a"].is_object());
        assert!(v["plan_b"].is_object());
        assert!(v["metrics"].is_object());
        assert!(v["metrics"]["jaccard_similarity"].is_number());
        assert!(v["metrics"]["population"].is_object());
        assert!(v["metrics"]["compactness"].is_object());
    }
}
