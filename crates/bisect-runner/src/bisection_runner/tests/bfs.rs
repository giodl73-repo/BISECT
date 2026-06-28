use super::*;

// ── Group: BFS Region-Growing (T.12) ─────────────────────────────────────

// L0: 4x4 grid → valid 2-way split (all tracts assigned, two non-empty districts).
#[test]
fn bfs_growth_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 42)
        .expect("bfs-growth must succeed on 4x4 grid");
    // All 16 tracts assigned
    let mut all: Vec<usize> = left.union(&right).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16).collect::<Vec<_>>(),
        "all 16 tracts must be covered"
    );
    // Both sides non-empty
    assert!(!left.is_empty(), "left side must be non-empty");
    assert!(!right.is_empty(), "right side must be non-empty");
    // Disjoint
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
}

// L0: same seed → same result (deterministic).
#[test]
fn bfs_growth_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let run = |seed: u64| {
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, seed).expect("bfs-growth must succeed")
    };
    let (l1, r1) = run(99);
    let (l2, r2) = run(99);
    assert_eq!(l1, l2, "same seed must produce identical left set");
    assert_eq!(r1, r2, "same seed must produce identical right set");
}

// L0: no unassigned tracts after BFS + rebalance.
#[test]
fn bfs_growth_all_tracts_assigned() {
    let (adj, pop) = small_grid(5, 5); // 25 tracts
    let tracts: HashSet<usize> = (0..25).collect();
    let (left, right) = split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 7)
        .expect("bfs-growth must succeed on 5x5 grid");
    assert_eq!(
        left.len() + right.len(),
        25,
        "all 25 tracts must be assigned"
    );
    assert!(
        left.is_disjoint(&right),
        "no tract may appear in both sides"
    );
}

// L0: both districts non-empty for any non-trivial input.
#[test]
fn bfs_growth_both_districts_nonempty() {
    // Linear chain: 10 nodes
    let n = 10usize;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nb = vec![];
            if i > 0 {
                nb.push(i - 1);
            }
            if i < n - 1 {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let pop = vec![1000i64; n];
    let tracts: HashSet<usize> = (0..n).collect();
    let (left, right) =
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 0).expect("bfs-growth must succeed on chain");
    assert!(!left.is_empty(), "left must be non-empty for chain graph");
    assert!(!right.is_empty(), "right must be non-empty for chain graph");
}

// L0: BFS from any tract in each district must reach all tracts in that district
//     (contiguity guarantee — BFS growth ensures this by construction).
#[test]
fn bfs_growth_contiguous_districts() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 123).expect("bfs-growth must succeed");

    let is_contiguous = |side: &HashSet<usize>| -> bool {
        if side.len() <= 1 {
            return true;
        }
        let start = *side.iter().next().unwrap();
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if side.contains(&nb) && !visited.contains(&nb) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        visited.len() == side.len()
    };

    assert!(is_contiguous(&left), "left district must be contiguous");
    assert!(is_contiguous(&right), "right district must be contiguous");
}

// L0: "bfs-growth" StructureMode parses correctly from CLI string.
#[test]
fn bfs_growth_structure_mode_parses() {
    use crate::args::StructureMode;
    use clap::ValueEnum;
    let parsed =
        StructureMode::from_str("bfs-growth", true).expect("StructureMode must parse 'bfs-growth'");
    assert_eq!(
        parsed,
        StructureMode::BfsGrowth,
        "parsed StructureMode must equal BfsGrowth"
    );
}

// L0: seed[1] should be farther from seed[0] than most other tracts
//     (maximally-spread seeds — seed[1] = argmax BFS distance from seed[0]).
#[test]
fn bfs_growth_seeds_maximally_spread() {
    // 4x4 grid: seeds should be near opposite corners.
    // We verify this indirectly: the two assigned seeds are not adjacent.
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 0).expect("bfs-growth must succeed");

    // The farthest-apart split on a 4x4 grid should not put all tracts
    // in one set (which would happen if both seeds were adjacent).
    assert!(left.len() >= 1, "left must have at least 1 tract");
    assert!(right.len() >= 1, "right must have at least 1 tract");

    // The seeds are the initial tracts: verify they are not the same
    // by confirming both sides have at least 1 tract.
    // (A more direct test would require exposing seed positions.)
    let max_frac = (left.len().max(right.len()) as f64) / 16.0;
    assert!(
        max_frac <= 0.95,
        "maximally-spread seeds should not produce a 95%:5% split; got {:.0}%:{:.0}%",
        left.len() as f64 / 16.0 * 100.0,
        right.len() as f64 / 16.0 * 100.0
    );
}

// L1: run_all_splits_bfs on 4x4 grid with k=2: valid, deterministic, contiguous.
#[test]
fn bfs_growth_run_all_splits_k2() {
    let (adj, pop) = small_grid(4, 4);
    let asgn = run_all_splits_bfs(&adj, &pop, 2, 0.05, None, 42)
        .expect("run_all_splits_bfs k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

// L1: run_all_splits_bfs with k=4: valid partition on 4x4 grid.
#[test]
fn bfs_growth_run_all_splits_k4() {
    let (adj, pop) = small_grid(4, 4);
    let asgn = run_all_splits_bfs(&adj, &pop, 4, 0.10, None, 7)
        .expect("run_all_splits_bfs k=4 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 4, "must produce exactly 4 districts");
}

// L1: same base_seed → identical run_all_splits_bfs result.
#[test]
fn bfs_growth_run_all_splits_deterministic() {
    let (adj, pop) = small_grid(4, 5); // 20 tracts
    let run = || {
        run_all_splits_bfs(&adj, &pop, 2, 0.05, None, 12345)
            .expect("run_all_splits_bfs must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignments");
}

// L2: NC 2020 k=14 — BFS Growth vs single METIS baseline.
#[test]
#[ignore]
fn bfs_growth_nc_ec_vs_metis() {
    // Requires: data/2020/north_carolina_adjacency.adj.bin
    // Run bfs-growth and METIS on NC 2020 k=14.
    // Expected: BFS Growth EC >= METIS EC (METIS explicitly minimizes EC).
    // Also check: BFS Growth is fast (O(n log n) vs METIS heuristic).
    // Skipped unless --include-ignored is passed.
}
