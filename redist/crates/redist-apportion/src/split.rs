//! `Partitioner` trait and `MetisPartitioner` implementation.
//!
//! A `Partitioner` takes a `SubGraph` region and a target part count `k`,
//! and returns a partition assignment `part[local_vertex] ∈ [0, k)`.
//!
//! Implementors can use any algorithm: METIS k-way, spectral, random, etc.
//! The default `MetisPartitioner` uses `redist-metis` (pure Rust) as its
//! primary partitioner. When the `shadow-metis` feature is enabled (the
//! default during the validation phase), C METIS also runs in parallel and
//! the two cut sizes are compared; a warning is logged when the Rust cut
//! exceeds the C cut by more than 20%.
//!
//! Note: `redist-cli` has an unrelated `SplitStrategy` *enum* that controls
//! METIS mode selection for a single bisection call. This trait is different:
//! it is the polymorphic interface that `PfrCompositor` uses to compose a
//! full multi-level partition.

use std::collections::HashMap;
use thiserror::Error;
use crate::graph::SubGraph;

// Alias to avoid name collision with the local `Partitioner` trait below.
use redist_metis::api::{
    MetisPartitioner as RustMetisPartitioner,
    MetisParams,
    Partitioner as RustPartitioner,
};
use redist_metis::graph::CsrGraph;

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

/// A pluggable region-splitting algorithm.
///
/// Each call receives a `SubGraph` (the current region) and the number of
/// parts `k` to split it into. The seed, if any, is for METIS's internal RNG.
///
/// Implementors must:
/// - Return exactly `k` distinct part IDs in `[0, k)`.
/// - Respect population balance (each part's vertex-weight sum ≈ equal).
/// - Produce contiguous parts when possible.
/// Pluggable k-way region partitioning algorithm for `PfrCompositor`.
///
/// Named `Partitioner` to avoid confusion with the `SplitStrategy` *enum* in
/// `redist-cli::runner`, which is the CLI-level discriminator for which
/// high-level algorithm to run. This trait is the inner split primitive that
/// `PfrCompositor` calls at each level of the factorization tree.
pub trait Partitioner: Send + Sync {
    /// Split `region` into `k` parts with **equal** population targets.
    fn split(
        &self,
        region: &SubGraph,
        k: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError>;

    /// Split `region` into `target_fracs.len()` parts with **specified**
    /// population fractions (must sum to ~1.0). Used for binary fallback on
    /// prime k, where the two halves hold unequal district counts (e.g. 8/17
    /// and 9/17 for k=17).
    ///
    /// Default: delegates to `split` with equal weights (ignores fractions).
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

/// Compute the edge cut of a partition on a `CsrGraph`.
/// Each crossing edge (u,v) with `assignment[u] != assignment[v]` is counted
/// once (the raw loop counts both directions so we divide by 2).
#[cfg_attr(not(feature = "shadow-metis"), allow(dead_code))]
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

/// Default `Partitioner`: pure-Rust METIS (`redist-metis`) with population
/// vertex weights and TIGER edge weights.
///
/// When the `shadow-metis` feature is enabled, C METIS runs concurrently as a
/// quality oracle. A warning is emitted to `stderr` when the Rust edge cut
/// exceeds the C edge cut by more than 20%.
pub struct MetisPartitioner {
    /// Population balance tolerance (0.005 = 0.5%).
    pub balance_tolerance: f64,
    /// Number of METIS refinement iterations (default: 10).
    pub niter: i32,
}

impl Default for MetisPartitioner {
    fn default() -> Self {
        Self { balance_tolerance: 0.005, niter: 10 }
    }
}

impl MetisPartitioner {
    /// UFactor: METIS encodes balance as ufactor where
    /// allowed imbalance = 1 + ufactor/1000.
    /// So balance_tolerance=0.005 → ufactor=5.
    fn ufactor(&self) -> u32 {
        ((self.balance_tolerance * 1000.0).round() as u32).clamp(1, 1000)
    }

    /// Run C METIS on `region` with equal target weights.
    /// Only compiled when `shadow-metis` feature is active.
    fn split_c_metis(
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
                .map_err(|e| SplitError::Metis(format!("C bisection: {e}")))?;
        } else {
            graph.part_kway(&mut part)
                .map_err(|e| SplitError::Metis(format!("C kway k={k}: {e}")))?;
        }

        Ok(part.iter().map(|&p| p as u32).collect())
    }

    /// Run C METIS on `region` with asymmetric target fractions.
    /// Only compiled when `shadow-metis` feature is active.
    fn split_c_metis_weighted(
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
            .map_err(|e| SplitError::Metis(format!("C weighted kway k={k}: {e}")))?;

        Ok(part.iter().map(|&p| p as u32).collect())
    }
}

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

        // Trivial case: only one part requested.
        if k == 1 {
            return Ok(vec![0u32; n]);
        }

        // Guard: redist-metis rejects k > n (C METIS silently produced trivial
        // results in this case). When k >= n, assign each vertex its own part.
        if k as usize >= n {
            return Ok((0..n as u32).collect());
        }

        // Build CsrGraph for the Rust METIS engine.
        let g = CsrGraph::from(region);

        // Primary: C METIS FFI (battle-tested, handles all graph sizes and k values).
        // The pure Rust METIS (redist-metis) may stall on planar census tract graphs
        // at certain k values due to coarsening limits. C METIS is used as primary.
        let c_assignment = self.split_c_metis(region, k, seed)?;

        // Optional shadow: compare Rust METIS result and warn on quality regression.
        #[cfg(feature = "shadow-metis")]
        {
            let rust_params = MetisParams {
                ufactor:    self.ufactor(),
                niter:      self.niter as u32,
                seed,
                coarsen_to: 20,
                tpwgts:     None,
                ..MetisParams::default()
            };
            if let Ok(rust_part) = RustMetisPartitioner::with_params(rust_params, k).split(&g, k, seed) {
                let c_cut    = compute_cut(&g, &c_assignment);
                let rust_cut = compute_cut(&g, &rust_part.assignment);
                if rust_cut > 0 && c_cut > rust_cut * 12 / 10 {
                    eprintln!(
                        "[shadow-metis] k={k}: C cut {c_cut} > Rust cut {rust_cut} ({:.0}% over)",
                        (c_cut as f64 / rust_cut as f64 - 1.0) * 100.0
                    );
                }
            }
        }

        Ok(c_assignment)
    }

    /// Override: passes actual target fractions for accurate population balance
    /// on asymmetric binary splits (prime-k fallback).
    ///
    /// `redist-metis` v1 delegates to equal-weight split; the shadow check
    /// still runs C METIS with the supplied fractions for a fair comparison.
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

        // Build CsrGraph for the Rust METIS engine.
        let g = CsrGraph::from(region);

        // Convert f32 fracs to proportional u32 weights (thousandths).
        let fracs_u32: Vec<u32> = target_fracs
            .iter()
            .map(|&f| (f * 1000.0).round() as u32)
            .collect();

        let rust_params = MetisParams {
            ufactor:    self.ufactor(),
            niter:      self.niter as u32,
            seed,
            coarsen_to: 20,
            tpwgts:     None,
            ..MetisParams::default()
        };
        // Primary: C METIS FFI for weighted splits (population-balanced binary splits).
        let c_assignment = self.split_c_metis_weighted(region, target_fracs, seed)?;

        // Optional shadow: compare Rust METIS result.
        #[cfg(feature = "shadow-metis")]
        {
            if let Ok(rust_part) = RustMetisPartitioner::with_params(rust_params, k)
                .split_weighted(&g, &fracs_u32, seed)
            {
                let c_cut    = compute_cut(&g, &c_assignment);
                let rust_cut = compute_cut(&g, &rust_part.assignment);
                if rust_cut > 0 && c_cut > rust_cut * 12 / 10 {
                    eprintln!(
                        "[shadow-metis] weighted k={k}: C cut {c_cut} > Rust cut {rust_cut} ({:.0}% over)",
                        (c_cut as f64 / rust_cut as f64 - 1.0) * 100.0
                    );
                }
            }
        }

        Ok(c_assignment)
    }
}

/// Convenience: build a SubGraph from raw arrays and split it in one call.
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
