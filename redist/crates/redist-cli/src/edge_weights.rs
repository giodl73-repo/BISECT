/// Composable edge weight pipeline.
///
/// Each `EdgeWeighter` is an independent transformation on a `HashMap<(usize,usize), f64>`.
/// Build a `ComposedWeighter` by pushing steps; call `.apply()` to get final weights.
/// To add a new signal: implement `EdgeWeighter`, push an instance — no other files change.
use std::collections::HashMap;

pub type EdgeMap = HashMap<(usize, usize), f64>;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

pub trait EdgeWeighter: Send + Sync {
    /// Transform the current edge map. Receives the map produced by previous
    /// steps (empty on the first step) and returns the modified map.
    fn apply(&self, weights: EdgeMap) -> EdgeMap;
}

// ---------------------------------------------------------------------------
// Composer
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct ComposedWeighter {
    steps: Vec<Box<dyn EdgeWeighter>>,
}

impl ComposedWeighter {
    pub fn new() -> Self { Self::default() }

    pub fn push<W: EdgeWeighter + 'static>(mut self, w: W) -> Self {
        self.steps.push(Box::new(w));
        self
    }

    pub fn apply(&self) -> EdgeMap {
        self.steps.iter().fold(EdgeMap::new(), |w, step| step.apply(w))
    }

    pub fn is_empty(&self) -> bool { self.steps.is_empty() }
}

// ---------------------------------------------------------------------------
// Step 1 — Geographic boundary lengths (base signal)
// ---------------------------------------------------------------------------

/// Replaces whatever came before with the precomputed TIGER boundary lengths.
/// Always the first step for MEC-based algorithms. Skip for unweighted.
pub struct GeographicWeighter {
    pub lengths: EdgeMap,
}

impl GeographicWeighter {
    pub fn from_map(lengths: EdgeMap) -> Self { Self { lengths } }
}

impl EdgeWeighter for GeographicWeighter {
    fn apply(&self, _: EdgeMap) -> EdgeMap { self.lengths.clone() }
}

// ---------------------------------------------------------------------------
// Step 2 — Governmental subdivision stickiness (B.10)
// ---------------------------------------------------------------------------

/// Makes intra-jurisdiction edges more expensive to cut, so the algorithm
/// prefers to cut along existing governmental lines.
///
/// `w' = w × (1 + alpha_county × same_county(u,v)
///              + alpha_mcd    × same_mcd(u,v)
///              + alpha_place  × same_place(u,v)
///              + alpha_vtd    × same_vtd(u,v))`
///
/// Each alpha independently controls one level. alpha = 0 means that level
/// has no effect. Add levels without changing any other code.
pub struct SubdivisionWeighter {
    county: Vec<Option<String>>,  // county FIPS per vertex — from GEOID[0..5]
    mcd:    Vec<Option<String>>,  // county subdivision FIPS — from cousub file
    place:  Vec<Option<String>>,  // incorporated place FIPS — from place file
    vtd:    Vec<Option<String>>,  // voting district FIPS — from vtd file

    pub alpha_county: f64,
    pub alpha_mcd:    f64,
    pub alpha_place:  f64,
    pub alpha_vtd:    f64,
}

impl SubdivisionWeighter {
    /// Phase 1: county only, derived free from GEOID.
    pub fn county_only(
        index_to_geoid: &HashMap<usize, String>,
        n_vertices: usize,
        alpha_county: f64,
    ) -> Self {
        let mut county = vec![None; n_vertices];
        for (&idx, geoid) in index_to_geoid {
            if geoid.len() >= 5 && idx < n_vertices {
                county[idx] = Some(geoid[..5].to_string());
            }
        }
        Self {
            county,
            mcd:   vec![None; n_vertices],
            place: vec![None; n_vertices],
            vtd:   vec![None; n_vertices],
            alpha_county,
            alpha_mcd:   0.0,
            alpha_place: 0.0,
            alpha_vtd:   0.0,
        }
    }

    /// Phase 2+: full hierarchy from pre-loaded TIGER join data.
    pub fn full(
        county: Vec<Option<String>>,
        mcd:    Vec<Option<String>>,
        place:  Vec<Option<String>>,
        vtd:    Vec<Option<String>>,
        alpha_county: f64,
        alpha_mcd:    f64,
        alpha_place:  f64,
        alpha_vtd:    f64,
    ) -> Self {
        Self { county, mcd, place, vtd, alpha_county, alpha_mcd, alpha_place, alpha_vtd }
    }

    fn bonus(&self, u: usize, v: usize) -> f64 {
        let mut b = 0.0;
        if self.alpha_county > 1e-10 {
            if same(&self.county, u, v) { b += self.alpha_county; }
        }
        if self.alpha_mcd > 1e-10 {
            if same(&self.mcd, u, v) { b += self.alpha_mcd; }
        }
        if self.alpha_place > 1e-10 {
            if same(&self.place, u, v) { b += self.alpha_place; }
        }
        if self.alpha_vtd > 1e-10 {
            if same(&self.vtd, u, v) { b += self.alpha_vtd; }
        }
        b
    }
}

impl EdgeWeighter for SubdivisionWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights.into_iter().map(|((u, v), w)| {
            let b = self.bonus(u, v);
            ((u, v), if b > 1e-10 { w * (1.0 + b) } else { w })
        }).collect()
    }
}

fn same(tbl: &[Option<String>], u: usize, v: usize) -> bool {
    match (tbl.get(u), tbl.get(v)) {
        (Some(Some(a)), Some(Some(b))) => a == b,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Step 3 — Partisan signal (two variants)
// ---------------------------------------------------------------------------

/// **Override** variant (matches historic B.2 behaviour): builds partisan
/// weights *from scratch*, ignoring any geographic base.  Same-lean edges
/// get a high weight; cross-lean edges get a low weight.  Equivalent to the
/// old `build_partisan_weights` in `redist_core`.  Use for existing
/// `partisan-weighted` / `proportional` modes.
pub struct PartisanOverrideWeighter {
    pub edges:         Vec<(usize, usize)>,
    pub dem_shares:    Vec<f64>,
    pub dem_threshold: f64,
    pub rep_threshold: f64,
}

impl PartisanOverrideWeighter {
    pub fn new(
        edges: Vec<(usize, usize)>,
        dem_shares: Vec<f64>,
        dem_threshold: f64,
        rep_threshold: f64,
    ) -> Self {
        Self { edges, dem_shares, dem_threshold, rep_threshold }
    }
}

impl EdgeWeighter for PartisanOverrideWeighter {
    fn apply(&self, _: EdgeMap) -> EdgeMap {
        redist_core::build_partisan_weights(
            &self.edges, &self.dem_shares,
            self.dem_threshold, self.rep_threshold,
        )
    }
}

/// **Augment** variant (new composable behaviour): multiplies the existing
/// geographic base.  Cross-partisan edges become 10× cheaper to cut;
/// same-lean edges are unchanged.  Intended for future research (e.g. a paper
/// that combines geographic + partisan signals).
pub struct PartisanAugmentWeighter {
    dem_shares:    Vec<f64>,
    dem_threshold: f64,
    rep_threshold: f64,
}

impl PartisanAugmentWeighter {
    pub fn new(dem_shares: Vec<f64>, dem_threshold: f64, rep_threshold: f64) -> Self {
        Self { dem_shares, dem_threshold, rep_threshold }
    }
}

/// Deprecated name kept for any code that referenced `PartisanWeighter`
/// before the override/augment split.  Resolves to the augment variant.
pub type PartisanWeighter = PartisanAugmentWeighter;

impl EdgeWeighter for PartisanAugmentWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights.into_iter().map(|((u, v), w)| {
            let du = self.dem_shares.get(u).copied().unwrap_or(0.5);
            let dv = self.dem_shares.get(v).copied().unwrap_or(0.5);
            let strong_d_u = du >= self.dem_threshold;
            let strong_r_u = du <= self.rep_threshold;
            let strong_d_v = dv >= self.dem_threshold;
            let strong_r_v = dv <= self.rep_threshold;
            // Cross-partisan edges: one strongly D, other strongly R → easy to cut
            let factor = if (strong_d_u && strong_r_v) || (strong_r_u && strong_d_v) {
                0.1  // much cheaper to cut across partisan boundary
            } else {
                1.0
            };
            ((u, v), w * factor)
        }).collect()
    }
}

// ---------------------------------------------------------------------------
// Step 4 — Minority / VRA signal (two variants)
// ---------------------------------------------------------------------------

/// **Override** variant (matches historic VRA behaviour): builds minority
/// weights *from scratch* using `redist_core::build_vra_edge_weights`.
/// Equivalent to the old `MetisVra` dispatch.  Use for existing `metis-vra`
/// mode.
pub struct MinorityOverrideWeighter {
    pub edges:          Vec<(usize, usize)>,
    pub minority_fracs: Vec<f64>,
    pub threshold:      f64,
}

impl MinorityOverrideWeighter {
    pub fn new(edges: Vec<(usize, usize)>, minority_fracs: Vec<f64>, threshold: f64) -> Self {
        Self { edges, minority_fracs, threshold }
    }
}

impl EdgeWeighter for MinorityOverrideWeighter {
    fn apply(&self, _: EdgeMap) -> EdgeMap {
        redist_core::build_vra_edge_weights(&self.edges, &self.minority_fracs, self.threshold)
    }
}

/// **Augment** variant (new composable behaviour): multiplies the existing
/// geographic base.  Same-minority edges get 2×; cross-minority edges get
/// 0.5×.  Intended for future research combining geographic + VRA signals.
pub struct MinorityAugmentWeighter {
    minority_fracs: Vec<f64>,
    threshold: f64,
}

impl MinorityAugmentWeighter {
    pub fn new(minority_fracs: Vec<f64>, threshold: f64) -> Self {
        Self { minority_fracs, threshold }
    }
}

/// Deprecated name kept for backward compat.  Resolves to augment variant.
pub type MinorityWeighter = MinorityAugmentWeighter;

impl EdgeWeighter for MinorityAugmentWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights.into_iter().map(|((u, v), w)| {
            let mu = self.minority_fracs.get(u).copied().unwrap_or(0.0);
            let mv = self.minority_fracs.get(v).copied().unwrap_or(0.0);
            let high_u = mu >= self.threshold;
            let high_v = mv >= self.threshold;
            let factor = if high_u == high_v { 2.0 } else { 0.5 };
            ((u, v), w * factor)
        }).collect()
    }
}

// ---------------------------------------------------------------------------
// Step 5 — COI (Community of Interest) file-based weights
// ---------------------------------------------------------------------------

/// Applies per-vertex COI weights loaded from a JSON file.
/// Higher COI weight = stronger preference to keep tract with neighbours.
pub struct CoiWeighter {
    coi: HashMap<usize, f64>,   // vertex → COI weight (0.0–1.0)
}

impl CoiWeighter {
    pub fn new(coi: HashMap<usize, f64>) -> Self { Self { coi } }
}

impl EdgeWeighter for CoiWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights.into_iter().map(|((u, v), w)| {
            let cu = self.coi.get(&u).copied().unwrap_or(0.0);
            let cv = self.coi.get(&v).copied().unwrap_or(0.0);
            let bonus = (cu + cv) / 2.0;
            ((u, v), w * (1.0 + bonus))
        }).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn edge_map(edges: &[((usize,usize), f64)]) -> EdgeMap {
        edges.iter().cloned().collect()
    }

    #[test]
    fn geographic_weighter_replaces_input() {
        let base = edge_map(&[((0,1), 999.0)]);
        let geo  = GeographicWeighter::from_map(edge_map(&[((0,1), 100.0), ((1,2), 200.0)]));
        let out  = geo.apply(base);
        assert_eq!(out[&(0,1)], 100.0);
        assert_eq!(out[&(1,2)], 200.0);
    }

    #[test]
    fn subdivision_same_county_adds_bonus() {
        let lengths = edge_map(&[((0,1), 100.0), ((1,2), 100.0)]);
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001".to_string() + "000100");
        geoid_map.insert(1usize, "01001".to_string() + "000200"); // same county 01001
        geoid_map.insert(2usize, "01002".to_string() + "000100"); // different county 01002
        let sw = SubdivisionWeighter::county_only(&geoid_map, 3, 5.0);
        let out = sw.apply(lengths);
        // (0,1): same county 01001 → 100 × (1+5) = 600
        assert!((out[&(0,1)] - 600.0).abs() < 1e-9);
        // (1,2): different county → unchanged 100
        assert!((out[&(1,2)] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn subdivision_zero_alpha_no_change() {
        let lengths = edge_map(&[((0,1), 100.0)]);
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string());
        geoid_map.insert(1usize, "01001000200".to_string());
        let sw = SubdivisionWeighter::county_only(&geoid_map, 2, 0.0);
        let out = sw.apply(lengths);
        assert!((out[&(0,1)] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn composed_weighter_chains_steps() {
        // Geographic sets base, subdivision adds bonus for same county
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string());
        geoid_map.insert(1usize, "01001000200".to_string()); // same county
        geoid_map.insert(2usize, "01002000100".to_string()); // diff county

        let geo_map = edge_map(&[((0,1), 100.0), ((1,2), 100.0)]);
        let sw = SubdivisionWeighter::county_only(&geoid_map, 3, 2.0);
        let composer = ComposedWeighter::new()
            .push(GeographicWeighter::from_map(geo_map))
            .push(sw);
        let out = composer.apply();
        assert!((out[&(0,1)] - 300.0).abs() < 1e-9); // 100 × (1+2)
        assert!((out[&(1,2)] - 100.0).abs() < 1e-9); // unchanged
    }

    #[test]
    fn partisan_weighter_cross_partisan_edges_cheaper() {
        let lengths = edge_map(&[((0,1), 1000.0), ((1,2), 1000.0)]);
        // 0: strong D (0.8), 1: strong R (0.2), 2: neutral (0.5)
        let pw = PartisanWeighter::new(vec![0.8, 0.2, 0.5], 0.55, 0.45);
        let out = pw.apply(lengths);
        // (0,1): strong-D vs strong-R → cheaper
        assert!(out[&(0,1)] < 200.0);
        // (1,2): strong-R vs neutral → unchanged
        assert!((out[&(1,2)] - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn empty_composer_returns_empty_map() {
        let out = ComposedWeighter::new().apply();
        assert!(out.is_empty());
    }

    #[test]
    fn multiple_subdivision_levels_additive() {
        let lengths = edge_map(&[((0,1), 100.0)]);
        let county = vec![Some("01001".to_string()), Some("01001".to_string())];
        let mcd    = vec![Some("90210".to_string()), Some("90210".to_string())];
        let place  = vec![None, None];
        let vtd    = vec![None, None];
        let sw = SubdivisionWeighter::full(county, mcd, place, vtd, 2.0, 1.0, 0.0, 0.0);
        let out = sw.apply(lengths);
        // bonus = alpha_county + alpha_mcd = 2 + 1 = 3 → 100 × (1+3) = 400
        assert!((out[&(0,1)] - 400.0).abs() < 1e-9);
    }

    #[test]
    fn partisan_override_replaces_input() {
        // Override variant ignores whatever was in the map before and builds from scratch.
        // Use a non-trivial input to confirm it's truly replaced, not multiplied.
        let existing = edge_map(&[((0,1), 99999.0), ((1,2), 99999.0)]);
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        // All tracts neutral (0.5) → build_partisan_weights produces uniform output.
        let dem_shares = vec![0.5f64, 0.5, 0.5];
        let pw = PartisanOverrideWeighter::new(edges, dem_shares, 0.55, 0.45);
        let out = pw.apply(existing);
        // Values come from redist_core::build_partisan_weights, not from 99999 input.
        for &w in out.values() {
            assert!(w < 99999.0 - 1.0,
                "override must replace input entirely, got {w}");
        }
    }

    #[test]
    fn minority_override_replaces_input() {
        // Override variant ignores the input map.
        let existing = edge_map(&[((0,1), 88888.0), ((1,2), 88888.0)]);
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let minority_fracs = vec![0.1f64, 0.1, 0.1]; // all low — no high-minority tracts
        let mw = MinorityOverrideWeighter::new(edges, minority_fracs, 0.40);
        let out = mw.apply(existing);
        for &w in out.values() {
            assert!(w < 88888.0 - 1.0,
                "override must replace input entirely, got {w}");
        }
    }

    #[test]
    fn minority_augment_boosts_same_minority_edges() {
        let lengths = edge_map(&[((0,1), 100.0), ((1,2), 100.0)]);
        // 0 and 1 are high-minority (0.8 > 0.4 threshold); 2 is not (0.1)
        let mw = MinorityAugmentWeighter::new(vec![0.8, 0.8, 0.1], 0.40);
        let out = mw.apply(lengths);
        // (0,1): both high-minority → ×2
        assert!((out[&(0,1)] - 200.0).abs() < 1e-9,
            "same-minority edge should be 2×, got {}", out[&(0,1)]);
        // (1,2): one high, one low → ×0.5 (easier to cut across minority boundary)
        assert!((out[&(1,2)] - 50.0).abs() < 1e-9,
            "cross-minority edge should be 0.5×, got {}", out[&(1,2)]);
    }

    #[test]
    fn coi_weighter_boosts_high_coi_edges() {
        let lengths = edge_map(&[((0,1), 100.0), ((1,2), 100.0)]);
        let mut coi = HashMap::new();
        coi.insert(0usize, 1.0); // max COI
        coi.insert(1usize, 1.0); // max COI
        // vertex 2 not in map → defaults to 0.0
        let cw = CoiWeighter::new(coi);
        let out = cw.apply(lengths);
        // (0,1): both COI=1.0 → bonus = (1+1)/2 = 1.0 → 100 × (1+1.0) = 200
        assert!((out[&(0,1)] - 200.0).abs() < 1e-9,
            "high-COI edge should be 2×, got {}", out[&(0,1)]);
        // (1,2): COI = (1+0)/2 = 0.5 → 100 × 1.5 = 150
        assert!((out[&(1,2)] - 150.0).abs() < 1e-9,
            "mixed-COI edge should be 1.5×, got {}", out[&(1,2)]);
    }

    #[test]
    fn partisan_augment_leaves_same_lean_edges_unchanged() {
        let lengths = edge_map(&[((0,1), 500.0), ((1,2), 500.0)]);
        // 0 and 1 both strong-D; 2 is strong-R
        let pw = PartisanAugmentWeighter::new(vec![0.8, 0.8, 0.1], 0.55, 0.45);
        let out = pw.apply(lengths);
        // (0,1): both strong-D → factor 1.0, unchanged
        assert!((out[&(0,1)] - 500.0).abs() < 1e-9,
            "same-lean edge should be unchanged, got {}", out[&(0,1)]);
        // (1,2): strong-D vs strong-R → factor 0.1
        assert!((out[&(1,2)] - 50.0).abs() < 1e-9,
            "cross-partisan edge should be 0.1×, got {}", out[&(1,2)]);
    }

    // ── ComposedWeighter additional cases ───────────────────────────────────

    #[test]
    fn composed_weighter_is_empty_true_when_no_steps() {
        assert!(ComposedWeighter::new().is_empty());
    }

    #[test]
    fn composed_weighter_is_empty_false_after_push() {
        let geo = GeographicWeighter::from_map(edge_map(&[]));
        assert!(!ComposedWeighter::new().push(geo).is_empty());
    }

    #[test]
    fn single_step_composer_matches_direct_apply() {
        let geo_map = edge_map(&[((0, 1), 42.0), ((2, 3), 99.0)]);
        let geo = GeographicWeighter::from_map(geo_map.clone());
        let composed = ComposedWeighter::new().push(GeographicWeighter::from_map(geo_map));
        let out = composed.apply();
        assert_eq!(out[&(0, 1)], 42.0);
        assert_eq!(out[&(2, 3)], 99.0);
    }

    #[test]
    fn geographic_weighter_empty_input_empty_output() {
        let geo = GeographicWeighter::from_map(EdgeMap::new());
        let out = geo.apply(edge_map(&[((0, 1), 1.0)]));
        assert!(out.is_empty(), "empty geo lengths must produce empty output");
    }

    // ── SubdivisionWeighter edge cases ──────────────────────────────────────

    #[test]
    fn subdivision_cross_county_no_bonus() {
        let lengths = edge_map(&[((0, 1), 200.0)]);
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string()); // county 01001
        geoid_map.insert(1usize, "01003000100".to_string()); // county 01003
        let sw = SubdivisionWeighter::county_only(&geoid_map, 2, 3.0);
        let out = sw.apply(lengths);
        assert!((out[&(0, 1)] - 200.0).abs() < 1e-9,
            "cross-county edge must be unchanged, got {}", out[&(0, 1)]);
    }

    #[test]
    fn subdivision_same_mcd_adds_mcd_bonus() {
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let county = vec![Some("01001".to_string()), Some("01002".to_string())]; // different county
        let mcd    = vec![Some("11111".to_string()), Some("11111".to_string())]; // same MCD
        let place  = vec![None, None];
        let vtd    = vec![None, None];
        let sw = SubdivisionWeighter::full(county, mcd, place, vtd, 0.0, 2.0, 0.0, 0.0);
        let out = sw.apply(lengths);
        // only MCD bonus applies: 100 × (1 + 2) = 300
        assert!((out[&(0, 1)] - 300.0).abs() < 1e-9,
            "same-MCD bonus must be applied, got {}", out[&(0, 1)]);
    }

    #[test]
    fn subdivision_same_vtd_adds_vtd_bonus() {
        let lengths = edge_map(&[((0, 1), 50.0)]);
        let county = vec![None, None];
        let mcd    = vec![None, None];
        let place  = vec![None, None];
        let vtd    = vec![Some("VTD001".to_string()), Some("VTD001".to_string())];
        let sw = SubdivisionWeighter::full(county, mcd, place, vtd, 0.0, 0.0, 0.0, 4.0);
        let out = sw.apply(lengths);
        // only VTD bonus: 50 × (1 + 4) = 250
        assert!((out[&(0, 1)] - 250.0).abs() < 1e-9,
            "same-VTD bonus must be applied, got {}", out[&(0, 1)]);
    }

    // ── MinorityAugmentWeighter edge cases ──────────────────────────────────

    #[test]
    fn minority_augment_both_below_threshold_boosts_2x() {
        // Both low-minority (same side) → 2× boost
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let mw = MinorityAugmentWeighter::new(vec![0.1, 0.2], 0.40);
        let out = mw.apply(lengths);
        assert!((out[&(0, 1)] - 200.0).abs() < 1e-9,
            "both below threshold → same side → 2×, got {}", out[&(0, 1)]);
    }

    #[test]
    fn minority_augment_exactly_at_threshold_is_high() {
        // Exactly at threshold (0.40) counts as high-minority
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let mw = MinorityAugmentWeighter::new(vec![0.40, 0.40], 0.40);
        let out = mw.apply(lengths);
        // both >= 0.40 → same side → 2×
        assert!((out[&(0, 1)] - 200.0).abs() < 1e-9,
            "exactly at threshold should be high-minority, got {}", out[&(0, 1)]);
    }

    // ── PartisanAugmentWeighter edge cases ──────────────────────────────────

    #[test]
    fn partisan_augment_both_neutral_leaves_unchanged() {
        // Both vertices between rep_threshold and dem_threshold → neither strong
        let lengths = edge_map(&[((0, 1), 300.0)]);
        let pw = PartisanAugmentWeighter::new(vec![0.50, 0.50], 0.55, 0.45);
        let out = pw.apply(lengths);
        assert!((out[&(0, 1)] - 300.0).abs() < 1e-9,
            "neutral-neutral edge must be unchanged, got {}", out[&(0, 1)]);
    }

    #[test]
    fn partisan_augment_both_strong_rep_unchanged() {
        let lengths = edge_map(&[((0, 1), 400.0)]);
        let pw = PartisanAugmentWeighter::new(vec![0.1, 0.2], 0.55, 0.45);
        let out = pw.apply(lengths);
        // Both strong-R → same lean → factor 1.0
        assert!((out[&(0, 1)] - 400.0).abs() < 1e-9,
            "same-lean (both strong-R) should be unchanged, got {}", out[&(0, 1)]);
    }

    // ── Multiple-step compose ordering ──────────────────────────────────────

    #[test]
    fn composed_three_steps_apply_in_order() {
        // Geo sets base to 100. SubdivisionWeighter with same county multiplies by (1+2)=300.
        // A second SubdivisionWeighter with same MCD multiplies by (1+1)=600.
        let geo_map = edge_map(&[((0, 1), 100.0)]);
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string());
        geoid_map.insert(1usize, "01001000200".to_string()); // same county

        let county  = vec![Some("01001".to_string()), Some("01001".to_string())];
        let mcd_v   = vec![Some("AAA".to_string()), Some("AAA".to_string())];
        let place_v = vec![None, None];
        let vtd_v   = vec![None, None];

        let sw1 = SubdivisionWeighter::county_only(&geoid_map, 2, 2.0); // county bonus only
        let sw2 = SubdivisionWeighter::full(county, mcd_v, place_v, vtd_v, 0.0, 1.0, 0.0, 0.0); // MCD bonus only

        let composed = ComposedWeighter::new()
            .push(GeographicWeighter::from_map(geo_map))
            .push(sw1)  // 100 × (1+2) = 300
            .push(sw2); // 300 × (1+1) = 600
        let out = composed.apply();
        assert!((out[&(0, 1)] - 600.0).abs() < 1e-9,
            "three-step compose should give 600, got {}", out[&(0, 1)]);
    }
}
