use std::collections::HashMap;

/// Build partisan edge weights using an adaptive boost formula.
///
/// Mirrors `vra::build_vra_edge_weights` in structure, but the signal is
/// per-unit Democratic vote share (computed from prior cycles) rather than
/// minority population fraction.
///
/// Formula: α = max(3.0, 10.0 × (1.0 − 0.7 × f_partisan))
/// where f_partisan = fraction of units classified as strong-D or strong-R
/// (non-swing). When few units are strongly partisan, the boost is higher to
/// preserve the rare partisan clusters; when most units are strongly partisan,
/// natural clusters already exist and boost is lower.
///
/// Symmetry: edges are boosted only when BOTH endpoints have the same lean
/// (both strong-D, or both strong-R). Mixed edges and swing-involving edges
/// receive no boost. This preserves whatever partisan clusters the geography
/// already contains and lets METIS produce the natural delegation that the
/// underlying distribution supports.
///
/// Asymmetric targeting (pack one party, fragment the other) is intentionally
/// not supported here — that requires additional constraints at the bisection
/// level, not edge-weighting. This module is the neutral, geography-respecting
/// primitive; partisan-gerrymander targeting belongs upstream.
///
/// Args:
///   edges: undirected edge list (u, v); caller's responsibility to dedupe.
///   dem_shares: per-unit Democratic vote share in [0, 1].
///   dem_threshold: units with dem_share ≥ this are strong-D (default 0.55).
///   rep_threshold: units with dem_share ≤ this are strong-R (default 0.45).
///
/// Returns: HashMap mapping (u, v) → α for boosted edges only. Unboosted
/// edges have implicit weight 1.0 and are not included.
pub fn build_partisan_weights(
    edges: &[(usize, usize)],
    dem_shares: &[f64],
    dem_threshold: f64,
    rep_threshold: f64,
) -> HashMap<(usize, usize), f64> {
    if dem_shares.is_empty() {
        return HashMap::new();
    }
    debug_assert!(
        dem_threshold >= rep_threshold,
        "dem_threshold ({}) must be >= rep_threshold ({})",
        dem_threshold, rep_threshold
    );

    // Step 1: compute fraction of units that are strongly partisan (non-swing).
    let n_partisan = dem_shares.iter()
        .filter(|&&s| s >= dem_threshold || s <= rep_threshold)
        .count();
    let f_partisan = n_partisan as f64 / dem_shares.len() as f64;
    let alpha = 3.0_f64.max(10.0 * (1.0 - 0.7 * f_partisan));

    // Step 2: boost only edges where BOTH endpoints share a non-swing lean.
    edges.iter()
        .filter(|&&(u, v)| {
            let s_u = dem_shares.get(u).copied().unwrap_or(0.5);
            let s_v = dem_shares.get(v).copied().unwrap_or(0.5);
            let both_d = s_u >= dem_threshold && s_v >= dem_threshold;
            let both_r = s_u <= rep_threshold && s_v <= rep_threshold;
            both_d || both_r
        })
        .map(|&(u, v)| ((u, v), alpha))
        .collect()
}

// ============================================================================
// E.4: Partisan-Similarity Edge Weights
// ============================================================================
//
// Mechanically distinct from `build_partisan_weights` above. Where mode #2
// boosts edges that share a *strong* lean (both ≥ 0.55 OR both ≤ 0.45),
// mode #3 (E.4) boosts edges based on the *similarity* of the two endpoints'
// dem_shares, regardless of strength.
//
// Formula (per `research/E.4+partisan-similarity-districts/plan.md` §2.2):
//
//     similarity(i, j) = | dem_share[i] - dem_share[j] |     (in [0, 1])
//
//     w(i, j) = alpha  if similarity(i, j) < tau
//               1.0    otherwise
//
// Two tracts at dem_share 0.50 and 0.51 are very similar (boost). Two at
// 0.70 and 0.30 are dissimilar (no boost). The result is that METIS clusters
// politically like-minded voters into the same district — the "partisan lean
// on steroids" mechanism.
//
// Parameters per E.4 plan:
//   alpha (weight multiplier): 1.0, 5.0, 10.0, 25.0, 50.0, 100.0
//   tau (similarity threshold, dem-share-fraction units): 0.10, 0.15, 0.20
//
// This primitive is mode-agnostic: it just produces an edge-weight HashMap.
// Wiring into the CLI (`--partition-mode partisan-similarity`) and into
// `bisection_runner` is the next-session pickup; this commit ships only the
// math primitive with comprehensive tests against the E.4 plan's worked
// examples.
//
// Compatibility note: this function intentionally does NOT use the adaptive
// alpha formula from `build_partisan_weights`. The E.4 plan calls for
// fixed user-supplied alpha values to enable the ablation study; adaptive
// scaling would obscure the parametric sweep.

/// Build E.4 partisan-similarity edge weights.
///
/// Returns `HashMap<(u, v), alpha>` containing only the boosted edges
/// (similarity < tau). Unboosted edges have implicit weight 1.0 and are
/// not in the map (consistent with `build_partisan_weights`).
///
/// Args:
///   edges: undirected edge list `(u, v)`; caller dedupes.
///   dem_shares: per-unit Democratic vote share in `[0, 1]`.
///   alpha: weight multiplier for similar-edge pairs (caller supplies; E.4
///     plan tests 1.0, 5.0, 10.0, 25.0, 50.0, 100.0).
///   tau: similarity threshold in dem-share-fraction units (caller supplies;
///     E.4 plan tests 0.10, 0.15, 0.20). Pairs with `|s_u - s_v| < tau` are
///     boosted; pairs with `|s_u - s_v| >= tau` are not.
pub fn build_partisan_similarity_weights(
    edges: &[(usize, usize)],
    dem_shares: &[f64],
    alpha: f64,
    tau: f64,
) -> HashMap<(usize, usize), f64> {
    if dem_shares.is_empty() || edges.is_empty() {
        return HashMap::new();
    }
    debug_assert!(alpha >= 1.0, "alpha must be >= 1.0 (got {alpha})");
    debug_assert!(tau > 0.0, "tau must be > 0 (got {tau})");

    edges
        .iter()
        .filter_map(|&(u, v)| {
            let s_u = dem_shares.get(u).copied().unwrap_or(0.5);
            let s_v = dem_shares.get(v).copied().unwrap_or(0.5);
            let similarity = (s_u - s_v).abs();
            if similarity < tau {
                Some(((u, v), alpha))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d_d_edge_boosted() {
        // Both endpoints strong-D → edge boosted.
        let dem_shares = vec![0.70, 0.65, 0.30]; // 0,1 strong-D; 2 strong-R
        let edges = vec![(0usize, 1usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(weights.contains_key(&(0, 1)));
    }

    #[test]
    fn test_r_r_edge_boosted() {
        // Both endpoints strong-R → edge boosted.
        let dem_shares = vec![0.30, 0.25, 0.70];
        let edges = vec![(0usize, 1usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(weights.contains_key(&(0, 1)));
    }

    #[test]
    fn test_mixed_edge_not_boosted() {
        // One D, one R → no boost.
        let dem_shares = vec![0.70, 0.30];
        let edges = vec![(0usize, 1usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(!weights.contains_key(&(0, 1)));
    }

    #[test]
    fn test_swing_edge_not_boosted() {
        // Both swing → no boost (preservation goal targets strong clusters).
        let dem_shares = vec![0.50, 0.52];
        let edges = vec![(0usize, 1usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(!weights.contains_key(&(0, 1)));
    }

    #[test]
    fn test_d_swing_edge_not_boosted() {
        // One strong, one swing → no boost.
        let dem_shares = vec![0.70, 0.50];
        let edges = vec![(0usize, 1usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(!weights.contains_key(&(0, 1)));
    }

    #[test]
    fn test_adaptive_formula_polarized_state() {
        // 100% partisan (no swing) → α = max(3.0, 10*(1-0.7)) = 3.0
        let dem_shares = vec![0.70; 50].into_iter()
            .chain(vec![0.30; 50]).collect::<Vec<f64>>();
        let edges: Vec<(usize, usize)> = (0..49).map(|i| (i, i+1)).collect();
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        // All boosted edges (D-D within first 50) should have α = 3.0
        for &alpha in weights.values() {
            assert!((alpha - 3.0).abs() < 1e-9, "polarized state: alpha={alpha}");
        }
    }

    #[test]
    fn test_adaptive_formula_swing_heavy_state() {
        // 20% partisan, 80% swing → α = max(3.0, 10*(1-0.7*0.2)) = 8.6
        let dem_shares: Vec<f64> = (0..100)
            .map(|i| if i < 10 { 0.70 } else if i < 20 { 0.30 } else { 0.50 })
            .collect();
        let edges = vec![(0usize, 1usize)]; // both strong-D
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        let alpha = weights[&(0, 1)];
        let expected = 3.0_f64.max(10.0 * (1.0 - 0.7 * 0.20));
        assert!((alpha - expected).abs() < 1e-9, "swing-heavy: alpha={alpha} expected={expected}");
    }

    #[test]
    fn test_floor_at_3() {
        // Even with 100% strong partisan, alpha never drops below 3.0.
        let dem_shares = vec![0.99; 10];
        let edges = vec![(0usize, 1usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!((weights[&(0, 1)] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_dem_shares() {
        let weights = build_partisan_weights(&[(0, 1)], &[], 0.55, 0.45);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_empty_edges() {
        let dem_shares = vec![0.70; 10];
        let weights = build_partisan_weights(&[], &dem_shares, 0.55, 0.45);
        assert!(weights.is_empty());
    }

    #[test]
    fn test_out_of_bounds_index_treated_as_swing() {
        // Out-of-bounds index falls back to 0.5 (swing) → no boost.
        let dem_shares = vec![0.70, 0.70, 0.70];
        let edges = vec![(0usize, 99usize)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(!weights.contains_key(&(0, 99)));
    }

    #[test]
    fn test_threshold_boundary_inclusive() {
        // dem_share exactly at threshold: included via >= and <=.
        let dem_shares = vec![0.55, 0.55, 0.45, 0.45];
        let edges = vec![(0, 1), (2, 3), (0, 2)];
        let weights = build_partisan_weights(&edges, &dem_shares, 0.55, 0.45);
        assert!(weights.contains_key(&(0, 1)), "exact-D boundary edge boosted");
        assert!(weights.contains_key(&(2, 3)), "exact-R boundary edge boosted");
        assert!(!weights.contains_key(&(0, 2)), "boundary D + boundary R not boosted");
    }

    #[test]
    fn test_symmetric_treatment_of_parties() {
        // A state with 30% strong-D and 30% strong-R produces the same alpha
        // as the mirror state with 30% strong-R and 30% strong-D.
        let n = 100;
        let dem_shares_a: Vec<f64> = (0..n)
            .map(|i| if i < 30 { 0.70 } else if i < 60 { 0.30 } else { 0.50 })
            .collect();
        let dem_shares_b: Vec<f64> = (0..n)
            .map(|i| if i < 30 { 0.30 } else if i < 60 { 0.70 } else { 0.50 })
            .collect();
        let edges = vec![(0usize, 1usize)]; // strong same-party in both
        let w_a = build_partisan_weights(&edges, &dem_shares_a, 0.55, 0.45);
        let w_b = build_partisan_weights(&edges, &dem_shares_b, 0.55, 0.45);
        assert_eq!(w_a[&(0,1)].to_bits(), w_b[&(0,1)].to_bits(),
            "alpha must be identical when D/R labels are swapped");
    }

    // ── E.4 partisan-similarity edge weights ──────────────────────────────────

    #[test]
    fn similarity_boosts_close_pairs() {
        // E.4 plan §2.2 worked example:
        // Tract 1 D+24 (dem_share 0.62), Tract 2 D+16 (dem_share 0.58).
        // similarity = |0.62 - 0.58| = 0.04 < tau=0.15 -> weight = alpha.
        let dem_shares = vec![0.62, 0.58, 0.45];
        let edges = vec![(0usize, 1usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 10.0, 0.15);
        assert!(w.contains_key(&(0, 1)));
        assert!((w[&(0, 1)] - 10.0).abs() < 1e-9);
    }

    #[test]
    fn similarity_skips_dissimilar_pairs() {
        // E.4 plan §2.2 worked example:
        // Tract 1 D+24 (dem_share 0.62), Tract 3 R+10 (dem_share 0.45).
        // similarity = 0.17 >= tau=0.15 -> NOT boosted.
        let dem_shares = vec![0.62, 0.58, 0.45];
        let edges = vec![(0usize, 2usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 10.0, 0.15);
        assert!(!w.contains_key(&(0, 2)));
    }

    #[test]
    fn similarity_threshold_strict() {
        // similarity == tau exactly should NOT be boosted (per spec: < tau, not <=).
        let dem_shares = vec![0.55, 0.40]; // |0.55 - 0.40| = 0.15
        let edges = vec![(0usize, 1usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 5.0, 0.15);
        assert!(!w.contains_key(&(0, 1)),
            "similarity exactly at threshold must not be boosted");
    }

    #[test]
    fn similarity_alpha_passthrough() {
        // Whatever alpha the caller passes is returned verbatim — no adaptive
        // scaling (this is the difference from build_partisan_weights).
        let dem_shares = vec![0.50, 0.51];
        let edges = vec![(0usize, 1usize)];
        for alpha in [1.0, 5.0, 10.0, 25.0, 50.0, 100.0] {
            let w = build_partisan_similarity_weights(&edges, &dem_shares, alpha, 0.10);
            assert!((w[&(0, 1)] - alpha).abs() < 1e-9,
                "alpha={alpha} must be returned verbatim");
        }
    }

    #[test]
    fn similarity_clusters_swing_voters_too() {
        // E.4 differs from build_partisan_weights: it boosts SIMILAR pairs
        // regardless of partisan strength. Two swing tracts at 0.50 and
        // 0.52 are very similar -> boosted.
        let dem_shares = vec![0.50, 0.52, 0.49];
        let edges = vec![(0usize, 1usize), (1usize, 2usize), (0usize, 2usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 10.0, 0.10);
        assert_eq!(w.len(), 3, "all three swing-pair edges should be boosted");
    }

    #[test]
    fn similarity_separates_R_and_D_clusters() {
        // Two D-leaning tracts (0.65, 0.70) and two R-leaning tracts (0.30, 0.35).
        // D-D and R-R edges boosted; D-R edges not.
        let dem_shares = vec![0.65, 0.70, 0.30, 0.35];
        let edges = vec![
            (0usize, 1usize), // D-D similar
            (2usize, 3usize), // R-R similar
            (0usize, 2usize), // D-R far
            (1usize, 3usize), // D-R far
        ];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 10.0, 0.15);
        assert!(w.contains_key(&(0, 1)));
        assert!(w.contains_key(&(2, 3)));
        assert!(!w.contains_key(&(0, 2)));
        assert!(!w.contains_key(&(1, 3)));
    }

    #[test]
    fn similarity_empty_inputs() {
        let w = build_partisan_similarity_weights(&[], &[0.5; 3], 10.0, 0.15);
        assert!(w.is_empty());
        let w = build_partisan_similarity_weights(&[(0, 1)], &[], 10.0, 0.15);
        assert!(w.is_empty());
    }

    #[test]
    fn similarity_out_of_bounds_treated_as_swing() {
        // Out-of-bounds index falls back to dem_share=0.5. An edge to OOB
        // is "similar" to anything in [0.4, 0.6] and dissimilar otherwise
        // (with tau=0.10).
        let dem_shares = vec![0.55, 0.70];
        let edges = vec![(0usize, 99usize), (1usize, 99usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 10.0, 0.10);
        // 0 has dem_share 0.55, OOB falls back to 0.5: |0.55-0.5|=0.05 < 0.10 -> boosted
        assert!(w.contains_key(&(0, 99)));
        // 1 has dem_share 0.70, OOB=0.5: |0.70-0.5|=0.20 >= 0.10 -> NOT boosted
        assert!(!w.contains_key(&(1, 99)));
    }

    #[test]
    fn similarity_alpha_1_produces_no_boost() {
        // alpha=1.0 is the neutral baseline: boost factor is identity.
        // The map will still contain entries (mapping to 1.0), but they're
        // distinguishable from unboosted edges only by being in the map.
        // This is consistent with build_partisan_weights's convention.
        let dem_shares = vec![0.50, 0.52];
        let edges = vec![(0usize, 1usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 1.0, 0.10);
        assert_eq!(w.len(), 1);
        assert!((w[&(0, 1)] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn similarity_extreme_alpha_100() {
        // E.4 plan's extreme configuration: alpha=100, tau=0.05.
        // Captures only very-similar pairs.
        let dem_shares = vec![0.50, 0.53, 0.60];
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let w = build_partisan_similarity_weights(&edges, &dem_shares, 100.0, 0.05);
        assert!(w.contains_key(&(0, 1)), "|0.50-0.53|=0.03 < 0.05 -> boosted");
        assert!(!w.contains_key(&(1, 2)), "|0.53-0.60|=0.07 >= 0.05 -> not boosted");
    }
}
