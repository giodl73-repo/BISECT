// Stub — full implementation lands in Task 3 (Core traits + api.rs)

/// Parameters forwarded to the METIS partitioner.
#[derive(Debug, Clone)]
pub struct MetisParams {
    pub k: u32,
}

/// Entry-point for graph partitioning.
pub struct Partitioner;
