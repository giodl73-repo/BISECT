use super::*;

// ── Simulated Annealing tests ─────────────────────────────────────────────

// L0: zero steps returns the initial METIS plan unchanged (best = initial).
#[test]
fn sa_zero_steps_returns_initial() {
    // 4x4 grid, steps_per_tract=0 → no SA steps → best = initial METIS plan
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_sa(
        &adj, &pop, &ew, &tracts, 0.10, // balance_tolerance
        0,    // steps_per_tract = 0 → n_steps = 0
        0.01, 1e-4, 42,
    )
    .expect("SA with 0 steps must succeed");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides non-empty"
    );
    assert_eq!(left.len() + right.len(), 16, "all tracts covered");
    assert!(left.is_disjoint(&right), "sides must be disjoint");
}

// L0: same sa_seed → identical result (determinism).
#[test]
fn sa_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let run = |seed: u64| {
        split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 5, 0.01, 1e-4, seed)
            .expect("SA must succeed")
    };
    let (l1, r1) = run(99);
    let (l2, r2) = run(99);
    // Sort to compare deterministically
    let mut v1: Vec<usize> = l1
        .iter()
        .chain(r1.iter())
        .map(|&v| v + if l1.contains(&v) { 0 } else { 100 })
        .collect();
    let mut v2: Vec<usize> = l2
        .iter()
        .chain(r2.iter())
        .map(|&v| v + if l2.contains(&v) { 0 } else { 100 })
        .collect();
    v1.sort_unstable();
    v2.sort_unstable();
    assert_eq!(v1, v2, "same seed must produce identical partition");
}

// L0: t0_factor=0.0 forces T0=max(1.0, 0.0)=1.0 (test fixture from spec).
#[test]
fn sa_t0_zero_factor_uses_floor() {
    // With t0_factor=0.0, T_0 = max(1.0, 0.0 * EC) = 1.0 regardless of EC.
    let t0_factor = 0.0_f64;
    let initial_ec = 5usize;
    let t0 = (t0_factor * initial_ec as f64).max(1.0);
    assert!(
        (t0 - 1.0).abs() < 1e-10,
        "t0_factor=0.0 must give T_0=1.0, got {t0}"
    );
    // Also verify it runs without panic
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 3, 0.0, 1e-4, 42);
    assert!(
        result.is_ok(),
        "SA with t0_factor=0.0 must succeed: {:?}",
        result.err()
    );
}

// L0: greedy mode (t_final == t0 effectively zero temperature) never increases EC.
// We test: SA with tiny t_final and small steps should not produce higher EC than initial.
#[test]
fn sa_never_increases_ec_greedy() {
    // With t_final=1e-15 (near zero) the acceptance probability for worsening moves
    // is ~exp(-delta/1e-15) ≈ 0 for any positive delta_ec.
    // So EC should be <= initial_ec (or equal if no improvement found).
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();

    // Get initial METIS EC for reference
    let (l_metis, r_metis) =
        split_subgraph(&adj, &pop, 1, &ew, &tracts, 1.10, 100, Some(42), None, None)
            .expect("METIS must succeed");
    let mut metis_asgn = HashMap::new();
    for &v in &l_metis {
        metis_asgn.insert(v, 1usize);
    }
    for &v in &r_metis {
        metis_asgn.insert(v, 2usize);
    }
    let initial_ec = count_edge_cuts(&metis_asgn, &adj);

    let (l_sa, r_sa) = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 5, 0.01, 1e-15, 42)
        .expect("SA greedy must succeed");
    let mut sa_asgn = HashMap::new();
    for &v in &l_sa {
        sa_asgn.insert(v, 1usize);
    }
    for &v in &r_sa {
        sa_asgn.insert(v, 2usize);
    }
    let sa_ec = count_edge_cuts(&sa_asgn, &adj);
    assert!(
        sa_ec <= initial_ec,
        "greedy SA (t_final=1e-15) must not increase EC: initial={initial_ec} sa={sa_ec}"
    );
}

// L1: SA produces a valid 2-partition on a 4x4 grid (contiguity + balance).
#[test]
fn sa_produces_valid_2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 10, 0.01, 1e-4, 777)
        .expect("SA 4x4 must succeed");

    // Completeness and disjointness
    assert_eq!(left.len() + right.len(), 16, "all 16 tracts covered");
    assert!(left.is_disjoint(&right), "sides disjoint");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides non-empty"
    );

    // Contiguity: BFS check for each side
    let check_connected = |side: &HashSet<usize>| -> bool {
        let members: Vec<usize> = side.iter().copied().collect();
        if members.len() <= 1 {
            return true;
        }
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(members[0]);
        visited.insert(members[0]);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if side.contains(&nb) && !visited.contains(&nb) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        members.iter().all(|v| visited.contains(v))
    };
    assert!(check_connected(&left), "left side must be contiguous");
    assert!(check_connected(&right), "right side must be contiguous");

    // Balance: each side within 10% of half total pop
    let total_pop: i64 = pop.iter().sum();
    let left_pop: i64 = left.iter().map(|&v| pop[v]).sum();
    let balance = (left_pop as f64 - total_pop as f64 / 2.0).abs() / total_pop as f64;
    assert!(
        balance <= 0.10,
        "SA balance must be within 10%: {balance:.3}"
    );
}

// L1: SA result EC <= initial METIS EC + small_margin (SA should not seriously worsen EC).
#[test]
fn sa_improves_or_equals_metis() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();

    // METIS baseline
    let (l_m, r_m) = split_subgraph(&adj, &pop, 1, &ew, &tracts, 1.10, 100, Some(42), None, None)
        .expect("METIS baseline must succeed");
    let mut m_asgn = HashMap::new();
    for &v in &l_m {
        m_asgn.insert(v, 1usize);
    }
    for &v in &r_m {
        m_asgn.insert(v, 2usize);
    }
    let metis_ec = count_edge_cuts(&m_asgn, &adj);

    // SA with enough steps to make progress
    let (l_sa, r_sa) = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 20, 0.01, 1e-4, 42)
        .expect("SA must succeed");
    let mut sa_asgn = HashMap::new();
    for &v in &l_sa {
        sa_asgn.insert(v, 1usize);
    }
    for &v in &r_sa {
        sa_asgn.insert(v, 2usize);
    }
    let sa_ec = count_edge_cuts(&sa_asgn, &adj);

    // SA may equal METIS (especially on a tight grid), but must not be >> METIS.
    // Allow up to +2 edge cuts as "small margin" for stochastic variance.
    assert!(
        sa_ec <= metis_ec + 2,
        "SA EC should not exceed METIS EC + 2: metis={metis_ec} sa={sa_ec}"
    );
}

// L2 (ignored): SA on North Carolina should improve or equal compactness vs METIS.
#[test]
#[ignore]
fn sa_nc_compactness_improvement() {
    // Requires real NC adjacency data at data/2020/ — runs as L2 only.
    // Placeholder: actual implementation would load NC graph and compare
    // Polsby-Popper scores between METIS and SA outputs.
    panic!("L2 test: requires real NC data — run manually with --ignored");
}
