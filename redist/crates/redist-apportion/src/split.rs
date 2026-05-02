//! `SplitStrategy` trait and `MetisKwaySplit` implementation.
//!
//! A `SplitStrategy` takes a `SubGraph` region and a target part count `k`,
//! and returns a partition assignment `part[local_vertex] ∈ [0, k)`.
//!
//! Implementors can use any algorithm: METIS k-way, spectral, random, etc.
//! The default `MetisKwaySplit` uses METIS with population balance and
//! TIGER edge weights.

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
pub trait SplitStrategy: Send + Sync {
    fn split(
        &self,
        region: &SubGraph,
        k: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u32>, SplitError>;
}

/// Default strategy: METIS k-way graph partitioning with population
/// vertex weights and TIGER edge weights.
///
/// For `k = 2`, uses `part_recursive` (faster). For `k > 2`, uses `part_kway`.
pub struct MetisKwaySplit {
    /// Population balance tolerance (0.005 = 0.5%).
    pub balance_tolerance: f64,
    /// Number of METIS refinement iterations (default: 10).
    pub niter: i32,
}

impl Default for MetisKwaySplit {
    fn default() -> Self {
        Self { balance_tolerance: 0.005, niter: 10 }
    }
}

impl SplitStrategy for MetisKwaySplit {
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
}

/// Convenience: build a SubGraph from raw arrays and split it in one call.
pub fn split_region(
    strategy: &dyn SplitStrategy,
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
