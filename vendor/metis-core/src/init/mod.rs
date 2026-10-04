use crate::error::PartitionError;
use crate::graph::{CsrGraph, Partition};

pub trait InitialPartitioner: Send + Sync {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Result<Partition, PartitionError>;
    fn partition_with_targets(&self, g: &CsrGraph, k: u32, seed: u64, targets: &[f32]) -> Result<Partition, PartitionError> {
        let mut partition=self.partition(g,k,seed)?;
        partition.tpwgts=Some(targets.to_vec());
        Ok(partition)
    }
}

pub mod grow;
pub mod multiconstraint;
pub mod random;
