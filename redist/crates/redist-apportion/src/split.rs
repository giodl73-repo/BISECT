//! `Partitioner` trait and `MetisPartitioner` implementation.
//!
//! Three METIS backends are supported, selectable via [`MetisEngine`]:
//!
//! | Engine | Rust name | Dependency | Notes |
//! |--------|-----------|------------|-------|
//! | C METIS FFI | `MetisEngine::CFfi` | `libmetis.so` / `.dll` / `.dylib` | Default; battle-tested, handles all k |
//! | metis-rs (alias) | same as CFfi | same | The `metis` Rust crate IS the C FFI |
//! | metis-core | `MetisEngine::RedistMetis` | none | Pure Rust; portable standalone binary; may stall on prime k for large graphs |
//! | gpmetis subprocess | `MetisEngine::Gpmetis` | `gpmetis` on PATH | Reserved; not yet implemented |
//!
//! When the `shadow-metis` Cargo feature is enabled with engine `CFfi`, the
//! `metis-core` Rust implementation also runs as a quality oracle: a warning
//! is emitted when the Rust edge-cut exceeds the C edge-cut by more than 20%.
//!
//! Note: `redist-cli` has an unrelated `SplitStrategy` *enum* that controls
//! the high-level bisection mode. This module's `Partitioner` trait is the
//! lower-level primitive that `PfrCompositor` calls at each tree level.

use std::collections::HashMap;
use thiserror::Error;
use crate::graph::SubGraph;

use metis_core::api::{
    MetisPartitioner as RustMetisPartitioner,
    MetisParams,
    Partitioner as RustPartitioner,
};
use metis_core::graph::CsrGraph;

// ── MetisEngine ───────────────────────────────────────────────────────────────

/// Which METIS backend `MetisPartitioner` uses for each split call.
///
/// Select via `--metis-engine` CLI flag or `engine:` in the YAML config.
///
/// The default depends on which Cargo features are enabled:
/// - With `c-ffi` feature (the crate default): `CFfi`
/// - Without `c-ffi` (pure-Rust build, `--no-default-features`): `RedistMetis`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetisEngine {
    /// **C METIS FFI** — the `metis` Rust crate with bundled C source (`vendored`).
    /// A C compiler is required at build time; the resulting binary is standalone
    /// (no system libmetis needed at runtime).  Handles all k values including
    /// prime k without coarsening stall.
    /// Only available when the `c-ffi` Cargo feature is enabled (the default).
    CFfi,

    /// **`metis-core`** — pure-Rust multilevel graph partitioning.
    /// No C dependency at all; no C compiler required.  Fully portable.
    /// May stall on planar census-tract graphs at certain prime k values
    /// due to coarsening depth limits in the Rust implementation.
    RedistMetis,

    /// **gpmetis subprocess** — shells out to the external `gpmetis` binary.
    /// Requires `gpmetis` on `PATH`.
    /// ⚠ Not yet implemented — returns `SplitError::Metis` if selected.
    Gpmetis,
}

impl Default for MetisEngine {
    fn default() -> Self {
        // When the c-ffi feature is enabled, default to the C engine (battle-tested).
        // When building without c-ffi (pure-Rust / portable), fall back to RedistMetis
        // so that MetisPartitioner::default() works without a C toolchain.
        #[cfg(feature = "c-ffi")]
        { MetisEngine::CFfi }
        #[cfg(not(feature = "c-ffi"))]
        { MetisEngine::RedistMetis }
    }
}

impl MetisEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            MetisEngine::CFfi        => "c-ffi",
            MetisEngine::RedistMetis => "metis-core",
            MetisEngine::Gpmetis     => "gpmetis",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "c-ffi" | "c" | "metis-rs" => Some(MetisEngine::CFfi),
            "redist-metis" | "rust"    => Some(MetisEngine::RedistMetis),
            "gpmetis" | "subprocess"   => Some(MetisEngine::Gpmetis),
            _                          => None,
        }
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SplitError {
    #[error("empty region passed to split strategy")]
    EmptyRegion,
    #[error("k=0 requested; minimum is 1")]
    ZeroParts,
    #[error("METIS error: {0}")]
    Metis(String),
    #[error("graph build error: {0}")]
    Graph(#[from] crate::graph::GraphError),
    #[error("split returned {returned} parts but {k} were requested")]
    WrongPartCount { returned: usize, k: u32 },
}

// ── Partitioner trait ─────────────────────────────────────────────────────────

/// Pluggable k-way region partitioning algorithm for `PfrCompositor`.
///
/// Each call receives a `SubGraph` (the current region) and the number of
/// parts `k`. The seed, if any, is forwarded to the backend's internal RNG.
pub trait Partitioner: Send + Sync {
    /// Split `region` into `k` parts with **equal** population targets.
    fn split(
        &self,
        region: &SubGraph,
        k: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError>;

    /// Split `region` into `target_fracs.len()` parts with **specified**
    /// population fractions (must sum to ~1.0).  Used for the binary
    /// fallback on prime k (e.g. 8/17 and 9/17 for k=17).
    ///
    /// Default: delegates to `split` with equal weights.
    /// Override for accurate population balance on asymmetric splits.
    fn split_weighted(
        &self,
        region: &SubGraph,
        target_fracs: &[f32],
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        self.split(region, target_fracs.len() as u32, seed)
    }
}

// ── Edge-cut helper (used by shadow comparison) ───────────────────────────────

#[allow(dead_code)]
fn compute_cut(g: &CsrGraph, assignment: &[u32]) -> usize {
    let mut cut = 0usize;
    for v in 0..g.n() {
        for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
            let u = g.adjncy[j] as usize;
            if assignment[v] != assignment[u] {
                cut += 1;
            }
        }
    }
    cut / 2
}

// ── MetisPartitioner ──────────────────────────────────────────────────────────

/// Concrete `Partitioner` that dispatches to one of the three METIS backends.
pub struct MetisPartitioner {
    /// Population balance tolerance (0.005 = 0.5%).
    pub balance_tolerance: f64,
    /// Number of METIS refinement iterations (default: 10).
    pub niter: i32,
    /// Which METIS backend to use (default: `CFfi`).
    pub engine: MetisEngine,
}

impl Default for MetisPartitioner {
    fn default() -> Self {
        Self { balance_tolerance: 0.005, niter: 10, engine: MetisEngine::default() }
    }
}

impl MetisPartitioner {
    pub fn with_engine(engine: MetisEngine) -> Self {
        Self { engine, ..Self::default() }
    }

    /// UFactor encoding: ufactor = balance_tolerance * 1000 (clamped to [1, 1000]).
    fn ufactor(&self) -> u32 {
        ((self.balance_tolerance * 1000.0).round() as u32).clamp(1, 1000)
    }

    // ── C METIS FFI path ─────────────────────────────────────────────────────
    // Only compiled when the "c-ffi" Cargo feature is enabled.

    /// Equal-weight k-way split via the `metis` Rust FFI crate → bundled C METIS.
    #[cfg(feature = "c-ffi")]
    fn split_c_ffi(
        &self,
        region: &SubGraph,
        k: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        let uf_int = ((self.balance_tolerance * 1000.0).round() as i32).clamp(1, 1000);
        let tpwgts: Vec<f32> = vec![1.0_f32 / k as f32; k as usize];
        let mut part = vec![0i32; region.n_vertices()];

        let graph = metis::Graph::new(1, k as i32, &region.xadj, &region.adjncy)
            .map_err(|e| SplitError::Metis(e.to_string()))?
            .set_vwgt(&region.vwgt)
            .set_tpwgts(&tpwgts);
        let graph = if let Some(ref ew) = region.adjwgt { graph.set_adjwgt(ew) } else { graph };
        let graph = graph
            .set_option(metis::option::UFactor(uf_int))
            .set_option(metis::option::NIter(self.niter))
            .set_option(metis::option::Contig(true))
            .set_option(metis::option::MinConn(true));
        let graph = if let Some(s) = seed {
            graph.set_option(metis::option::Seed(((s & 0x7FFF_FFFF) as i32).max(1)))
        } else { graph };

        if k == 2 {
            graph.part_recursive(&mut part)
                .map_err(|e| SplitError::Metis(format!("c-ffi bisection: {e}")))?;
        } else {
            graph.part_kway(&mut part)
                .map_err(|e| SplitError::Metis(format!("c-ffi kway k={k}: {e}")))?;
        }

        Ok(part.iter().map(|&p| p as u32).collect())
    }

    /// Asymmetric-weight k-way split via the `metis` Rust FFI crate → bundled C METIS.
    #[cfg(feature = "c-ffi")]
    fn split_c_ffi_weighted(
        &self,
        region: &SubGraph,
        target_fracs: &[f32],
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        let k = target_fracs.len() as u32;
        let uf_int = ((self.balance_tolerance * 1000.0).round() as i32).clamp(1, 1000);
        let mut part = vec![0i32; region.n_vertices()];

        let graph = metis::Graph::new(1, k as i32, &region.xadj, &region.adjncy)
            .map_err(|e| SplitError::Metis(e.to_string()))?
            .set_vwgt(&region.vwgt)
            .set_tpwgts(target_fracs);
        let graph = if let Some(ref ew) = region.adjwgt { graph.set_adjwgt(ew) } else { graph };
        let graph = graph
            .set_option(metis::option::UFactor(uf_int))
            .set_option(metis::option::NIter(self.niter))
            .set_option(metis::option::Contig(true))
            .set_option(metis::option::MinConn(true));
        let graph = if let Some(s) = seed {
            graph.set_option(metis::option::Seed(((s & 0x7FFF_FFFF) as i32).max(1)))
        } else { graph };

        graph.part_kway(&mut part)
            .map_err(|e| SplitError::Metis(format!("c-ffi weighted kway k={k}: {e}")))?;

        Ok(part.iter().map(|&p| p as u32).collect())
    }

    // ── redist-metis pure-Rust path ───────────────────────────────────────────

    /// Equal-weight k-way split via the `redist-metis` pure-Rust engine.
    fn split_metis_core(
        &self,
        g: &CsrGraph,
        k: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        let params = MetisParams {
            ufactor:    self.ufactor(),
            niter:      self.niter as u32,
            seed,
            coarsen_to: 20,
            tpwgts:     None,
            ..MetisParams::default()
        };
        RustMetisPartitioner::with_params(params, k)
            .split(g, k, seed)
            .map(|r| r.assignment)
            .map_err(|e| SplitError::Metis(format!("redist-metis k={k}: {e}")))
    }

    /// Asymmetric-weight k-way split via `redist-metis`.
    /// Falls back to equal-weight split (v1 does not support tpwgts).
    fn split_metis_core_weighted(
        &self,
        g: &CsrGraph,
        k: u32,
        fracs_u32: &[u32],
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        let params = MetisParams {
            ufactor:    self.ufactor(),
            niter:      self.niter as u32,
            seed,
            coarsen_to: 20,
            tpwgts:     None,
            ..MetisParams::default()
        };
        RustMetisPartitioner::with_params(params, k)
            .split_weighted(g, fracs_u32, seed)
            .map(|r| r.assignment)
            .map_err(|e| SplitError::Metis(format!("redist-metis weighted k={k}: {e}")))
    }
}

// ── Partitioner impl ──────────────────────────────────────────────────────────

impl Partitioner for MetisPartitioner {
    fn split(
        &self,
        region: &SubGraph,
        k: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        if k == 0 { return Err(SplitError::ZeroParts); }
        let n = region.n_vertices();
        if n == 0 { return Err(SplitError::EmptyRegion); }
        if k == 1 { return Ok(vec![0u32; n]); }
        if k as usize >= n { return Ok((0..n as u32).collect()); }

        match self.engine {
            MetisEngine::CFfi => {
                #[cfg(not(feature = "c-ffi"))]
                return Err(SplitError::Metis(
                    "redist-apportion was compiled without the c-ffi feature; \
                     rebuild with --features c-ffi or use --metis-engine redist-metis".into()
                ));
                #[cfg(feature = "c-ffi")]
                {
                    let assignment = self.split_c_ffi(region, k, seed)?;

                    #[cfg(feature = "shadow-metis")]
                    {
                        let g = CsrGraph::from(region);
                        if let Ok(rust_assignment) = self.split_metis_core(&g, k, seed) {
                            let c_cut    = compute_cut(&g, &assignment);
                            let rust_cut = compute_cut(&g, &rust_assignment);
                            if rust_cut > 0 && c_cut > rust_cut * 12 / 10 {
                                eprintln!(
                                    "[shadow-metis] k={k}: c-ffi cut {c_cut} > redist-metis \
                                     cut {rust_cut} ({:.0}% over)",
                                    (c_cut as f64 / rust_cut as f64 - 1.0) * 100.0
                                );
                            }
                        }
                    }

                    Ok(assignment)
                }
            }

            MetisEngine::RedistMetis => {
                let g = CsrGraph::from(region);
                self.split_metis_core(&g, k, seed)
            }

            MetisEngine::Gpmetis => Err(SplitError::Metis(
                "gpmetis subprocess engine is not yet implemented; \
                 use c-ffi (default) or redist-metis".into()
            )),
        }
    }

    fn split_weighted(
        &self,
        region: &SubGraph,
        target_fracs: &[f32],
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError> {
        let k = target_fracs.len() as u32;
        if k == 0 { return Err(SplitError::ZeroParts); }
        if region.n_vertices() == 0 { return Err(SplitError::EmptyRegion); }
        if k == 1 { return Ok(vec![0u32; region.n_vertices()]); }

        match self.engine {
            MetisEngine::CFfi => {
                #[cfg(not(feature = "c-ffi"))]
                return Err(SplitError::Metis(
                    "redist-apportion was compiled without the c-ffi feature; \
                     rebuild with --features c-ffi or use --metis-engine redist-metis".into()
                ));
                #[cfg(feature = "c-ffi")]
                {
                    let assignment = self.split_c_ffi_weighted(region, target_fracs, seed)?;

                    #[cfg(feature = "shadow-metis")]
                    {
                        let g = CsrGraph::from(region);
                        let fracs_u32: Vec<u32> = target_fracs
                            .iter()
                            .map(|&f| (f * 1000.0).round() as u32)
                            .collect();
                        if let Ok(rust_assignment) =
                            self.split_metis_core_weighted(&g, k, &fracs_u32, seed)
                        {
                            let c_cut    = compute_cut(&g, &assignment);
                            let rust_cut = compute_cut(&g, &rust_assignment);
                            if rust_cut > 0 && c_cut > rust_cut * 12 / 10 {
                                eprintln!(
                                    "[shadow-metis] weighted k={k}: c-ffi cut {c_cut} > \
                                     redist-metis cut {rust_cut} ({:.0}% over)",
                                    (c_cut as f64 / rust_cut as f64 - 1.0) * 100.0
                                );
                            }
                        }
                    }

                    Ok(assignment)
                }
            }

            MetisEngine::RedistMetis => {
                let g = CsrGraph::from(region);
                let fracs_u32: Vec<u32> = target_fracs
                    .iter()
                    .map(|&f| (f * 1000.0).round() as u32)
                    .collect();
                self.split_metis_core_weighted(&g, k, &fracs_u32, seed)
            }

            MetisEngine::Gpmetis => Err(SplitError::Metis(
                "gpmetis subprocess engine is not yet implemented; \
                 use c-ffi (default) or redist-metis".into()
            )),
        }
    }
}

// ── Convenience function ──────────────────────────────────────────────────────

/// Build a SubGraph from raw arrays and split it in one call.
pub fn split_region(
    strategy: &dyn Partitioner,
    vertices: &[usize],
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    k: u32,
    seed: Option<u64>,
) -> Result<(SubGraph, Vec<u32>), SplitError> {
    let region = SubGraph::build(vertices, adjacency, vertex_weights, edge_weights)?;
    let assignment = strategy.split(&region, k, seed)?;
    Ok((region, assignment))
}
