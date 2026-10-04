use crate::error::PartitionError;
use crate::graph::{CoarseMap, CsrGraph};

pub trait Coarsener: Send + Sync {
    /// Collapse g by one level. Output graph has strictly fewer vertices.
    /// Requires: g.is_valid(), g.n() >= 2.
    fn coarsen(&self, g: &CsrGraph) -> Result<(CsrGraph, CoarseMap), PartitionError>;

    /// True when g is small enough to partition directly.
    /// Guaranteed to return true when g.n() <= max(coarsen_to * k, 40).
    fn should_stop(&self, g: &CsrGraph) -> bool;
}

pub mod hem;
pub mod mindegree;
pub mod shem;
pub mod twohop;

// Bound aggregate weights at every matching level. Without this, strong edges
// can collapse most population into one indivisible coarse vertex; refinement
// then starts far outside balance and may be unable to recover contiguously.
pub(crate) fn matching_weight_limits(g: &CsrGraph, threshold: u32) -> Vec<i64> {
    (0..g.ncon as usize).map(|c| {
        let total: i64 = g.vwgt.chunks(g.ncon as usize).map(|w|i64::from(w[c])).sum();
        ((total * 3 + i64::from(threshold) * 2 - 1) / (i64::from(threshold) * 2)).max(1)
    }).collect()
}

pub(crate) fn can_match(g: &CsrGraph, v: usize, u: usize, limits: Option<&[i64]>) -> bool {
    limits.is_none_or(|limits| (0..g.ncon as usize).all(|c|
        i64::from(g.vwgt[v*g.ncon as usize+c]) + i64::from(g.vwgt[u*g.ncon as usize+c]) <= limits[c]))
}
