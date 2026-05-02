//! `Partitioner` trait and `MetisPartitioner` implementation.
//!
//! A `Partitioner` takes a `SubGraph` region and a target part count `k`,
//! and returns a partition assignment `part[local_vertex] ∈ [0, k)`.
//!
//! Implementors can use any algorithm: METIS k-way, spectral, random, etc.
//! The default `MetisPartitioner` uses METIS with population balance and
//! TIGER edge weights.
//!
//! Note: `redist-cli` has an unrelated `SplitStrategy` *enum* that controls
//! METIS mode selection for a single bisection call. This trait is different:
//! it is the polymorphic interface that `PfrCompositor` uses to compose a
//! full multi-level partition.

use std::collections::HashMap;
use thiserror::Error;
use crate::graph::SubGraph;

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

/// Default `Partitioner`: METIS k-way with population vertex weights and
/// TIGER edge weights. For `k = 2` uses `part_recursive` (faster); `k > 2`
/// uses `part_kway`.
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

        // UFactor: METIS encodes balance as ufactor where
        // allowed imbalance = 1 + ufactor/1000.
        // So balance_tolerance=0.005 → ufactor=5.
        let uf_int = ((self.balance_tolerance * 1000.0).round() as i32).clamp(1, 1000);

        // Equal target weights across all k parts.
        let tpwgts: Vec<f32> = vec![1.0_f32 / k as f32; k as usize];

        let mut part = vec![0i32; n];

        let graph = metis::Graph::new(1, k as i32, &region.xadj, &region.adjncy)
            .map_err(|e| SplitError::Metis(e.to_string()))?
            .set_vwgt(&region.vwgt)
            .set_tpwgts(&tpwgts);

        let graph = if let Some(ref ew) = region.adjwgt {
            graph.set_adjwgt(ew)
        } else {
            graph
        };

        let graph = graph
            .set_option(metis::option::UFactor(uf_int))
            .set_option(metis::option::NIter(self.niter))
            .set_option(metis::option::Contig(true))
            .set_option(metis::option::MinConn(true));

        let graph = if let Some(s) = seed {
            graph.set_option(metis::option::Seed(((s & 0x7FFF_FFFF) as i32).max(1)))
        } else {
            graph
        };

        if k == 2 {
            graph.part_recursive(&mut part)
                .map_err(|e| SplitError::Metis(format!("bisection: {e}")))?;
        } else {
            graph.part_kway(&mut part)
                .map_err(|e| SplitError::Metis(format!("kway k={k}: {e}")))?;
        }

        Ok(part.iter().map(|&p| p as u32).collect())
    }

    /// Override: passes actual target fractions to METIS for accurate
    /// population balance on asymmetric binary splits (prime-k fallback).
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

        // Asymmetric splits always use part_kway (part_recursive requires equal halves).
        graph.part_kway(&mut part)
            .map_err(|e| SplitError::Metis(format!("weighted kway k={k}: {e}")))?;

        Ok(part.iter().map(|&p| p as u32).collect())
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
