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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<W: EdgeWeighter + 'static>(mut self, w: W) -> Self {
        self.steps.push(Box::new(w));
        self
    }

    pub fn apply(&self) -> EdgeMap {
        self.steps
            .iter()
            .fold(EdgeMap::new(), |w, step| step.apply(w))
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
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
    pub fn from_map(lengths: EdgeMap) -> Self {
        Self { lengths }
    }
}

impl EdgeWeighter for GeographicWeighter {
    fn apply(&self, _: EdgeMap) -> EdgeMap {
        self.lengths.clone()
    }
}

// ---------------------------------------------------------------------------
// Step 2 — Governmental subdivision stickiness (T.3)
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
    county: Vec<Option<String>>, // county FIPS per vertex — from GEOID[0..5]
    mcd: Vec<Option<String>>,    // county subdivision FIPS — from cousub file
    place: Vec<Option<String>>,  // incorporated place FIPS — from place file
    vtd: Vec<Option<String>>,    // voting district FIPS — from vtd file

    pub alpha_county: f64,
    pub alpha_mcd: f64,
    pub alpha_place: f64,
    pub alpha_vtd: f64,
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
            mcd: vec![None; n_vertices],
            place: vec![None; n_vertices],
            vtd: vec![None; n_vertices],
            alpha_county,
            alpha_mcd: 0.0,
            alpha_place: 0.0,
            alpha_vtd: 0.0,
        }
    }

    /// Phase 2+: full hierarchy from pre-loaded TIGER join data.
    pub fn full(
        county: Vec<Option<String>>,
        mcd: Vec<Option<String>>,
        place: Vec<Option<String>>,
        vtd: Vec<Option<String>>,
        alpha_county: f64,
        alpha_mcd: f64,
        alpha_place: f64,
        alpha_vtd: f64,
    ) -> Self {
        Self {
            county,
            mcd,
            place,
            vtd,
            alpha_county,
            alpha_mcd,
            alpha_place,
            alpha_vtd,
        }
    }

    fn bonus(&self, u: usize, v: usize) -> f64 {
        let mut b = 0.0;
        if self.alpha_county > 1e-10 {
            if same(&self.county, u, v) {
                b += self.alpha_county;
            }
        }
        if self.alpha_mcd > 1e-10 {
            if same(&self.mcd, u, v) {
                b += self.alpha_mcd;
            }
        }
        if self.alpha_place > 1e-10 {
            if same(&self.place, u, v) {
                b += self.alpha_place;
            }
        }
        if self.alpha_vtd > 1e-10 {
            if same(&self.vtd, u, v) {
                b += self.alpha_vtd;
            }
        }
        b
    }
}

impl EdgeWeighter for SubdivisionWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let b = self.bonus(u, v);
                ((u, v), if b > 1e-10 { w * (1.0 + b) } else { w })
            })
            .collect()
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
/// old `build_partisan_weights` in `bisect_core`.  Use for existing
/// `partisan-weighted` / `proportional` modes.
pub struct PartisanOverrideWeighter {
    pub edges: Vec<(usize, usize)>,
    pub dem_shares: Vec<f64>,
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
        Self {
            edges,
            dem_shares,
            dem_threshold,
            rep_threshold,
        }
    }
}

impl EdgeWeighter for PartisanOverrideWeighter {
    fn apply(&self, _: EdgeMap) -> EdgeMap {
        bisect_core::build_partisan_weights(
            &self.edges,
            &self.dem_shares,
            self.dem_threshold,
            self.rep_threshold,
        )
    }
}

/// **Augment** variant (new composable behaviour): multiplies the existing
/// geographic base.  Cross-partisan edges become 10× cheaper to cut;
/// same-lean edges are unchanged.  Intended for future research (e.g. a paper
/// that combines geographic + partisan signals).
pub struct PartisanAugmentWeighter {
    dem_shares: Vec<f64>,
    dem_threshold: f64,
    rep_threshold: f64,
}

impl PartisanAugmentWeighter {
    pub fn new(dem_shares: Vec<f64>, dem_threshold: f64, rep_threshold: f64) -> Self {
        Self {
            dem_shares,
            dem_threshold,
            rep_threshold,
        }
    }
}

/// Deprecated name kept for any code that referenced `PartisanWeighter`
/// before the override/augment split.  Resolves to the augment variant.
pub type PartisanWeighter = PartisanAugmentWeighter;

impl EdgeWeighter for PartisanAugmentWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let du = self.dem_shares.get(u).copied().unwrap_or(0.5);
                let dv = self.dem_shares.get(v).copied().unwrap_or(0.5);
                let strong_d_u = du >= self.dem_threshold;
                let strong_r_u = du <= self.rep_threshold;
                let strong_d_v = dv >= self.dem_threshold;
                let strong_r_v = dv <= self.rep_threshold;
                // Cross-partisan edges: one strongly D, other strongly R → easy to cut
                let factor = if (strong_d_u && strong_r_v) || (strong_r_u && strong_d_v) {
                    0.1 // much cheaper to cut across partisan boundary
                } else {
                    1.0
                };
                ((u, v), w * factor)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Step 4 — Minority / VRA signal (two variants)
// ---------------------------------------------------------------------------

/// **Override** variant (matches historic VRA behaviour): builds minority
/// weights *from scratch* using `bisect_core::build_vra_edge_weights`.
/// Equivalent to the old `MetisVra` dispatch.  Use for existing `metis-vra`
/// mode.
pub struct MinorityOverrideWeighter {
    pub edges: Vec<(usize, usize)>,
    pub minority_fracs: Vec<f64>,
    pub threshold: f64,
}

impl MinorityOverrideWeighter {
    pub fn new(edges: Vec<(usize, usize)>, minority_fracs: Vec<f64>, threshold: f64) -> Self {
        Self {
            edges,
            minority_fracs,
            threshold,
        }
    }
}

impl EdgeWeighter for MinorityOverrideWeighter {
    fn apply(&self, _: EdgeMap) -> EdgeMap {
        bisect_core::build_vra_edge_weights(&self.edges, &self.minority_fracs, self.threshold)
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
        Self {
            minority_fracs,
            threshold,
        }
    }
}

/// Deprecated name kept for backward compat.  Resolves to augment variant.
pub type MinorityWeighter = MinorityAugmentWeighter;

impl EdgeWeighter for MinorityAugmentWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let mu = self.minority_fracs.get(u).copied().unwrap_or(0.0);
                let mv = self.minority_fracs.get(v).copied().unwrap_or(0.0);
                let high_u = mu >= self.threshold;
                let high_v = mv >= self.threshold;
                let factor = if high_u == high_v { 2.0 } else { 0.5 };
                ((u, v), w * factor)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Step 5 — Economic character similarity (M.9/M.1)
// ---------------------------------------------------------------------------

/// Blends existing edge weights by the cosine similarity of adjacent tracts'
/// economic character vectors.
///
/// Formula: `w_new = w * (alpha + (1 - alpha) * sim)`
///
/// - `sim = 1.0` (both tracts have the same economic character) → weight unchanged.
/// - `sim = 0.0` (tracts are economically orthogonal) → weight scaled to `alpha * w`.
///   With the default `alpha = 0.5` this halves the edge, making it easier to cut.
///
/// Requires LODES WAC data. If data is absent, this weighter is not pushed and
/// the pipeline falls back to geographic weights unchanged.
pub struct EconomicCharacterWeighter {
    chars: Vec<crate::lodes::EconChar>, // indexed by node
    alpha: f64,
}

impl EconomicCharacterWeighter {
    pub fn new(chars: Vec<crate::lodes::EconChar>, alpha: f64) -> Self {
        Self { chars, alpha }
    }
}

impl EdgeWeighter for EconomicCharacterWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        let zero = crate::lodes::EconChar::zero();
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let cu = self.chars.get(u).unwrap_or(&zero);
                let cv = self.chars.get(v).unwrap_or(&zero);
                let sim = crate::lodes::cosine_similarity(cu, cv);
                // Blend: alpha keeps existing weight, (1-alpha) scales by similarity.
                // Similar tracts (sim=1): weight unchanged. Dissimilar (sim=0): weight * alpha.
                let w_new = w * (self.alpha + (1.0 - self.alpha) * sim);
                ((u, v), w_new)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Step 5b -- Housing character similarity (M.3)
// ---------------------------------------------------------------------------

/// Blends existing edge weights by ACS housing-character cosine similarity.
pub struct HousingCharacterWeighter {
    chars: Vec<crate::housing::HousingChar>,
    alpha: f64,
}

impl HousingCharacterWeighter {
    pub fn new(chars: Vec<crate::housing::HousingChar>, alpha: f64) -> Self {
        Self { chars, alpha }
    }
}

impl EdgeWeighter for HousingCharacterWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        let neutral = crate::housing::HousingChar::neutral();
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let cu = self.chars.get(u).unwrap_or(&neutral);
                let cv = self.chars.get(v).unwrap_or(&neutral);
                let sim = crate::housing::cosine_similarity(cu, cv);
                ((u, v), w * (self.alpha + (1.0 - self.alpha) * sim))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Step 5c -- Track Y cohesion weights
// ---------------------------------------------------------------------------

/// Multiplies existing edge weights by local cycle/population cohesion factors.
///
/// The factor is computed with unit boundary weights, so the weighter preserves
/// the upstream base signal while adding the Track Y structural signal:
/// cycle-supported edges get stronger, bridge-like edges get weaker, and dense
/// local population neighborhoods contribute more mass.
pub struct CohesionWeighter {
    factors: HashMap<(usize, usize), f64>,
}

impl CohesionWeighter {
    pub fn try_new(
        adjacency: Vec<Vec<usize>>,
        vertex_weights: Vec<i64>,
        params: bisect_core::CohesionParams,
    ) -> Result<Self, String> {
        Self::try_new_with_geography(
            adjacency,
            vertex_weights,
            bisect_core::CohesionGeography::default(),
            params,
        )
    }

    pub fn try_new_with_geography(
        adjacency: Vec<Vec<usize>>,
        vertex_weights: Vec<i64>,
        geography: bisect_core::CohesionGeography,
        params: bisect_core::CohesionParams,
    ) -> Result<Self, String> {
        let graph = bisect_core::Graph::new(adjacency, vertex_weights)
            .map_err(|e| format!("invalid graph: {e}"))?;
        let unit_weights = unit_edge_weights(&graph);
        let terms = bisect_core::cohesion_edge_terms_with_geography(
            &graph,
            &unit_weights,
            &geography,
            params,
        )
        .map_err(|e| e.to_string())?;
        let factors = terms
            .into_iter()
            .map(|term| ((term.u, term.v), term.cohesion_weight))
            .collect();
        Ok(Self { factors })
    }
}

impl EdgeWeighter for CohesionWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let key = canonical_edge(u, v);
                let factor = self.factors.get(&key).copied().unwrap_or(1.0);
                (key, w * factor)
            })
            .collect()
    }
}

fn unit_edge_weights(graph: &bisect_core::Graph) -> EdgeMap {
    graph
        .adjacency
        .iter()
        .enumerate()
        .flat_map(|(u, neighbors)| {
            neighbors
                .iter()
                .copied()
                .filter(move |&v| u < v)
                .map(move |v| ((u, v), 1.0))
        })
        .collect()
}

fn canonical_edge(u: usize, v: usize) -> (usize, usize) {
    if u < v {
        (u, v)
    } else {
        (v, u)
    }
}

// ---------------------------------------------------------------------------
// Step 6 — COI (Community of Interest) file-based weights
// ---------------------------------------------------------------------------

/// Applies per-vertex COI weights loaded from a JSON file.
/// Higher COI weight = stronger preference to keep tract with neighbours.
pub struct CoiWeighter {
    coi: HashMap<usize, f64>, // vertex → COI weight (0.0–1.0)
}

impl CoiWeighter {
    pub fn new(coi: HashMap<usize, f64>) -> Self {
        Self { coi }
    }
}

impl EdgeWeighter for CoiWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let cu = self.coi.get(&u).copied().unwrap_or(0.0);
                let cv = self.coi.get(&v).copied().unwrap_or(0.0);
                let bonus = (cu + cv) / 2.0;
                ((u, v), w * (1.0 + bonus))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Step 7 -- Administrative zone co-membership (M.6)
// ---------------------------------------------------------------------------

/// Boosts edges between tracts that share administrative zones (school districts,
/// utility service territories, etc.).
///
/// Formula: `w_new = w * (1 + alpha * score)`
///
/// where `score = shared_zones / available_zones` in [0.0, 1.0].
///
/// - `score = 1.0` (all zones shared) and `alpha = 1.0` -> `w * 2` (max bond)
/// - `score = 0.0` (no zones shared) -> `w * 1` (unmodified)
///
/// Zone data is loaded from TIGER school district shapefiles and EIA Form 861
/// service territory shapefiles via spatial join (Phase 2). Until Phase 2 is
/// implemented, use the graceful-skip path in `build_edge_weights`.
pub struct ZoneMembershipWeighter {
    /// Per-edge precomputed co-membership score in [0.0, 1.0].
    scores: HashMap<(usize, usize), f64>,
    /// Boost factor (default 1.0). alpha=1.0 means full-shared zones -> 2x weight.
    alpha: f64,
}

impl ZoneMembershipWeighter {
    /// Build from per-node zone vectors. Each node has a `Vec<Option<String>>`
    /// where index = zone_type, value = zone_id (`None` if zone unavailable).
    ///
    /// `score(u,v) = shared / available` where `available` counts zone types where
    /// both nodes have a non-None assignment, and `shared` counts types where both
    /// assignments are equal.
    pub fn from_zone_assignments(
        assignments: &[Vec<Option<String>>],
        edges: &[(usize, usize)],
        alpha: f64,
    ) -> Self {
        let mut scores = HashMap::new();
        for &(u, v) in edges {
            let zu = &assignments[u];
            let zv = &assignments[v];
            let n = zu.len().max(zv.len());
            if n == 0 {
                scores.insert((u, v), 0.0);
                continue;
            }
            let mut shared = 0usize;
            let mut available = 0usize;
            for i in 0..n.min(zu.len()).min(zv.len()) {
                if let (Some(a), Some(b)) = (&zu[i], &zv[i]) {
                    available += 1;
                    if a == b {
                        shared += 1;
                    }
                }
            }
            let score = if available == 0 {
                0.0
            } else {
                shared as f64 / available as f64
            };
            scores.insert((u, v), score);
        }
        Self { scores, alpha }
    }
}

impl EdgeWeighter for ZoneMembershipWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let score = self.scores.get(&(u, v)).copied().unwrap_or(0.0);
                // Additive boost: w * (1 + alpha * score)
                // score=1 (all zones shared) and alpha=1.0 -> w * 2 (max bond)
                // score=0 (no zones shared) -> w * 1 (unmodified)
                let w_new = w * (1.0 + self.alpha * score);
                ((u, v), w_new)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn edge_map(edges: &[((usize, usize), f64)]) -> EdgeMap {
        edges.iter().cloned().collect()
    }

    #[test]
    fn geographic_weighter_replaces_input() {
        let base = edge_map(&[((0, 1), 999.0)]);
        let geo = GeographicWeighter::from_map(edge_map(&[((0, 1), 100.0), ((1, 2), 200.0)]));
        let out = geo.apply(base);
        assert_eq!(out[&(0, 1)], 100.0);
        assert_eq!(out[&(1, 2)], 200.0);
    }

    #[test]
    fn subdivision_same_county_adds_bonus() {
        let lengths = edge_map(&[((0, 1), 100.0), ((1, 2), 100.0)]);
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001".to_string() + "000100");
        geoid_map.insert(1usize, "01001".to_string() + "000200"); // same county 01001
        geoid_map.insert(2usize, "01002".to_string() + "000100"); // different county 01002
        let sw = SubdivisionWeighter::county_only(&geoid_map, 3, 5.0);
        let out = sw.apply(lengths);
        // (0,1): same county 01001 → 100 × (1+5) = 600
        assert!((out[&(0, 1)] - 600.0).abs() < 1e-9);
        // (1,2): different county → unchanged 100
        assert!((out[&(1, 2)] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn subdivision_zero_alpha_no_change() {
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string());
        geoid_map.insert(1usize, "01001000200".to_string());
        let sw = SubdivisionWeighter::county_only(&geoid_map, 2, 0.0);
        let out = sw.apply(lengths);
        assert!((out[&(0, 1)] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn composed_weighter_chains_steps() {
        // Geographic sets base, subdivision adds bonus for same county
        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string());
        geoid_map.insert(1usize, "01001000200".to_string()); // same county
        geoid_map.insert(2usize, "01002000100".to_string()); // diff county

        let geo_map = edge_map(&[((0, 1), 100.0), ((1, 2), 100.0)]);
        let sw = SubdivisionWeighter::county_only(&geoid_map, 3, 2.0);
        let composer = ComposedWeighter::new()
            .push(GeographicWeighter::from_map(geo_map))
            .push(sw);
        let out = composer.apply();
        assert!((out[&(0, 1)] - 300.0).abs() < 1e-9); // 100 × (1+2)
        assert!((out[&(1, 2)] - 100.0).abs() < 1e-9); // unchanged
    }

    #[test]
    fn partisan_weighter_cross_partisan_edges_cheaper() {
        let lengths = edge_map(&[((0, 1), 1000.0), ((1, 2), 1000.0)]);
        // 0: strong D (0.8), 1: strong R (0.2), 2: neutral (0.5)
        let pw = PartisanWeighter::new(vec![0.8, 0.2, 0.5], 0.55, 0.45);
        let out = pw.apply(lengths);
        // (0,1): strong-D vs strong-R → cheaper
        assert!(out[&(0, 1)] < 200.0);
        // (1,2): strong-R vs neutral → unchanged
        assert!((out[&(1, 2)] - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn empty_composer_returns_empty_map() {
        let out = ComposedWeighter::new().apply();
        assert!(out.is_empty());
    }

    #[test]
    fn multiple_subdivision_levels_additive() {
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let county = vec![Some("01001".to_string()), Some("01001".to_string())];
        let mcd = vec![Some("90210".to_string()), Some("90210".to_string())];
        let place = vec![None, None];
        let vtd = vec![None, None];
        let sw = SubdivisionWeighter::full(county, mcd, place, vtd, 2.0, 1.0, 0.0, 0.0);
        let out = sw.apply(lengths);
        // bonus = alpha_county + alpha_mcd = 2 + 1 = 3 → 100 × (1+3) = 400
        assert!((out[&(0, 1)] - 400.0).abs() < 1e-9);
    }

    #[test]
    fn partisan_override_replaces_input() {
        // Override variant ignores whatever was in the map before and builds from scratch.
        // Use a non-trivial input to confirm it's truly replaced, not multiplied.
        let existing = edge_map(&[((0, 1), 99999.0), ((1, 2), 99999.0)]);
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        // All tracts neutral (0.5) → build_partisan_weights produces uniform output.
        let dem_shares = vec![0.5f64, 0.5, 0.5];
        let pw = PartisanOverrideWeighter::new(edges, dem_shares, 0.55, 0.45);
        let out = pw.apply(existing);
        // Values come from bisect_core::build_partisan_weights, not from 99999 input.
        for &w in out.values() {
            assert!(
                w < 99999.0 - 1.0,
                "override must replace input entirely, got {w}"
            );
        }
    }

    #[test]
    fn minority_override_replaces_input() {
        // Override variant ignores the input map.
        let existing = edge_map(&[((0, 1), 88888.0), ((1, 2), 88888.0)]);
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let minority_fracs = vec![0.1f64, 0.1, 0.1]; // all low — no high-minority tracts
        let mw = MinorityOverrideWeighter::new(edges, minority_fracs, 0.40);
        let out = mw.apply(existing);
        for &w in out.values() {
            assert!(
                w < 88888.0 - 1.0,
                "override must replace input entirely, got {w}"
            );
        }
    }

    #[test]
    fn minority_augment_boosts_same_minority_edges() {
        let lengths = edge_map(&[((0, 1), 100.0), ((1, 2), 100.0)]);
        // 0 and 1 are high-minority (0.8 > 0.4 threshold); 2 is not (0.1)
        let mw = MinorityAugmentWeighter::new(vec![0.8, 0.8, 0.1], 0.40);
        let out = mw.apply(lengths);
        // (0,1): both high-minority → ×2
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "same-minority edge should be 2×, got {}",
            out[&(0, 1)]
        );
        // (1,2): one high, one low → ×0.5 (easier to cut across minority boundary)
        assert!(
            (out[&(1, 2)] - 50.0).abs() < 1e-9,
            "cross-minority edge should be 0.5×, got {}",
            out[&(1, 2)]
        );
    }

    #[test]
    fn coi_weighter_boosts_high_coi_edges() {
        let lengths = edge_map(&[((0, 1), 100.0), ((1, 2), 100.0)]);
        let mut coi = HashMap::new();
        coi.insert(0usize, 1.0); // max COI
        coi.insert(1usize, 1.0); // max COI
                                 // vertex 2 not in map → defaults to 0.0
        let cw = CoiWeighter::new(coi);
        let out = cw.apply(lengths);
        // (0,1): both COI=1.0 → bonus = (1+1)/2 = 1.0 → 100 × (1+1.0) = 200
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "high-COI edge should be 2×, got {}",
            out[&(0, 1)]
        );
        // (1,2): COI = (1+0)/2 = 0.5 → 100 × 1.5 = 150
        assert!(
            (out[&(1, 2)] - 150.0).abs() < 1e-9,
            "mixed-COI edge should be 1.5×, got {}",
            out[&(1, 2)]
        );
    }

    // ── EconomicCharacterWeighter L0 tests ──────────────────────────────────

    fn ec(ci: f64, ind: f64, jpr: f64) -> crate::lodes::EconChar {
        crate::lodes::EconChar {
            commercial_intensity: ci,
            industrial_fraction: ind,
            jobs_per_resident: jpr,
        }
    }

    #[test]
    fn econ_weighter_leaves_similar_tracts_unchanged() {
        // Two identical tracts → sim = 1.0 → weight unchanged (alpha=0.5)
        let chars = vec![ec(0.5, 0.2, 3.0), ec(0.5, 0.2, 3.0)];
        let ew = EconomicCharacterWeighter::new(chars, 0.5);
        let weights = edge_map(&[((0, 1), 200.0)]);
        let out = ew.apply(weights);
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "similar tracts (sim=1) should leave weight unchanged, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn econ_weighter_halves_dissimilar_edges() {
        // Orthogonal chars → sim = 0.0 → w_new = w * (0.5 + 0.5*0) = 0.5*w
        let chars = vec![ec(1.0, 0.0, 0.0), ec(0.0, 1.0, 0.0)];
        let ew = EconomicCharacterWeighter::new(chars, 0.5);
        let weights = edge_map(&[((0, 1), 100.0)]);
        let out = ew.apply(weights);
        assert!(
            (out[&(0, 1)] - 50.0).abs() < 1e-9,
            "dissimilar tracts (sim=0, alpha=0.5) should halve weight, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn econ_weighter_preserves_edge_ordering() {
        // Two pairs with same similarity but different base weights — ordering preserved
        let chars = vec![
            ec(0.5, 0.2, 3.0), // node 0
            ec(0.5, 0.2, 3.0), // node 1 — identical to 0
            ec(0.5, 0.2, 3.0), // node 2
            ec(0.5, 0.2, 3.0), // node 3 — identical to 2
        ];
        let ew = EconomicCharacterWeighter::new(chars, 0.5);
        let weights = edge_map(&[((0, 1), 300.0), ((2, 3), 100.0)]);
        let out = ew.apply(weights);
        // Both have sim=1 → unchanged; (0,1) heavier than (2,3)
        assert!(
            out[&(0, 1)] > out[&(2, 3)],
            "edge ordering must be preserved: {} vs {}",
            out[&(0, 1)],
            out[&(2, 3)]
        );
    }

    #[test]
    fn econ_weighter_both_residential_weight_unchanged() {
        // Both residential (zero EconChar) → sim = 1.0 → weight unchanged
        let chars = vec![
            crate::lodes::EconChar::zero(),
            crate::lodes::EconChar::zero(),
        ];
        let ew = EconomicCharacterWeighter::new(chars, 0.5);
        let weights = edge_map(&[((0, 1), 150.0)]);
        let out = ew.apply(weights);
        assert!(
            (out[&(0, 1)] - 150.0).abs() < 1e-9,
            "two residential tracts (both zero) should leave weight unchanged, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn econ_weighter_one_residential_neutral_blend() {
        // One residential, one commercial → sim = 0.5 → w * (0.5 + 0.5*0.5) = w * 0.75
        let chars = vec![crate::lodes::EconChar::zero(), ec(0.8, 0.1, 2.0)];
        let ew = EconomicCharacterWeighter::new(chars, 0.5);
        let weights = edge_map(&[((0, 1), 100.0)]);
        let out = ew.apply(weights);
        assert!(
            (out[&(0, 1)] - 75.0).abs() < 1e-9,
            "one residential + one non-zero (sim=0.5) should give 75.0, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn partisan_augment_leaves_same_lean_edges_unchanged() {
        let lengths = edge_map(&[((0, 1), 500.0), ((1, 2), 500.0)]);
        // 0 and 1 both strong-D; 2 is strong-R
        let pw = PartisanAugmentWeighter::new(vec![0.8, 0.8, 0.1], 0.55, 0.45);
        let out = pw.apply(lengths);
        // (0,1): both strong-D → factor 1.0, unchanged
        assert!(
            (out[&(0, 1)] - 500.0).abs() < 1e-9,
            "same-lean edge should be unchanged, got {}",
            out[&(0, 1)]
        );
        // (1,2): strong-D vs strong-R → factor 0.1
        assert!(
            (out[&(1, 2)] - 50.0).abs() < 1e-9,
            "cross-partisan edge should be 0.1×, got {}",
            out[&(1, 2)]
        );
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
        assert!(
            out.is_empty(),
            "empty geo lengths must produce empty output"
        );
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
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "cross-county edge must be unchanged, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn subdivision_same_mcd_adds_mcd_bonus() {
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let county = vec![Some("01001".to_string()), Some("01002".to_string())]; // different county
        let mcd = vec![Some("11111".to_string()), Some("11111".to_string())]; // same MCD
        let place = vec![None, None];
        let vtd = vec![None, None];
        let sw = SubdivisionWeighter::full(county, mcd, place, vtd, 0.0, 2.0, 0.0, 0.0);
        let out = sw.apply(lengths);
        // only MCD bonus applies: 100 × (1 + 2) = 300
        assert!(
            (out[&(0, 1)] - 300.0).abs() < 1e-9,
            "same-MCD bonus must be applied, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn subdivision_same_vtd_adds_vtd_bonus() {
        let lengths = edge_map(&[((0, 1), 50.0)]);
        let county = vec![None, None];
        let mcd = vec![None, None];
        let place = vec![None, None];
        let vtd = vec![Some("VTD001".to_string()), Some("VTD001".to_string())];
        let sw = SubdivisionWeighter::full(county, mcd, place, vtd, 0.0, 0.0, 0.0, 4.0);
        let out = sw.apply(lengths);
        // only VTD bonus: 50 × (1 + 4) = 250
        assert!(
            (out[&(0, 1)] - 250.0).abs() < 1e-9,
            "same-VTD bonus must be applied, got {}",
            out[&(0, 1)]
        );
    }

    // ── MinorityAugmentWeighter edge cases ──────────────────────────────────

    #[test]
    fn minority_augment_both_below_threshold_boosts_2x() {
        // Both low-minority (same side) → 2× boost
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let mw = MinorityAugmentWeighter::new(vec![0.1, 0.2], 0.40);
        let out = mw.apply(lengths);
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "both below threshold → same side → 2×, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn minority_augment_exactly_at_threshold_is_high() {
        // Exactly at threshold (0.40) counts as high-minority
        let lengths = edge_map(&[((0, 1), 100.0)]);
        let mw = MinorityAugmentWeighter::new(vec![0.40, 0.40], 0.40);
        let out = mw.apply(lengths);
        // both >= 0.40 → same side → 2×
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "exactly at threshold should be high-minority, got {}",
            out[&(0, 1)]
        );
    }

    // ── PartisanAugmentWeighter edge cases ──────────────────────────────────

    #[test]
    fn partisan_augment_both_neutral_leaves_unchanged() {
        // Both vertices between rep_threshold and dem_threshold → neither strong
        let lengths = edge_map(&[((0, 1), 300.0)]);
        let pw = PartisanAugmentWeighter::new(vec![0.50, 0.50], 0.55, 0.45);
        let out = pw.apply(lengths);
        assert!(
            (out[&(0, 1)] - 300.0).abs() < 1e-9,
            "neutral-neutral edge must be unchanged, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn partisan_augment_both_strong_rep_unchanged() {
        let lengths = edge_map(&[((0, 1), 400.0)]);
        let pw = PartisanAugmentWeighter::new(vec![0.1, 0.2], 0.55, 0.45);
        let out = pw.apply(lengths);
        // Both strong-R → same lean → factor 1.0
        assert!(
            (out[&(0, 1)] - 400.0).abs() < 1e-9,
            "same-lean (both strong-R) should be unchanged, got {}",
            out[&(0, 1)]
        );
    }

    // ── ZoneMembershipWeighter L0 tests ─────────────────────────────────────

    #[test]
    fn test_zone_all_shared_doubles_weight() {
        // All zone types shared, alpha=1.0 -> score=1.0 -> w * (1 + 1.0 * 1.0) = w * 2
        let assignments = vec![
            vec![Some("SchoolA".to_string()), Some("Utility1".to_string())],
            vec![Some("SchoolA".to_string()), Some("Utility1".to_string())],
        ];
        let edges = vec![(0usize, 1usize)];
        let weighter = ZoneMembershipWeighter::from_zone_assignments(&assignments, &edges, 1.0);
        let weights = edge_map(&[((0, 1), 100.0)]);
        let out = weighter.apply(weights);
        assert!(
            (out[&(0, 1)] - 200.0).abs() < 1e-9,
            "all zones shared, alpha=1.0 must double weight, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn test_zone_none_shared_unchanged() {
        // No zone types shared -> score=0.0 -> w * (1 + alpha * 0) = w * 1 (unchanged)
        let assignments = vec![
            vec![Some("SchoolA".to_string()), Some("UtilityX".to_string())],
            vec![Some("SchoolB".to_string()), Some("UtilityY".to_string())],
        ];
        let edges = vec![(0usize, 1usize)];
        let weighter = ZoneMembershipWeighter::from_zone_assignments(&assignments, &edges, 1.0);
        let weights = edge_map(&[((0, 1), 150.0)]);
        let out = weighter.apply(weights);
        assert!(
            (out[&(0, 1)] - 150.0).abs() < 1e-9,
            "no zones shared must leave weight unchanged, got {}",
            out[&(0, 1)]
        );
    }

    #[test]
    fn test_zone_half_shared_factor() {
        // 3 of 6 zones shared -> score = 0.5, alpha=1.0 -> w * (1 + 1.0 * 0.5) = w * 1.5
        let a = Some("Same".to_string());
        let b = Some("Diff_u".to_string());
        let c = Some("Diff_v".to_string());
        let assignments = vec![
            vec![
                a.clone(),
                a.clone(),
                a.clone(),
                b.clone(),
                b.clone(),
                b.clone(),
            ],
            vec![
                a.clone(),
                a.clone(),
                a.clone(),
                c.clone(),
                c.clone(),
                c.clone(),
            ],
        ];
        let edges = vec![(0usize, 1usize)];
        let weighter = ZoneMembershipWeighter::from_zone_assignments(&assignments, &edges, 1.0);
        let weights = edge_map(&[((0, 1), 200.0)]);
        let out = weighter.apply(weights);
        assert!(
            (out[&(0, 1)] - 300.0).abs() < 1e-9,
            "3/6 zones shared, alpha=1.0 must give weight * 1.5 = 300.0, got {}",
            out[&(0, 1)]
        );
    }

    // ── CohesionWeighter L0 tests ──────────────────────────────────────────

    #[test]
    fn cohesion_weighter_boosts_cycle_edge_over_bridge_edge() {
        let adjacency = vec![
            vec![1, 3],
            vec![0, 2, 4],
            vec![1, 3],
            vec![0, 2],
            vec![1, 5],
            vec![4],
        ];
        let populations = vec![100, 100, 100, 100, 100, 100];
        let weighter = CohesionWeighter::try_new(
            adjacency,
            populations,
            bisect_core::CohesionParams::default(),
        )
        .expect("cohesion weighter must build for a valid graph");
        let out = weighter.apply(edge_map(&[((0, 1), 10.0), ((4, 5), 10.0)]));

        assert!(
            out[&(0, 1)] > out[&(4, 5)],
            "cycle-supported edges should score above bridge-like edges"
        );
    }

    #[test]
    fn cohesion_weighter_accepts_declared_geography_terms() {
        let adjacency = vec![vec![1], vec![0, 2], vec![1]];
        let populations = vec![100, 100, 100];
        let mut geography = bisect_core::CohesionGeography::default();
        geography.geo_affinity.insert((0, 1), 1.0);
        geography.barrier_penalty.insert((1, 2), 1.0);
        let params = bisect_core::CohesionParams {
            alpha_cycle: 0.0,
            alpha_bridge: 0.0,
            alpha_geo: 0.50,
            alpha_barrier: 0.50,
            ..bisect_core::CohesionParams::default()
        };
        let weighter =
            CohesionWeighter::try_new_with_geography(adjacency, populations, geography, params)
                .expect("cohesion weighter must build with declared geography");
        let out = weighter.apply(edge_map(&[((0, 1), 10.0), ((1, 2), 10.0)]));

        assert!(
            out[&(0, 1)] > out[&(1, 2)],
            "declared corridor edge should score above declared barrier edge"
        );
    }

    #[test]
    fn cohesion_weighter_boosts_dense_core_over_sparse_peer() {
        let adjacency = vec![
            vec![1, 2],
            vec![0, 2],
            vec![0, 1],
            vec![4, 5],
            vec![3, 5],
            vec![3, 4],
        ];
        let populations = vec![1_000, 1_000, 1_000, 10, 10, 10];
        let weighter = CohesionWeighter::try_new(
            adjacency,
            populations,
            bisect_core::CohesionParams::default(),
        )
        .expect("cohesion weighter must build for matched synthetic triangles");
        let out = weighter.apply(edge_map(&[((0, 1), 10.0), ((3, 4), 10.0)]));

        assert!(
            out[&(0, 1)] > out[&(3, 4)],
            "dense local population mass should make matched mesh edges costlier"
        );
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

        let county = vec![Some("01001".to_string()), Some("01001".to_string())];
        let mcd_v = vec![Some("AAA".to_string()), Some("AAA".to_string())];
        let place_v = vec![None, None];
        let vtd_v = vec![None, None];

        let sw1 = SubdivisionWeighter::county_only(&geoid_map, 2, 2.0); // county bonus only
        let sw2 = SubdivisionWeighter::full(county, mcd_v, place_v, vtd_v, 0.0, 1.0, 0.0, 0.0); // MCD bonus only

        let composed = ComposedWeighter::new()
            .push(GeographicWeighter::from_map(geo_map))
            .push(sw1) // 100 × (1+2) = 300
            .push(sw2); // 300 × (1+1) = 600
        let out = composed.apply();
        assert!(
            (out[&(0, 1)] - 600.0).abs() < 1e-9,
            "three-step compose should give 600, got {}",
            out[&(0, 1)]
        );
    }
}
