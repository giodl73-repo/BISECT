//! District proposal for SMC: spanning tree growth + balanced cut selection.
//! Per spec §2.2. Implemented in task #108.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProposeError {
    #[error("no valid balanced cut found in spanning tree (stage {stage}, particle {particle_idx})")]
    NoValidCut { stage: usize, particle_idx: usize },
    #[error("unassigned subgraph is empty at stage {stage}")]
    EmptySubgraph { stage: usize },
}

// Implementation in task #108.
