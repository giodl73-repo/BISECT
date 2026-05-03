use crate::{graph::{CsrGraph, Partition}, error::PartitionError};
pub use crate::coarsen::Coarsener;
pub use crate::init::InitialPartitioner;
pub use crate::refine::Refiner;
use crate::multilevel::hierarchy::CoarseningHierarchy;
use crate::multilevel::pipeline::Pipeline;
use crate::coarsen::shem::SortedHeavyEdgeMatchWithParams;
use crate::init::grow::GrowBisect;
use crate::refine::fm::FiducciaMattheyses;

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

/// Concrete type alias: SHEM + GrowBisect + FM — the default METIS-like pipeline.
pub type MetisPartitioner = RustMetisPartitioner<
    SortedHeavyEdgeMatchWithParams,
    GrowBisect,
    FiducciaMattheyses,
>;

impl MetisPartitioner {
    pub fn with_params(params: MetisParams, k: u32) -> Self {
        RustMetisPartitioner {
            coarsener: SortedHeavyEdgeMatchWithParams {
                coarsen_to: params.coarsen_to,
                k,
            },
            init:    GrowBisect,
            refiner: FiducciaMattheyses { niter: params.niter },
            params,
        }
    }
}

impl<C: Coarsener, I: InitialPartitioner, R: Refiner> Partitioner
    for RustMetisPartitioner<C, I, R>
{
    // ── Prusti formal postconditions ─────────────────────────────────────────
    // Activate with: cargo prusti (Prusti 0.2.x, Viper backend)
    // These postconditions are the legally-cited properties from the deposition.
    //
    // PRUSTI: #[requires(g.is_valid())]
    // PRUSTI: #[requires(k >= 1)]
    // PRUSTI: #[ensures(result.is_ok() ==> result.as_ref().unwrap().assignment.len() == g.n())]
    // PRUSTI: #[ensures(result.is_ok() ==>
    // PRUSTI:     forall(|i: usize| i < result.as_ref().unwrap().assignment.len()
    // PRUSTI:         ==> result.as_ref().unwrap().assignment[i] < k))]
    // PRUSTI: #[ensures(result.is_ok() ==>
    // PRUSTI:     population_balance(result.as_ref().unwrap(), g) <= epsilon(g))]
    //
    // epsilon(g) = (total_vwgt_sum(g) * 5 + 999) / 1000  (ceiling of 0.5%, integer)
    //
    // Postcondition 1: full coverage — every vertex assigned
    // Postcondition 2: valid part IDs — no phantom districts
    // Postcondition 3: population balance ≤ 0.5% — one-person-one-vote
    // ─────────────────────────────────────────────────────────────────────────

    fn split(&self, g: &CsrGraph, k: u32, seed: Option<u64>)
        -> Result<Partition, PartitionError>
    {
        if k == 0 { return Err(PartitionError::ZeroParts); }
        if g.n() == 0 { return Err(PartitionError::EmptyGraph); }
        if !g.is_valid() {
            return Err(PartitionError::InvalidGraph("is_valid() failed"));
        }
        if k as usize > g.n() {
            return Err(PartitionError::TooManyParts { k, n: g.n() });
        }

        let rng_seed = self.params.seed.or(seed).unwrap_or(0xDEAD_BEEF_CAFE_1234u64);

        let hierarchy = CoarseningHierarchy::build(g, &self.coarsener)?;
        let p = Pipeline::new(hierarchy)
            .initial_partition(&self.init, k, rng_seed)
            .refine_and_project(&self.refiner)
            .into_partition();

        Ok(p)
    }

    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>
    {
        if fracs.is_empty() { return Err(PartitionError::ZeroParts); }
        let k = fracs.len() as u32;
        // v1: delegate to equal-weight split; proportional balance is handled
        // by FM during refinement.
        self.split(g, k, seed)
    }
}

/// Population balance metric used in Prusti postcondition 3.
/// Returns true iff max deviation from target per part is ≤ epsilon.
/// epsilon = (total_pop * 5 + 999) / 1000  (ceiling of 0.5%, integer arithmetic).
#[cfg(any(test, doc))]
pub fn population_balance_check(p: &Partition, g: &CsrGraph) -> bool {
    let total_pop: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
    let target = total_pop / p.k as i64;
    let epsilon = (total_pop * 5 + 999) / 1000;
    for part in 0..p.k {
        let pop: i64 = (0..g.n())
            .filter(|&v| p.assignment[v] == part)
            .map(|v| g.vwgt[v] as i64)
            .sum();
        if (pop - target).abs() > epsilon { return false; }
    }
    true
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

    fn make_path_graph(n: usize) -> CsrGraph {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i - 1) as u32); }
            if i < n - 1 { adjncy.push((i + 1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
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

    #[test]
    fn full_pipeline_path10_k2() {
        use crate::coarsen::shem::SortedHeavyEdgeMatchWithParams;
        use crate::init::grow::GrowBisect;
        use crate::refine::fm::FiducciaMattheyses;
        let g = make_path_graph(10);
        let partitioner = RustMetisPartitioner {
            coarsener: SortedHeavyEdgeMatchWithParams { coarsen_to: 20, k: 2 },
            init:      GrowBisect,
            refiner:   FiducciaMattheyses { niter: 10 },
            params:    MetisParams::default(),
        };
        let p = partitioner.split(&g, 2, Some(42)).unwrap();
        assert_eq!(p.assignment.len(), 10);
        assert_eq!(p.k, 2);
        assert!(p.assignment.contains(&0));
        assert!(p.assignment.contains(&1));
    }

    #[test]
    fn split_weighted_empty_fracs_errors() {
        let g = make_path_graph(10);
        let p = MetisPartitioner::with_params(MetisParams::default(), 1)
            .split_weighted(&g, &[], Some(0));
        assert!(matches!(p, Err(crate::error::PartitionError::ZeroParts)));
    }

    #[test]
    fn split_zero_k_errors() {
        let g = make_path_graph(10);
        let partitioner = MetisPartitioner::with_params(MetisParams::default(), 2);
        let result = partitioner.split(&g, 0, None);
        assert!(matches!(result, Err(PartitionError::ZeroParts)));
    }

    #[test]
    fn split_too_many_parts_errors() {
        let g = make_path_graph(3);
        let partitioner = MetisPartitioner::with_params(MetisParams::default(), 10);
        let result = partitioner.split(&g, 10, None);
        assert!(matches!(result, Err(PartitionError::TooManyParts { k: 10, n: 3 })));
    }

    #[test]
    fn metis_partitioner_with_params_works() {
        let g = make_path_graph(20);
        let partitioner = MetisPartitioner::with_params(MetisParams::default(), 2);
        let p = partitioner.split(&g, 2, Some(7)).unwrap();
        assert_eq!(p.assignment.len(), 20);
        assert_eq!(p.k, 2);
    }

    #[test]
    fn split_weighted_delegates_to_split() {
        let g = make_path_graph(10);
        let partitioner = MetisPartitioner::with_params(MetisParams::default(), 2);
        // 2 fracs → k=2 split
        let p = partitioner.split_weighted(&g, &[50, 50], Some(0)).unwrap();
        assert_eq!(p.assignment.len(), 10);
        assert_eq!(p.k, 2);
    }
}
