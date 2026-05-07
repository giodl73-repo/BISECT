/// Composable vertex weight pipeline.
///
/// METIS supports `ncon` balance constraints per vertex (the `vwgt` array).
/// Each `VertexWeighter` contributes one constraint column.
/// `ComposedVertexWeighter` interleaves them into the flat array METIS expects:
///   [w0_v0, w1_v0, ..., w0_v1, w1_v1, ..., w0_vN, w1_vN, ...]
///
/// Config-time: `AlgorithmConfig` holds `Vec<VertexConstraintKind>` — which
/// constraints to apply. This is serialisable and requires no runtime data.
///
/// Runtime: `ComposedVertexWeighter` is built from the spec + loaded graph data
/// (populations from adjacency, areas from TIGER). Same pattern as edge weights.

// ---------------------------------------------------------------------------
// Config-time spec — what constraints to use (no data)
// ---------------------------------------------------------------------------

/// Which vertex balance constraints METIS should enforce.
/// Ordering matters: first entry = first column of vwgt.
/// Add a new constraint here; `build_vertex_weights` handles the data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VertexConstraintKind {
    /// Census population (always present, usually first).
    Population,
    /// TIGER ALAND land area in hectares. Requires TIGER geometry.
    Area,
}

/// Build `ComposedVertexWeighter` from a spec + graph data.
/// Called in `run_single_state` after the adjacency graph is loaded.
pub fn build_vertex_weights(
    constraints: &[VertexConstraintKind],
    populations: &[i64],
    areas_m2: &[f64],
) -> ComposedVertexWeighter {
    let mut c = ComposedVertexWeighter::new();
    for kind in constraints {
        match kind {
            VertexConstraintKind::Population =>
                c = c.push(PopulationWeighter::from_graph(populations.to_vec())),
            VertexConstraintKind::Area =>
                c = c.push(AreaWeighter::from_tiger(areas_m2.to_vec())),
        }
    }
    if c.steps.is_empty() {
        c = c.push(PopulationWeighter::from_graph(populations.to_vec()));
    }
    c
}

pub trait VertexWeighter: Send + Sync {
    fn weights(&self, n_vertices: usize) -> Vec<i64>;
}

// ---------------------------------------------------------------------------
// Composer
// ---------------------------------------------------------------------------

pub struct ComposedVertexWeighter {
    steps: Vec<Box<dyn VertexWeighter>>,
}

impl ComposedVertexWeighter {
    pub fn new() -> Self { Self { steps: vec![] } }

    pub fn push<W: VertexWeighter + 'static>(mut self, w: W) -> Self {
        self.steps.push(Box::new(w));
        self
    }

    /// Number of balance constraints (= ncon for METIS).
    pub fn ncon(&self) -> usize { self.steps.len().max(1) }

    /// Interleaved vertex weights for METIS `vwgt`:
    /// [w0_v0, w1_v0, ..., w0_v1, w1_v1, ...]
    pub fn interleaved(&self, n_vertices: usize) -> Vec<i64> {
        if self.steps.is_empty() {
            return vec![1i64; n_vertices]; // uniform — METIS treats as equal weight
        }
        let cols: Vec<Vec<i64>> = self.steps.iter()
            .map(|s| s.weights(n_vertices))
            .collect();
        (0..n_vertices)
            .flat_map(|v| cols.iter().map(move |col| col[v]))
            .collect()
    }

    pub fn is_default(&self) -> bool { self.steps.len() <= 1 }
}

impl Default for ComposedVertexWeighter {
    fn default() -> Self { Self::new().push(PopulationWeighter::default()) }
}

// ---------------------------------------------------------------------------
// Population (always first — required for all modes)
// ---------------------------------------------------------------------------

/// Standard population balance: each vertex weight = census population.
/// METIS minimises edge-cut subject to equal-population partitions.
pub struct PopulationWeighter {
    pub populations: Vec<i64>,
}

impl Default for PopulationWeighter {
    fn default() -> Self { Self { populations: vec![] } }
}

impl PopulationWeighter {
    pub fn from_graph(populations: Vec<i64>) -> Self { Self { populations } }
}

impl VertexWeighter for PopulationWeighter {
    fn weights(&self, n_vertices: usize) -> Vec<i64> {
        if self.populations.is_empty() {
            vec![1i64; n_vertices]
        } else {
            self.populations.clone()
        }
    }
}

// ---------------------------------------------------------------------------
// Land area (second constraint for AreaSection / B.9)
// ---------------------------------------------------------------------------

/// Land area balance: each vertex weight = ALAND in hectares (m² ÷ 10,000).
/// Scaled to hectares so values fit within METIS's internal i32 range.
/// When composed with `PopulationWeighter`, produces ncon=2 (pop + area).
pub struct AreaWeighter {
    pub areas_m2: Vec<f64>,
}

impl AreaWeighter {
    pub fn from_tiger(areas_m2: Vec<f64>) -> Self { Self { areas_m2 } }
}

impl VertexWeighter for AreaWeighter {
    fn weights(&self, n_vertices: usize) -> Vec<i64> {
        if self.areas_m2.is_empty() {
            vec![1i64; n_vertices]
        } else {
            self.areas_m2.iter()
                .map(|&a| ((a / 10_000.0) as i64).max(1))
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_composer_ncon_is_1() {
        let c = ComposedVertexWeighter::default();
        assert_eq!(c.ncon(), 1);
    }

    #[test]
    fn population_plus_area_ncon_is_2() {
        let c = ComposedVertexWeighter::new()
            .push(PopulationWeighter::from_graph(vec![100, 200, 300]))
            .push(AreaWeighter::from_tiger(vec![10_000.0, 20_000.0, 30_000.0]));
        assert_eq!(c.ncon(), 2);
    }

    #[test]
    fn interleaved_format_correct() {
        // 3 vertices, pop=[10,20,30], area=[1ha,2ha,3ha] (10k/20k/30k m²)
        let c = ComposedVertexWeighter::new()
            .push(PopulationWeighter::from_graph(vec![10, 20, 30]))
            .push(AreaWeighter::from_tiger(vec![10_000.0, 20_000.0, 30_000.0]));
        let out = c.interleaved(3);
        // Expected: [pop_v0, area_v0, pop_v1, area_v1, pop_v2, area_v2]
        assert_eq!(out, vec![10, 1, 20, 2, 30, 3]);
    }

    #[test]
    fn population_only_interleaved_matches_input() {
        let c = ComposedVertexWeighter::new()
            .push(PopulationWeighter::from_graph(vec![100, 200, 300]));
        let out = c.interleaved(3);
        assert_eq!(out, vec![100, 200, 300]);
    }

    #[test]
    fn empty_composer_returns_uniform_weights() {
        let c = ComposedVertexWeighter::new();
        let out = c.interleaved(4);
        assert_eq!(out, vec![1, 1, 1, 1]);
    }

    #[test]
    fn area_weighter_scales_to_hectares() {
        let aw = AreaWeighter::from_tiger(vec![50_000.0, 1_000_000.0]);
        let out = aw.weights(2);
        assert_eq!(out[0], 5);    // 50,000 m² = 5 ha
        assert_eq!(out[1], 100);  // 1,000,000 m² = 100 ha
    }

    #[test]
    fn area_weighter_clamps_zero_to_one() {
        let aw = AreaWeighter::from_tiger(vec![0.0, 500.0]);
        let out = aw.weights(2);
        assert_eq!(out[0], 1); // 0 m² → clamped to 1
        assert_eq!(out[1], 1); // 500 m² = 0 ha → clamped to 1
    }

    // ── ComposedVertexWeighter additional cases ──────────────────────────────

    #[test]
    fn empty_composer_ncon_is_1() {
        // Even with no steps, ncon() must return at least 1 (METIS requires ≥1 constraint)
        let c = ComposedVertexWeighter::new();
        assert_eq!(c.ncon(), 1);
    }

    #[test]
    fn two_step_composer_is_not_default() {
        let c = ComposedVertexWeighter::new()
            .push(PopulationWeighter::from_graph(vec![100, 200]))
            .push(AreaWeighter::from_tiger(vec![10_000.0, 20_000.0]));
        assert!(!c.is_default(), "two-constraint composer must not be flagged as default");
    }

    #[test]
    fn single_step_composer_is_default() {
        let c = ComposedVertexWeighter::new()
            .push(PopulationWeighter::from_graph(vec![100, 200]));
        assert!(c.is_default(), "single-constraint composer must be flagged as default");
    }

    #[test]
    fn interleaved_ncon_2_vertex_ordering() {
        // Verify exact byte-layout: for ncon=2 the output must be
        // [pop_v0, area_v0, pop_v1, area_v1] — METIS requires column-major interleave
        let c = ComposedVertexWeighter::new()
            .push(PopulationWeighter::from_graph(vec![500, 800]))
            .push(AreaWeighter::from_tiger(vec![10_000.0, 30_000.0]));
        let out = c.interleaved(2);
        assert_eq!(out.len(), 4, "ncon=2 × n_vertices=2 = 4 elements");
        assert_eq!(out[0], 500, "v0 population");
        assert_eq!(out[1], 1,   "v0 area = 10000/10000 = 1 ha");
        assert_eq!(out[2], 800, "v1 population");
        assert_eq!(out[3], 3,   "v1 area = 30000/10000 = 3 ha");
    }

    // ── PopulationWeighter edge cases ────────────────────────────────────────

    #[test]
    fn population_weighter_empty_populations_returns_uniform() {
        let pw = PopulationWeighter::from_graph(vec![]);
        let out = pw.weights(3);
        assert_eq!(out, vec![1, 1, 1], "empty populations must return uniform weight=1");
    }

    #[test]
    fn population_weighter_single_vertex() {
        let pw = PopulationWeighter::from_graph(vec![42000]);
        let out = pw.weights(1);
        assert_eq!(out, vec![42000]);
    }

    // ── AreaWeighter edge cases ──────────────────────────────────────────────

    #[test]
    fn area_weighter_empty_areas_returns_uniform() {
        let aw = AreaWeighter::from_tiger(vec![]);
        let out = aw.weights(4);
        assert_eq!(out, vec![1, 1, 1, 1], "empty areas must return uniform weight=1");
    }

    #[test]
    fn area_weighter_very_large_area_fits_in_i64() {
        // Alaska's largest census tract ≈ 200 billion m² — must not overflow i64
        let large = 2e11_f64; // 200,000,000,000 m² = 20,000,000 ha
        let aw = AreaWeighter::from_tiger(vec![large]);
        let out = aw.weights(1);
        let expected = (large / 10_000.0) as i64;
        assert_eq!(out[0], expected, "very large area must be representable as i64");
        assert!(out[0] > 0, "large area must remain positive");
    }

    // ── build_vertex_weights function ────────────────────────────────────────

    #[test]
    fn build_vertex_weights_empty_constraints_defaults_to_population() {
        let pops = vec![100i64, 200, 300];
        let areas: Vec<f64> = vec![];
        let c = build_vertex_weights(&[], &pops, &areas);
        // Empty constraint list → fallback to population
        assert_eq!(c.ncon(), 1);
        let out = c.interleaved(3);
        assert_eq!(out, vec![100, 200, 300]);
    }

    #[test]
    fn build_vertex_weights_population_only() {
        let pops = vec![10i64, 20, 30];
        let c = build_vertex_weights(&[VertexConstraintKind::Population], &pops, &[]);
        assert_eq!(c.ncon(), 1);
        assert_eq!(c.interleaved(3), vec![10, 20, 30]);
    }

    #[test]
    fn build_vertex_weights_area_only_ncon_1() {
        let areas = vec![10_000.0f64, 20_000.0, 30_000.0];
        let c = build_vertex_weights(&[VertexConstraintKind::Area], &[], &areas);
        assert_eq!(c.ncon(), 1);
        let out = c.interleaved(3);
        assert_eq!(out, vec![1, 2, 3]); // 10k/20k/30k m² → 1/2/3 ha
    }

    #[test]
    fn build_vertex_weights_population_and_area_ncon_2() {
        let pops  = vec![100i64, 200];
        let areas = vec![50_000.0f64, 100_000.0];
        let c = build_vertex_weights(
            &[VertexConstraintKind::Population, VertexConstraintKind::Area],
            &pops,
            &areas,
        );
        assert_eq!(c.ncon(), 2);
        let out = c.interleaved(2);
        // [pop_v0=100, area_v0=5ha, pop_v1=200, area_v1=10ha]
        assert_eq!(out, vec![100, 5, 200, 10]);
    }
}
