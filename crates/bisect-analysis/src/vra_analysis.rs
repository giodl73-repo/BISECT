/// VRA majority-minority district analysis.
///
/// Ports vra_utils.py:analyze_mm_districts() to Rust.
/// Written atomically alongside final_assignments in redist-cli::output.
/// This eliminates the vra_mode premature-clear bug class: analysis runs
/// inside the same function that produces the partition.
use std::collections::HashMap;

/// Per-district VRA demographics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VraDistrict {
    pub district: usize,
    /// Fraction of total district population that is minority.
    pub pct_minority: f64,
    /// Fraction Black (non-Hispanic).
    pub pct_black: f64,
    /// Fraction Hispanic/Latino.
    pub pct_hispanic: f64,
    /// True if pct_minority >= 0.50.
    pub is_mm: bool,
}

/// Full VRA analysis result for one state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VraAnalysis {
    pub mm_count: usize,
    pub mm_districts: Vec<usize>,
    pub districts: Vec<VraDistrict>,
}

/// Compute VRA majority-minority district analysis from tract assignments.
///
/// Matches Python vra_utils.py:analyze_mm_districts().
///
/// Args:
///   assignments: tract_index → district_id (1-based)
///   total_pops: total population per tract
///   minority_pops: minority population per tract
///   black_pops: Black (non-Hispanic) population per tract
///   hispanic_pops: Hispanic/Latino population per tract
///   mm_threshold: fraction at or above which a district is majority-minority (default 0.50)
pub fn analyze_mm_districts(
    assignments: &HashMap<usize, usize>,
    total_pops: &[i64],
    minority_pops: &[f64],
    black_pops: &[f64],
    hispanic_pops: &[f64],
    mm_threshold: f64,
) -> VraAnalysis {
    // Aggregate per district
    let mut dist_total: HashMap<usize, i64> = HashMap::new();
    let mut dist_minority: HashMap<usize, f64> = HashMap::new();
    let mut dist_black: HashMap<usize, f64> = HashMap::new();
    let mut dist_hispanic: HashMap<usize, f64> = HashMap::new();

    // Sort by tract index for deterministic floating-point summation order.
    // HashMap iteration is non-deterministic; sorting ensures reproducible
    // pct_minority values across runs (FP addition is not associative).
    let mut sorted_assignments: Vec<(usize, usize)> = assignments
        .iter().map(|(&t, &d)| (t, d)).collect();
    sorted_assignments.sort_unstable_by_key(|&(tract, _)| tract);

    for (tract, dist) in sorted_assignments {
        if tract >= total_pops.len() { continue; }
        *dist_total.entry(dist).or_insert(0) += total_pops[tract];
        *dist_minority.entry(dist).or_insert(0.0) +=
            minority_pops.get(tract).copied().unwrap_or(0.0);
        *dist_black.entry(dist).or_insert(0.0) +=
            black_pops.get(tract).copied().unwrap_or(0.0);
        *dist_hispanic.entry(dist).or_insert(0.0) +=
            hispanic_pops.get(tract).copied().unwrap_or(0.0);
    }

    // Build district list in sorted order
    let mut dist_ids: Vec<usize> = dist_total.keys().copied().collect();
    dist_ids.sort_unstable();

    let mut districts = Vec::with_capacity(dist_ids.len());
    let mut mm_districts = Vec::new();

    for &dist in &dist_ids {
        let total = dist_total[&dist] as f64;
        let minority = *dist_minority.get(&dist).unwrap_or(&0.0);
        let black = *dist_black.get(&dist).unwrap_or(&0.0);
        let hispanic = *dist_hispanic.get(&dist).unwrap_or(&0.0);

        let pct_minority = if total > 0.0 { minority / total } else { 0.0 };
        let pct_black = if total > 0.0 { black / total } else { 0.0 };
        let pct_hispanic = if total > 0.0 { hispanic / total } else { 0.0 };
        // Python vra_utils.py line 236: `is_mm = pct_minority > mm_threshold` (exclusive)
        let is_mm = pct_minority > mm_threshold;

        if is_mm { mm_districts.push(dist); }

        districts.push(VraDistrict { district: dist, pct_minority, pct_black, pct_hispanic, is_mm });
    }

    let mm_count = mm_districts.len();
    VraAnalysis { mm_count, mm_districts, districts }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_assignments() -> HashMap<usize, usize> {
        // 6 tracts: 0-2 → district 1, 3-5 → district 2
        (0..6).map(|i| (i, if i < 3 { 1 } else { 2 })).collect()
    }

    #[test]
    fn test_mm_district_detected() {
        let assignments = simple_assignments();
        let total = vec![1000i64; 6];
        // District 1 tracts: 600 minority each → 60% → MM
        let minority = vec![600.0, 600.0, 600.0, 100.0, 100.0, 100.0];
        let black = vec![500.0, 500.0, 500.0, 80.0, 80.0, 80.0];
        let hisp = vec![100.0; 6];

        let vra = analyze_mm_districts(&assignments, &total, &minority, &black, &hisp, 0.50);
        assert_eq!(vra.mm_count, 1);
        assert!(vra.mm_districts.contains(&1));
        assert!(!vra.mm_districts.contains(&2));
    }

    #[test]
    fn test_no_mm_districts() {
        let assignments = simple_assignments();
        let total = vec![1000i64; 6];
        let minority = vec![300.0; 6]; // 30% — below 50%
        let black = vec![200.0; 6];
        let hisp = vec![100.0; 6];

        let vra = analyze_mm_districts(&assignments, &total, &minority, &black, &hisp, 0.50);
        assert_eq!(vra.mm_count, 0);
        assert!(vra.mm_districts.is_empty());
    }

    #[test]
    fn test_all_mm_districts() {
        let assignments: HashMap<usize, usize> = (0..4).map(|i| (i, i + 1)).collect();
        let total = vec![1000i64; 4];
        let minority = vec![600.0; 4]; // all 60%
        let black = vec![500.0; 4];
        let hisp = vec![100.0; 4];

        let vra = analyze_mm_districts(&assignments, &total, &minority, &black, &hisp, 0.50);
        assert_eq!(vra.mm_count, 4);
        assert_eq!(vra.districts.len(), 4);
    }

    #[test]
    fn test_district_list_sorted() {
        let assignments: HashMap<usize, usize> = vec![(0,3),(1,1),(2,2)].into_iter().collect();
        let total = vec![1000i64; 3];
        let minority = vec![300.0; 3];
        let vra = analyze_mm_districts(&assignments, &total, &minority, &minority, &minority, 0.50);
        let ids: Vec<usize> = vra.districts.iter().map(|d| d.district).collect();
        assert_eq!(ids, vec![1, 2, 3], "districts must be in sorted order");
    }

    #[test]
    fn test_pct_minority_calculation() {
        let assignments: HashMap<usize, usize> = vec![(0,1),(1,1)].into_iter().collect();
        let total = vec![1000i64, 2000i64];
        // Tract 0: 500 minority, Tract 1: 1000 minority
        // District 1: total=3000, minority=1500 → 50.0%
        let minority = vec![500.0, 1000.0];
        let vra = analyze_mm_districts(
            &assignments, &total, &minority, &[0.0; 2], &[0.0; 2], 0.50
        );
        // Python vra_utils.py uses `pct_minority > mm_threshold` (exclusive)
        // Exactly 50% → NOT MM
        assert_eq!(vra.mm_count, 0, "exactly 50% is NOT MM (Python uses >, not >=)");
        let d = &vra.districts[0];
        assert!((d.pct_minority - 0.5).abs() < 1e-9);
        assert!(!d.is_mm, "50.0% must not be is_mm");
    }

    #[test]
    fn test_threshold_boundary_exclusive() {
        // Python vra_utils.py line 236: `is_mm = pct_minority > mm_threshold` (exclusive)
        // So: exactly 50.00% → NOT MM; 50.01% → MM
        let a1: HashMap<usize, usize> = vec![(0,1)].into_iter().collect();
        let total = vec![10000i64];

        let minority_below = vec![4999.0]; // 49.99% → not MM
        let vra_below = analyze_mm_districts(&a1, &total, &minority_below, &[0.0], &[0.0], 0.50);
        assert_eq!(vra_below.mm_count, 0, "49.99% must not be MM");

        let minority_exact = vec![5000.0]; // 50.00% exactly → NOT MM (exclusive >)
        let vra_exact = analyze_mm_districts(&a1, &total, &minority_exact, &[0.0], &[0.0], 0.50);
        assert_eq!(vra_exact.mm_count, 0, "50.00% must NOT be MM (Python uses >, not >=)");

        let minority_above = vec![5001.0]; // 50.01% → MM
        let vra_above = analyze_mm_districts(&a1, &total, &minority_above, &[0.0], &[0.0], 0.50);
        assert_eq!(vra_above.mm_count, 1, "50.01% must be MM");
    }

    #[test]
    fn test_deterministic_across_runs() {
        // Same input always produces same pct_minority (sorted aggregation)
        let mut assignments = HashMap::new();
        for i in 0..100usize {
            assignments.insert(i, if i < 50 { 1 } else { 2 });
        }
        let total = vec![1000i64; 100];
        let minority = vec![600.0f64; 100];
        let r1 = analyze_mm_districts(&assignments, &total, &minority, &[0.0;100], &[0.0;100], 0.50);
        let r2 = analyze_mm_districts(&assignments, &total, &minority, &[0.0;100], &[0.0;100], 0.50);
        for (d1, d2) in r1.districts.iter().zip(r2.districts.iter()) {
            assert_eq!(d1.pct_minority.to_bits(), d2.pct_minority.to_bits(),
                "pct_minority must be bit-identical across runs");
        }
    }

    #[test]
    fn test_out_of_bounds_tract_skipped() {
        let mut assignments = HashMap::new();
        assignments.insert(0usize, 1usize); // valid
        assignments.insert(999usize, 2usize); // out of bounds
        let total = vec![1000i64; 2]; // only tracts 0 and 1
        let minority = vec![300.0; 2];
        let vra = analyze_mm_districts(&assignments, &total, &minority, &[0.0;2], &[0.0;2], 0.50);
        // Tract 999 silently skipped; district 1 has 30% < 50% → not MM
        assert_eq!(vra.mm_count, 0);
    }

    #[test]
    fn test_json_serializes_correctly() {
        let vra = VraAnalysis {
            mm_count: 1,
            mm_districts: vec![2],
            districts: vec![
                VraDistrict { district: 1, pct_minority: 0.30, pct_black: 0.20, pct_hispanic: 0.10, is_mm: false },
                VraDistrict { district: 2, pct_minority: 0.55, pct_black: 0.40, pct_hispanic: 0.15, is_mm: true },
            ],
        };
        let json = serde_json::to_string_pretty(&vra).unwrap();
        let reparsed: VraAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed.mm_count, 1);
        assert_eq!(reparsed.districts[1].is_mm, true);
    }
}
