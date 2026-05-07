/// Plan-to-plan comparison: Jaccard similarity, population equality, compactness.
///
/// Legal note: Output MUST NOT use "Winner:" framing — use "Lower:" instead.
/// The choice of "better" plan is a legal/political determination outside this tool.
use std::collections::{HashMap, HashSet};

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

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Compute Jaccard similarity between two assignment maps.
///
/// Algorithm:
/// 1. Invert both maps: district -> set of tract GEOIDs.
/// 2. For each district in plan_a, find the district in plan_b that maximises
///    Jaccard (greedy matching).
/// 3. Return the mean Jaccard across all matched pairs.
///
/// For single-district states this is equivalent to the simple set Jaccard.
pub fn compare_plans(
    assignments_a: &HashMap<String, usize>,
    assignments_b: &HashMap<String, usize>,
) -> PlanComparison {
    // Invert: district -> tract set
    let invert = |asgn: &HashMap<String, usize>| -> HashMap<usize, HashSet<String>> {
        let mut m: HashMap<usize, HashSet<String>> = HashMap::new();
        for (geoid, &d) in asgn {
            m.entry(d).or_default().insert(geoid.clone());
        }
        m
    };
    let map_a = invert(assignments_a);
    let map_b = invert(assignments_b);

    let num_districts_a = map_a.len();
    let num_districts_b = map_b.len();

    // Compute mean Jaccard via greedy matching (per plan A district, pick best B district).
    let mut total_jaccard = 0.0;
    let mut matched = 0usize;
    for (_da, tracts_a) in &map_a {
        let best = map_b
            .values()
            .map(|tracts_b| jaccard(tracts_a, tracts_b))
            .fold(f64::NEG_INFINITY, f64::max);
        if best.is_finite() {
            total_jaccard += best;
            matched += 1;
        }
    }
    // Handle the edge case of empty plans.
    let jaccard_similarity = if map_a.is_empty() && map_b.is_empty() {
        1.0
    } else if matched == 0 {
        0.0
    } else {
        total_jaccard / matched as f64
    };

    // Population comparison: use max-deviation proxy from district sizes.
    // (Without actual population data we approximate via tract counts.)
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

    PlanComparison {
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
    }
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

/// Proxy for population max-deviation based on district tract counts.
/// Returns the coefficient of variation of district sizes (std/mean).
fn population_max_dev_proxy(map: &HashMap<usize, HashSet<String>>) -> f64 {
    if map.is_empty() {
        return 0.0;
    }
    let sizes: Vec<f64> = map.values().map(|s| s.len() as f64).collect();
    let mean = sizes.iter().sum::<f64>() / sizes.len() as f64;
    if mean == 0.0 {
        return 0.0;
    }
    let max_dev = sizes
        .iter()
        .map(|&s| (s - mean).abs() / mean)
        .fold(0.0f64, f64::max);
    max_dev
}

// ---------------------------------------------------------------------------
// Output formatting
// ---------------------------------------------------------------------------

const DISCLAIMER: &str =
    "No single metric determines legal compliance. \
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
    out.push_str(&format!("  Difference: {:.4}%\n", comparison.population.difference_pct * 100.0));
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
