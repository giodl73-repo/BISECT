//! L1 integration tests — correctness oracle and termination checks.

use redist_metis::graph::CsrGraph;
use redist_metis::coarsen::Coarsener;
use redist_metis::coarsen::shem::SortedHeavyEdgeMatchWithParams;

fn make_path(n: usize) -> CsrGraph {
    let mut xadj = vec![0u32];
    let mut adjncy = Vec::new();
    for i in 0..n {
        if i > 0 { adjncy.push((i - 1) as u32); }
        if i < n - 1 { adjncy.push((i + 1) as u32); }
        xadj.push(adjncy.len() as u32);
    }
    CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
}

/// L1: repeated coarsening of a 255-vertex path reaches should_stop within MAX_LEVELS=50
#[test]
fn coarsening_terminates_path255() {
    let g = make_path(255);
    let coarsener = SortedHeavyEdgeMatchWithParams { coarsen_to: 20, k: 1 };
    let mut current = g;
    for level in 0..50usize {
        if coarsener.should_stop(&current) { return; }
        let (next, _) = coarsener.coarsen(&current);
        assert!(next.is_valid(), "invalid coarsened graph at level {level}");
        assert!(next.n() < current.n(),
            "graph did not shrink at level {level}: {} -> {}", current.n(), next.n());
        current = next;
    }
    panic!("coarsening did not reach should_stop within 50 levels (final n={})", current.n());
}
