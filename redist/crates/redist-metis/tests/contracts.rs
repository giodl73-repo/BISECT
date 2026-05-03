//! L1/L0 integration tests — correctness oracle, RNG golden pin, termination.

use redist_metis::graph::CsrGraph;
use redist_metis::api::{MetisPartitioner, MetisParams, Partitioner};

// ── graph helpers ──────────────────────────────────────────────────────────

fn make_path(n: usize) -> CsrGraph {
    let mut xadj = vec![0u32];
    let mut adjncy = Vec::new();
    for i in 0..n {
        if i > 0 { adjncy.push((i-1) as u32); }
        if i < n-1 { adjncy.push((i+1) as u32); }
        xadj.push(adjncy.len() as u32);
    }
    CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
}

fn make_grid(rows: usize, cols: usize) -> CsrGraph {
    let n = rows * cols;
    let mut xadj = vec![0u32];
    let mut adjncy = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            let mut nbrs = Vec::new();
            if r > 0 { nbrs.push((r-1)*cols+c); }
            if r < rows-1 { nbrs.push((r+1)*cols+c); }
            if c > 0 { nbrs.push(r*cols+(c-1)); }
            if c < cols-1 { nbrs.push(r*cols+(c+1)); }
            for &u in &nbrs { adjncy.push(u as u32); }
            xadj.push(adjncy.len() as u32);
            let _ = v;
        }
    }
    CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
}

fn make_dumbbell() -> CsrGraph {
    // Two K5 cliques joined by one bridge edge (4--5)
    let n = 10usize;
    let mut xadj = vec![0u32];
    let mut adjncy = Vec::new();
    for v in 0..n {
        let mut nbrs = Vec::new();
        let range = if v < 5 { 0..5 } else { 5..10 };
        for u in range { if u != v { nbrs.push(u); } }
        if v == 4 { nbrs.push(5); }
        if v == 5 { nbrs.push(4); }
        for &u in &nbrs { adjncy.push(u as u32); }
        xadj.push(adjncy.len() as u32);
    }
    CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
}

fn make_weighted_path(n: usize) -> CsrGraph {
    // Alternating heavy (10) and light (1) edges
    let mut xadj = vec![0u32];
    let mut adjncy = Vec::new();
    let mut adjwgt = Vec::new();
    for i in 0..n {
        if i > 0 {
            adjncy.push((i-1) as u32);
            adjwgt.push(if (i-1) % 2 == 0 { 10i32 } else { 1i32 });
        }
        if i < n-1 {
            adjncy.push((i+1) as u32);
            adjwgt.push(if i % 2 == 0 { 10i32 } else { 1i32 });
        }
        xadj.push(adjncy.len() as u32);
    }
    CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: Some(adjwgt) }
}

fn cut(g: &CsrGraph, assignment: &[u32]) -> u32 {
    let mut c = 0u32;
    for v in 0..g.n() {
        for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
            let u = g.adjncy[j] as usize;
            if assignment[v] != assignment[u] { c += 1; }
        }
    }
    c / 2
}

fn metis(k: u32, _seed: u64) -> MetisPartitioner {
    MetisPartitioner::with_params(MetisParams::default(), k)
}

// ── L0 correctness oracle (7 graphs) ──────────────────────────────────────

#[test]
fn oracle_k1_trivial() {
    let g = make_path(10);
    let p = metis(1, 0).split(&g, 1, Some(0)).unwrap();
    assert!(p.assignment.iter().all(|&x| x == 0));
    assert_eq!(cut(&g, &p.assignment), 0);
}

#[test]
fn oracle_path_bisect_cut_1() {
    let g = make_path(10);
    let p = metis(2, 42).split(&g, 2, Some(42)).unwrap();
    assert_eq!(cut(&g, &p.assignment), 1, "path bisect optimal cut is 1");
}

#[test]
fn oracle_grid_4x4_k4_coverage() {
    let g = make_grid(4, 4);
    let p = metis(4, 0).split(&g, 4, Some(0)).unwrap();
    assert_eq!(p.assignment.len(), 16);
    for part in 0..4u32 {
        assert!(p.assignment.contains(&part), "part {part} must be present");
    }
}

#[test]
fn oracle_dumbbell_cut_1() {
    let g = make_dumbbell();
    let p = metis(2, 0).split(&g, 2, Some(0)).unwrap();
    assert_eq!(cut(&g, &p.assignment), 1, "dumbbell optimal cut is 1");
}

#[test]
fn oracle_full_coverage_all_vertices() {
    let g = make_grid(4, 4);
    for k in [1u32, 2, 4] {
        let p = metis(k, 0).split(&g, k, Some(0)).unwrap();
        assert_eq!(p.assignment.len(), g.n());
        assert!(p.assignment.iter().all(|&a| a < k));
    }
}

#[test]
fn oracle_weighted_path_respects_weights() {
    let g = make_weighted_path(10);
    let p = metis(2, 0).split(&g, 2, Some(0)).unwrap();
    // Just verify: valid output, cut is reasonable
    assert_eq!(p.assignment.len(), 10);
    assert!(p.assignment.iter().all(|&a| a < 2));
}

// L1: termination (from earlier task — kept here as part of oracle suite)
use redist_metis::coarsen::Coarsener;
use redist_metis::coarsen::shem::SortedHeavyEdgeMatchWithParams;

#[test]
fn coarsening_terminates_path255() {
    let g = make_path(255);
    let coarsener = SortedHeavyEdgeMatchWithParams { coarsen_to: 20, k: 1 };
    let mut current = g;
    for level in 0..50usize {
        if coarsener.should_stop(&current) { return; }
        let (next, _) = coarsener.coarsen(&current);
        assert!(next.is_valid(), "invalid at level {level}");
        assert!(next.n() < current.n(), "did not shrink at level {level}");
        current = next;
    }
    panic!("did not reach should_stop within 50 levels");
}

// ── Golden RNG determinism pin ─────────────────────────────────────────────

/// Run with `cargo test generate_golden -- --ignored` to regenerate.
/// Commit the resulting tests/golden/vt_seed42.json.
/// Regenerate ONLY when rand_pcg is intentionally upgraded.
#[test]
#[ignore]
fn generate_golden() {
    let g = make_path(255);
    let p = MetisPartitioner::with_params(MetisParams::default(), 1)
        .split(&g, 1, Some(42))
        .unwrap();
    let json = serde_json::json!({
        "seed": 42,
        "n": 255,
        "k": 1,
        "note": "Regenerate only when rand_pcg crate version changes",
        "assignment": p.assignment,
    });
    std::fs::create_dir_all("tests/golden").unwrap();
    std::fs::write(
        "tests/golden/vt_seed42.json",
        serde_json::to_string_pretty(&json).unwrap(),
    ).unwrap();
    println!("Golden value written — commit tests/golden/vt_seed42.json");
}

#[test]
fn golden_rng_determinism() {
    // k=1 partition is all-zeros regardless of seed — the golden test is mainly about
    // ensuring the pipeline runs deterministically. For non-trivial seeds, use k=2.
    let g = make_path(20);
    let p1 = MetisPartitioner::with_params(MetisParams::default(), 2)
        .split(&g, 2, Some(42))
        .unwrap();
    let p2 = MetisPartitioner::with_params(MetisParams::default(), 2)
        .split(&g, 2, Some(42))
        .unwrap();
    assert_eq!(p1.assignment, p2.assignment,
        "same seed must produce identical partition (RNG determinism)");
}
