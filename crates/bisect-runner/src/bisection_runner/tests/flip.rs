use super::*;

// ── Flip search tests ─────────────────────────────────────────────────────

// L0: zero steps returns the initial plan (visited list has exactly 1 entry).
#[test]
fn flip_zero_steps_returns_initial_plan() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    let (asgn, visited_count, rank) = run_flip_chain(&adj, &pop, &ew, 2, 0.05, 0, 42, 0.0)
        .expect("flip_chain must succeed with 0 steps");
    assert_eq!(
        visited_count, 1,
        "0 steps => visited has 1 entry (initial plan)"
    );
    assert_eq!(rank, 0, "only one plan, rank must be 0");
    assert_eq!(asgn.len(), 16, "all 16 tracts assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "2 districts");
}

// L0: visited list is never empty (initial plan always included).
#[test]
fn flip_visited_count_ge_1() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    // Even with 0 steps, visited must be >= 1.
    let (_, visited_count, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.05, 0, 99, 0.0).expect("flip_chain must succeed");
    assert!(
        visited_count >= 1,
        "visited list must contain at least the initial plan"
    );
}

// L0: p=0.0 selects plan with EC <= plan selected by p=1.0 (sort ascending).
#[test]
fn flip_p0_le_flip_p1_ec() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    let (asgn_p0, _, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.1, 200, 42, 0.0).expect("flip p=0.0 must succeed");
    let (asgn_p1, _, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.1, 200, 42, 1.0).expect("flip p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: determinism — same seed produces the same result.
#[test]
fn flip_deterministic() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    let (asgn1, v1, r1) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.05, 100, 777, 0.5).expect("first run must succeed");
    let (asgn2, v2, r2) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.05, 100, 777, 0.5).expect("second run must succeed");
    assert_eq!(v1, v2, "visited_count must be the same for same seed");
    assert_eq!(r1, r2, "rank must be the same for same seed");
    assert_eq!(asgn1, asgn2, "assignment must be identical for same seed");
}

// L0: all-fail case — tiny graph where all flips violate balance.
// Use 2 tracts of very unequal weight and zero tolerance: flip always violates balance.
#[test]
fn flip_all_fail_returns_initial_plan() {
    // 4-node path, all equal pop, very tight tolerance (0.0 -> max_dev = 1)
    // With some effort flips will be accepted on balanced graph, but let's use
    // a disconnected scenario: 2 isolated nodes each assigned to their own district.
    // No boundary tracts -> boundary is empty -> all steps are no-ops.
    let adj = vec![vec![], vec![]]; // 2 isolated nodes
    let pop = vec![1000i64, 1000];
    let ew = HashMap::new();
    let (asgn, visited_count, _) = run_flip_chain(&adj, &pop, &ew, 2, 0.0, 100, 42, 0.0)
        .expect("flip_chain on isolated graph must succeed");
    assert_eq!(
        visited_count, 1,
        "no boundary tracts -> no flips -> only initial plan"
    );
    assert_eq!(asgn.len(), 2);
}

// L1: valid partition — all tracts assigned, no overlap, correct district count.
#[test]
fn flip_produces_valid_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let (asgn, _, _) = run_flip_chain(&adj, &pop, &ew, 4, 0.1, 500, 42, 0.0)
        .expect("flip_chain must produce a valid partition");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    for i in 0..16 {
        let d = asgn[&i];
        assert!(
            d >= 1 && d <= 4,
            "district {d} out of range [1,4] for tract {i}"
        );
    }
    // No duplicate assignments (each tract assigned exactly once).
    let unique: std::collections::HashSet<usize> = asgn.keys().copied().collect();
    assert_eq!(unique.len(), 16, "each tract appears exactly once");
}

// L1: contiguity preserved — all districts remain contiguous after flips.
#[test]
fn flip_contiguity_preserved() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let (asgn, _, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.1, 500, 123, 0.5).expect("flip_chain must succeed");
    // Check contiguity of each district using BFS.
    let n = adj.len();
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    for d in &districts {
        let members: Vec<usize> = (0..n).filter(|&v| asgn[&v] == *d).collect();
        assert!(!members.is_empty(), "district {d} must be non-empty");
        if members.len() == 1 {
            continue;
        }
        let start = members[0];
        let mut visited_bfs = vec![false; n];
        visited_bfs[start] = true;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if !visited_bfs[nb] && asgn[&nb] == *d {
                    visited_bfs[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
        for &v in &members {
            assert!(
                visited_bfs[v],
                "district {d}: tract {v} is disconnected from the rest"
            );
        }
    }
}

// L2: real-data NC test (ignored by default).
#[test]
#[ignore]
fn flip_nc_improves_ec() {
    // Placeholder: load NC adjacency, compare flip p=0.0 EC vs baseline.
    // Skipped unless --include-ignored is passed.
}
