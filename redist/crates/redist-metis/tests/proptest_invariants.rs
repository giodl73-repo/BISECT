//! L0 property-based tests — CsrGraph invariants through coarsening.

use proptest::prelude::*;
use redist_metis::graph::CsrGraph;
use redist_metis::coarsen::Coarsener;
use redist_metis::coarsen::shem::SortedHeavyEdgeMatch;

fn arb_path(max_n: usize) -> impl Strategy<Value = CsrGraph> {
    (2usize..=max_n).prop_map(|n| {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i-1) as u32); }
            if i < n-1 { adjncy.push((i+1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    })
}

proptest! {
    #[test]
    fn coarsen_preserves_validity(g in arb_path(32)) {
        let (coarsened, cmap) = SortedHeavyEdgeMatch.coarsen(&g);
        prop_assert!(coarsened.is_valid(),
            "coarsened graph must satisfy is_valid()");
        prop_assert_eq!(cmap.cmap.len(), g.n(),
            "cmap length must equal fine graph vertex count");
        prop_assert!(coarsened.n() < g.n(),
            "coarsened graph must be strictly smaller");
    }

    #[test]
    fn coarsen_cmap_targets_in_range(g in arb_path(32)) {
        let (coarsened, cmap) = SortedHeavyEdgeMatch.coarsen(&g);
        for &t in &cmap.cmap {
            prop_assert!((t as usize) < coarsened.n(),
                "cmap target {} out of range (coarsened n={})", t, coarsened.n());
        }
    }
}
