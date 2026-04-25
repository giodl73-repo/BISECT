use std::collections::HashMap;

/// Build VRA edge weights using the adaptive boost formula.
///
/// Formula: α = max(3.0, 10.0 × (1.0 − 0.7 × f_minority))
/// where f_minority = fraction of tracts with minority_frac ≥ threshold.
///
/// Only minority-to-minority edges (both endpoints above threshold) are boosted.
/// All other edges have implicit weight 1.0 (not included in the returned map).
///
/// This is the single authoritative implementation — both redist-cli and the
/// Python pipeline (via PyO3) call this function.
pub fn build_vra_edge_weights(
    edges: &[(usize, usize)],
    minority_fracs: &[f64],
    threshold: f64,
) -> HashMap<(usize, usize), f64> {
    if minority_fracs.is_empty() {
        return HashMap::new();
    }

    // Step 1: compute state-level minority tract fraction
    let n_minority = minority_fracs.iter().filter(|&&f| f >= threshold).count();
    let f_minority = n_minority as f64 / minority_fracs.len() as f64;
    let alpha = 3.0_f64.max(10.0 * (1.0 - 0.7 * f_minority));

    // Step 2: boost only edges where BOTH endpoints are minority tracts
    edges.iter()
        .filter(|&&(u, v)| {
            minority_fracs.get(u).copied().unwrap_or(0.0) >= threshold
                && minority_fracs.get(v).copied().unwrap_or(0.0) >= threshold
        })
        .map(|&(u, v)| ((u, v), alpha))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_formula_alabama() {
        // Alabama: ~22% of tracts above 40% minority → f ≈ 0.22 → α ≈ 8.46
        let n = 100usize;
        let minority_fracs: Vec<f64> = (0..n)
            .map(|i| if i < 22 { 0.50 } else { 0.20 })
            .collect();
        // All edges connect tract 0 (minority) to tract 1 (minority)
        let edges = vec![(0usize, 1usize)];
        let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);
        let alpha = weights[&(0, 1)];
        // expected: max(3.0, 10.0*(1.0 - 0.7*0.22)) = max(3.0, 10.0*0.846) = 8.46
        assert!((alpha - 8.46).abs() < 0.01, "alpha={alpha}");
    }

    #[test]
    fn test_non_minority_edge_not_boosted() {
        let minority_fracs = vec![0.20, 0.20, 0.60]; // only tract 2 is minority
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);
        assert!(!weights.contains_key(&(0, 1)), "non-minority edge should not be boosted");
        assert!(!weights.contains_key(&(1, 2)), "mixed edge should not be boosted");
    }

    #[test]
    fn test_floor_at_high_minority_density() {
        // All tracts minority → f=1.0 → α = max(3.0, 10*(1-0.7)) = max(3.0, 3.0) = 3.0
        let minority_fracs = vec![0.60; 50];
        let edges: Vec<(usize, usize)> = (0..49).map(|i| (i, i + 1)).collect();
        let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);
        for &alpha in weights.values() {
            assert!((alpha - 3.0).abs() < 1e-9, "floor should be 3.0, got {alpha}");
        }
    }

    #[test]
    fn test_minority_minority_edge_is_boosted() {
        // Both endpoints minority → edge appears in output with correct α
        let minority_fracs = vec![0.60, 0.55, 0.10]; // tracts 0,1 minority; 2 not
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);
        // f_minority = 2/3 → α = max(3.0, 10*(1-0.7*(2/3))) = max(3.0, 5.33) = 5.33
        assert!(weights.contains_key(&(0, 1)), "minority-minority edge must be in output");
        let alpha = weights[&(0, 1)];
        let expected = 3.0_f64.max(10.0 * (1.0 - 0.7 * (2.0 / 3.0)));
        assert!((alpha - expected).abs() < 1e-9, "alpha={alpha} expected={expected}");
        // Mixed edge not in output
        assert!(!weights.contains_key(&(1, 2)));
    }

    #[test]
    fn test_out_of_bounds_index_falls_back_to_zero() {
        // Edge (0, 99) — tract 99 doesn't exist in minority_fracs (len=3)
        // get(99) returns None → unwrap_or(0.0) → 0.0 < threshold → not boosted
        let minority_fracs = vec![0.70, 0.70, 0.70]; // all minority
        let edges = vec![(0usize, 99usize)]; // 99 is out of bounds
        let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);
        assert!(!weights.contains_key(&(0, 99)), "out-of-bounds vertex treated as non-minority");
    }

    #[test]
    fn test_empty_minority_fracs() {
        // Empty fracs → early return, no panic
        let weights = build_vra_edge_weights(&[(0, 1)], &[], 0.40);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_empty_edges() {
        let minority_fracs = vec![0.5; 10];
        let weights = build_vra_edge_weights(&[], &minority_fracs, 0.40);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_all_tracts_below_threshold() {
        // No minority tracts → no edges boosted (but f_minority=0 → α=10.0)
        // Because no edges pass the filter, output is empty
        let minority_fracs = vec![0.10, 0.15, 0.20];
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let weights = build_vra_edge_weights(&edges, &minority_fracs, 0.40);
        assert!(weights.is_empty(), "no minority tracts → no boosted edges");
    }
}
