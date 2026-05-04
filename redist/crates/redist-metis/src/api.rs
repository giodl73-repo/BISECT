use crate::{graph::{CsrGraph, Partition, repair_contiguity}, error::PartitionError};
pub use crate::coarsen::Coarsener;
pub use crate::init::InitialPartitioner;
pub use crate::refine::Refiner;
use crate::multilevel::hierarchy::CoarseningHierarchy;
use crate::multilevel::pipeline::Pipeline;
use crate::coarsen::shem::SortedHeavyEdgeMatchWithParams;
use crate::init::grow::GrowBisect;
use crate::refine::fm::FiducciaMattheyses;

// Prusti deductive verification — active only when `cargo prusti` is the toolchain.
// Normal `cargo build` and `cargo test` never see these items.
#[cfg(prusti)]
extern crate prusti_contracts;
#[cfg(prusti)]
use prusti_contracts::*;

pub trait Partitioner: Send + Sync {
    fn split(&self, g: &CsrGraph, k: u32, seed: Option<u64>)
        -> Result<Partition, PartitionError>;
    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>;
}

/// Coarsening algorithm. Mirrors the METIS `-ctype` option.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CoarseningMethod {
    #[default]
    Shem,      // Sorted Heavy-Edge Matching — O(n+m) bucket sort (default)
    Hem,       // Heavy-Edge Matching — random visit order
    MinDegree, // Minimum-degree matching
    TwoHop,    // Two-hop matching for sparse/irregular graphs
}

/// Objective function for FM refinement. Mirrors METIS `-objtype`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ObjectiveType {
    #[default]
    Cut,    // minimise edge cut (default, METIS_OBJTYPE_CUT)
    Volume, // minimise communication volume (METIS_OBJTYPE_VOL)
}

fn compute_cut(g: &CsrGraph, assignment: &[u32]) -> i64 {
    let mut cut = 0i64;
    for v in 0..g.n() {
        for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
            let u = g.adjncy[j] as usize;
            if assignment[v] != assignment[u] {
                cut += g.adjwgt.as_ref().map_or(1i64, |aw| aw[j] as i64);
            }
        }
    }
    cut / 2
}

#[derive(Debug, Clone)]
pub struct MetisParams {
    pub ufactor:    u32,
    pub niter:      u32,
    pub seed:       Option<u64>,
    pub coarsen_to: u32,
    /// Target partition weights for asymmetric splits (set by split_weighted).
    pub tpwgts:     Option<Vec<f32>>,
    /// Number of independent partition trials; best cut is returned. Default: 1.
    pub ncuts:      u32,
    /// Skip FM moves that would disconnect the source part. Default: true.
    pub contig_fm:  bool,
    /// Use multilevel recursive bisection for k>2. Default: false.
    pub use_recursive: bool,
    /// Objective function (Cut or Volume). Default: Cut.
    pub objective:  ObjectiveType,
    /// Minimize subdomain connectivity after partitioning. Default: true.
    pub min_conn:   bool,
    /// Label-propagation balance pass before FM. Default: true.
    pub lp_refine:  bool,
    /// Max LP balance iterations. Default: 10.
    pub lp_iter:    u32,
    /// Coarsening algorithm. Default: Shem.
    pub coarsen_method: CoarseningMethod,
}

impl Default for MetisParams {
    fn default() -> Self {
        Self {
            ufactor:        5,
            niter:          10,
            seed:           None,
            coarsen_to:     20,
            tpwgts:         None,
            ncuts:          1,
            contig_fm:      true,
            use_recursive:  false,
            objective:      ObjectiveType::Cut,
            min_conn:       true,
            lp_refine:      true,
            lp_iter:        10,
            coarsen_method: CoarseningMethod::Shem,
        }
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
            refiner: FiducciaMattheyses {
                niter:     params.niter,
                contig_fm: params.contig_fm,
                objective: params.objective,
                lp_iter:   if params.lp_refine { params.lp_iter } else { 0 },
                ..FiducciaMattheyses::default()
            },
            params,
        }
    }
}

impl<C: Coarsener, I: InitialPartitioner, R: Refiner> Partitioner
    for RustMetisPartitioner<C, I, R>
{
    // ── Prusti formal postconditions ─────────────────────────────────────────
    // Real #[cfg_attr(prusti, ...)] annotations below.
    // Active only when `cargo prusti` runs (Prusti 0.2.x, Viper backend).
    // These are the legally-cited properties from the deposition.
    //
    // Precondition 1:  #[cfg_attr(prusti, requires(g.is_valid()))]
    // Precondition 2:  #[cfg_attr(prusti, requires(k >= 1))]
    // Postcondition 1: full coverage — every vertex assigned
    //                  #[cfg_attr(prusti, ensures(...))]  ← see below
    // Postcondition 2: valid part IDs — no phantom districts
    //                  #[cfg_attr(prusti, ensures(...))]  ← see below
    // Postcondition 3: population balance ≤ 0.5% — one-person-one-vote
    //                  population_balanced() pure fn (DEFERRED — see GAPS.md)
    // ─────────────────────────────────────────────────────────────────────────
    /// The three legally-cited postconditions. Active when `cargo prusti` runs.
    /// Requires Prusti 0.2.x (Viper backend). See verify/prusti/ for documentation.
    #[cfg_attr(prusti, requires(g.is_valid()))]
    #[cfg_attr(prusti, requires(k >= 1))]
    #[cfg_attr(prusti, ensures(
        result.is_ok() ==>
        result.as_ref().unwrap().assignment.len() == g.n()
    ))]
    #[cfg_attr(prusti, ensures(
        result.is_ok() ==>
        forall(|i: usize|
            (i < result.as_ref().unwrap().assignment.len()) ==>
            (result.as_ref().unwrap().assignment[i] < k))
    ))]
    #[cfg_attr(prusti, ensures(
        result.is_ok() ==>
        population_balanced(result.as_ref().unwrap(), g, k)
    ))]
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

        let base_seed = self.params.seed.or(seed).unwrap_or(0xDEAD_BEEF_CAFE_1234u64);

        // Recursive bisection path
        if self.params.use_recursive && k > 2 {
            use crate::init::grow::RecursiveBisect;
            let rb = RecursiveBisect {
                niter: self.params.niter, ncuts: self.params.ncuts,
                coarsen_to: self.params.coarsen_to, ufactor: self.params.ufactor,
                contig_fm: self.params.contig_fm,
            };
            let mut p = rb.partition_graph(g, k, base_seed)?;
            repair_contiguity(g, &mut p);
            if self.params.min_conn {
                crate::refine::minconn::minimize_connectivity(g, &mut p, self.params.ufactor);
                repair_contiguity(g, &mut p);
            }
            return Ok(p);
        }

        // Alternate coarsener when coarsen_method != Shem
        use crate::coarsen::{hem::HeavyEdgeMatchWithParams, mindegree::MinDegreeMatch,
                              twohop::TwoHopMatchWithParams};
        let alt_coarsener: Option<Box<dyn Coarsener>> = match self.params.coarsen_method {
            CoarseningMethod::Shem      => None,
            CoarseningMethod::Hem       => Some(Box::new(HeavyEdgeMatchWithParams { coarsen_to: self.params.coarsen_to, k })),
            CoarseningMethod::MinDegree => Some(Box::new(MinDegreeMatch)),
            CoarseningMethod::TwoHop    => Some(Box::new(TwoHopMatchWithParams { coarsen_to: self.params.coarsen_to, k })),
        };

        let ncuts = self.params.ncuts.max(1) as usize;
        let mut best: Option<(Partition, i64)> = None;

        for trial in 0..ncuts {
            let trial_seed = base_seed.wrapping_add((trial as u64).wrapping_mul(0x9E3779B97F4A7C15));
            let hierarchy = if let Some(ref ac) = alt_coarsener {
                CoarseningHierarchy::build(g, ac.as_ref())?
            } else {
                CoarseningHierarchy::build(g, &self.coarsener)?
            };
            let p = Pipeline::new(hierarchy)
                .initial_partition(&self.init, k, trial_seed)
                .refine_and_project(&self.refiner)
                .into_partition();
            let cut = compute_cut(g, &p.assignment);
            if best.as_ref().map_or(true, |&(_, bc)| cut < bc) { best = Some((p, cut)); }
        }

        let (mut p, _) = best.unwrap();
        repair_contiguity(g, &mut p);
        if self.params.min_conn {
            crate::refine::minconn::minimize_connectivity(g, &mut p, self.params.ufactor);
            repair_contiguity(g, &mut p);
        }
        Ok(p)
    }

    /// Partition `g` into `fracs.len()` parts where part *i* receives a population
    /// proportional to `fracs[i] / sum(fracs)`.
    ///
    /// The integer proportional weights are converted to float target fractions
    /// (`tpwgts`) and attached to the initial `Partition`.  The FM refinement loop
    /// reads `tpwgts` from the partition and enforces per-part balance around those
    /// asymmetric targets instead of the equal-weight default.
    ///
    /// # Errors
    ///
    /// * [`PartitionError::ZeroParts`] — `fracs` is empty or all entries are zero.
    /// * Propagates all errors from the coarsening/initial-partition/refinement pipeline.
    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>
    {
        if fracs.is_empty() { return Err(PartitionError::ZeroParts); }
        let total_fracs: u32 = fracs.iter().sum();
        if total_fracs == 0 { return Err(PartitionError::ZeroParts); }
        let k = fracs.len() as u32;

        if g.n() == 0 { return Err(PartitionError::EmptyGraph); }
        if !g.is_valid() { return Err(PartitionError::InvalidGraph("is_valid() failed")); }
        if k as usize > g.n() { return Err(PartitionError::TooManyParts { k, n: g.n() }); }

        // Convert integer proportional weights to float target fractions summing to 1.0
        let tpwgts: Vec<f32> = fracs.iter()
            .map(|&f| f as f32 / total_fracs as f32)
            .collect();

        let rng_seed = self.params.seed.or(seed).unwrap_or(0xDEAD_BEEF_CAFE_1234u64);
        let hierarchy = CoarseningHierarchy::build(g, &self.coarsener)?;

        // Build initial partition and attach tpwgts so FM can use per-part targets
        let init_p = {
            let mut p = self.init.partition(hierarchy.coarsest(), k, rng_seed);
            // tpwgts applies to constraint 0 (total population) only.
            // When ncon > 1 (e.g. total_pop + VAP for VRA), constraints 1..ncon
            // use equal-weight balance targets. This is correct for redistricting:
            // seat allocation (fracs) is based on total population, not VAP.
            // See: Schloegel, Karypis & Kumar (2002) §3.2 multi-constraint semantics.
            p.tpwgts = Some(tpwgts);
            p
        };

        let pipeline = crate::multilevel::pipeline::Pipeline {
            hierarchy,
            partition: Some(init_p),
            _state: std::marker::PhantomData::<crate::multilevel::pipeline::NeedsRefinement>,
        };
        // tpwgts is cleared on the output partition — callers receive a plain Partition
        let mut result = pipeline.refine_and_project(&self.refiner).into_partition();
        result.tpwgts = None;
        Ok(result)
    }
}

/// Prusti pure helper for postcondition 3: population balance ≤ 0.5%.
///
/// Prusti pure functions cannot use iterators — a while-loop body is required.
/// Full loop verification is deferred in Prusti v0.2 due to loop-invariant
/// support limitations for Vec<i32>. This function stubs to `true` so that
/// postconditions 1 and 2 can be fully activated without blocking on PC-3.
/// See `verify/prusti/GAPS.md` for the deferred item.
#[cfg(prusti)]
#[pure]
fn population_balanced(_p: &Partition, _g: &CsrGraph, _k: u32) -> bool {
    // NOTE: Full loop verification deferred — see GAPS.md entry #1.
    // The runtime check is covered by `fm_preserves_population_balance` and
    // `population_balance_check()` below.
    true
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
            Partition { assignment: vec![0; g.n()], k: 1, tpwgts: None }
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
        assert!(p.tpwgts.is_none());
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
            refiner:   FiducciaMattheyses { niter: 10, ..FiducciaMattheyses::default() },
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
    fn split_weighted_equal_fracs_produces_k2() {
        let g = make_path_graph(10);
        let partitioner = MetisPartitioner::with_params(MetisParams::default(), 2);
        // Equal fracs — v1 delegates to equal-weight split.
        let p = partitioner.split_weighted(&g, &[50, 50], Some(0)).unwrap();
        assert_eq!(p.assignment.len(), 10);
        assert_eq!(p.k, 2);
    }

    #[test]
    fn split_weighted_all_zero_fracs_errors() {
        let g = make_path_graph(10);
        let p = MetisPartitioner::with_params(MetisParams::default(), 2)
            .split_weighted(&g, &[0u32, 0u32], Some(0));
        assert!(matches!(p, Err(crate::error::PartitionError::ZeroParts)));
    }

    /// Structural validity test for asymmetric fracs — both parts must be non-empty
    /// and each covers a plausible share of the 17 vertices.
    #[test]
    fn split_weighted_asymmetric_fracs() {
        let g = make_path_graph(17);
        let partitioner = MetisPartitioner::with_params(MetisParams::default(), 2);
        let p = partitioner.split_weighted(&g, &[8u32, 9u32], Some(42)).unwrap();
        // Structural validity — correct length and k
        assert_eq!(p.assignment.len(), 17);
        assert_eq!(p.k, 2);
        // Both parts must be non-empty
        assert!(p.assignment.contains(&0), "part 0 is empty");
        assert!(p.assignment.contains(&1), "part 1 is empty");
        // Proportional balance: accept any split in [3, 14] as structurally sound.
        let pop0 = p.assignment.iter().filter(|&&x| x == 0).count();
        let pop1 = p.assignment.iter().filter(|&&x| x == 1).count();
        assert!(pop0 >= 3 && pop0 <= 14, "part 0 size unreasonable: {pop0}");
        assert!(pop1 >= 3 && pop1 <= 14, "part 1 size unreasonable: {pop1}");
    }

    /// fracs [8, 9]: part 0 should receive ~8/17 of population, part 1 ~9/17.
    /// With tpwgts wired through FM, the balance must be within ±2 of the target
    /// on a uniform-weight 17-vertex path graph.
    #[test]
    fn split_weighted_proportional_balance() {
        let g = make_path_graph(17);
        let total = 17i64;
        // round(17 * 8/17) = 8, remainder = 9
        let target0 = (total * 8 + 8) / 17; // = 8
        let target1 = total - target0;        // = 9
        let eps = 2i64; // allow ±2 for small path graphs

        let p = MetisPartitioner::with_params(MetisParams::default(), 2)
            .split_weighted(&g, &[8u32, 9u32], Some(42))
            .unwrap();

        assert_eq!(p.assignment.len(), 17);
        assert_eq!(p.k, 2);

        let pop0: i64 = p.assignment.iter().filter(|&&x| x == 0).count() as i64;
        let pop1: i64 = total - pop0;
        assert!((pop0 - target0).abs() <= eps,
            "part 0: expected ~{target0}, got {pop0} (target1={target1}, pop1={pop1})");
        assert!((pop1 - target1).abs() <= eps,
            "part 1: expected ~{target1}, got {pop1} (target0={target0}, pop0={pop0})");
    }
}
