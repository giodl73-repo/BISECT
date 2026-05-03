use crate::{graph::{CsrGraph, Partition}, error::PartitionError};
pub use crate::coarsen::Coarsener;
pub use crate::init::InitialPartitioner;
pub use crate::refine::Refiner;

pub trait Partitioner: Send + Sync {
    fn split(&self, g: &CsrGraph, k: u32, seed: Option<u64>)
        -> Result<Partition, PartitionError>;
    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>;
}

#[derive(Debug, Clone)]
pub struct MetisParams {
    pub ufactor:    u32,
    pub niter:      u32,
    pub seed:       Option<u64>,
    pub coarsen_to: u32,
}

impl Default for MetisParams {
    fn default() -> Self {
        Self { ufactor: 5, niter: 10, seed: None, coarsen_to: 20 }
    }
}

pub struct RustMetisPartitioner<C, I, R> {
    pub coarsener: C,
    pub init:      I,
    pub refiner:   R,
    pub params:    MetisParams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CsrGraph, Partition, CoarseMap};
    use crate::coarsen::Coarsener;
    use crate::init::InitialPartitioner;
    use crate::refine::Refiner;

    struct AlwaysTrivial;
    impl Coarsener for AlwaysTrivial {
        fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) {
            let cmap = (0..g.n() as u32).collect();
            (g.clone(), CoarseMap { cmap })
        }
        fn should_stop(&self, _: &CsrGraph) -> bool { true }
    }

    struct AllZeroPartitioner;
    impl InitialPartitioner for AllZeroPartitioner {
        fn partition(&self, g: &CsrGraph, _k: u32, _seed: u64) -> Partition {
            Partition { assignment: vec![0; g.n()], k: 1 }
        }
    }

    struct IdentityRefiner;
    impl Refiner for IdentityRefiner {
        fn refine(&self, _g: &CsrGraph, p: Partition) -> Partition { p }
    }

    #[test]
    fn mock_traits_compile() {
        let _c: &dyn Coarsener = &AlwaysTrivial;
        let _i: &dyn InitialPartitioner = &AllZeroPartitioner;
        let _r: &dyn Refiner = &IdentityRefiner;
    }

    #[test]
    fn metis_params_default() {
        let p = MetisParams::default();
        assert_eq!(p.ufactor, 5);
        assert_eq!(p.niter, 10);
        assert_eq!(p.seed, None);
        assert_eq!(p.coarsen_to, 20);
    }
}
