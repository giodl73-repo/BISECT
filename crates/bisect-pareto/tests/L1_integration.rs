//! L1 integration tests for bisect-pareto.
//!
//! All tests use synthetic graphs and run unconditionally (no #[ignore]).
//! Per spec §12 (L1 invariants).

use bisect_pareto::crossover::{crossover, is_plan_valid};
use bisect_pareto::dominance::{dominates, fast_non_dominated_sort};
use bisect_pareto::mutation::mutate;
use bisect_pareto::{run_nsga2, ParetoConfig, ParetoResult};

fn path_adj(n: usize) -> Vec<Vec<usize>> {
    (0..n)
        .map(|i| {
            let mut nb = Vec::new();
            if i > 0 {
                nb.push(i - 1);
            }
            if i < n - 1 {
                nb.push(i + 1);
            }
            nb
        })
        .collect()
}

fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<usize>> {
    let n = rows * cols;
    let mut adj = vec![vec![]; n];
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols {
                adj[v].push(v + 1);
                adj[v + 1].push(v);
            }
            if r + 1 < rows {
                adj[v].push(v + cols);
                adj[v + cols].push(v);
            }
        }
    }
    adj
}

fn is_connected(tracts: &[usize], adj: &[Vec<usize>]) -> bool {
    if tracts.is_empty() {
        return true;
    }
    let set: std::collections::HashSet<usize> = tracts.iter().copied().collect();
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(tracts[0]);
    visited.insert(tracts[0]);
    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v] {
            if set.contains(&nb) && !visited.contains(&nb) {
                visited.insert(nb);
                queue.push_back(nb);
            }
        }
    }
    visited.len() == tracts.len()
}

fn check_plan_contiguous(plan: &[u32], k: usize, adj: &[Vec<usize>]) -> bool {
    let n = plan.len();
    for d in 1u32..=k as u32 {
        let tracts: Vec<usize> = (0..n).filter(|&t| plan[t] == d).collect();
        if tracts.is_empty() {
            return false;
        }
        if !is_connected(&tracts, adj) {
            return false;
        }
    }
    true
}

// ── L1.1: 4-node path, k=2, N_pop=10, N_gen=3 ────────────────────────────────

#[test]
fn nsga2_4node_path_k2_returns_valid_frontier() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let config = ParetoConfig {
        n_population: 10,
        n_generations: 3,
        base_seed: 42,
        balance_tolerance: 0.5,
    };
    let result = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    assert!(!result.frontier.is_empty(), "frontier must be non-empty");

    for entry in &result.frontier {
        let plan = &entry.plan;
        assert_eq!(plan.len(), 4, "plan must cover all 4 tracts");
        for &d in plan {
            assert!(d >= 1 && d <= 2, "district ID out of range [1,2]: {d}");
        }
        // Both districts non-empty
        assert!(plan.iter().any(|&d| d == 1), "district 1 must be non-empty");
        assert!(plan.iter().any(|&d| d == 2), "district 2 must be non-empty");
        // Contiguous
        assert!(
            check_plan_contiguous(plan, 2, &adj),
            "all districts must be contiguous"
        );
        // Not dominated
        assert!(!entry.dominated, "frontier plans must have dominated=false");
        assert_eq!(
            entry.validity_status.as_deref(),
            Some("valid"),
            "frontier plans should carry plan-level validity status"
        );
    }
}

// ── L1.2: Frontier is mutually non-dominated ──────────────────────────────────

#[test]
fn frontier_is_mutually_non_dominated() {
    let adj = path_adj(6);
    let pop = vec![100i64; 6];
    let config = ParetoConfig {
        n_population: 10,
        n_generations: 5,
        base_seed: 7,
        balance_tolerance: 0.5,
    };
    let result = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    let objectives: Vec<_> = result
        .frontier
        .iter()
        .map(|e| e.objectives.clone())
        .collect();

    // No plan in the frontier should be dominated by another plan in the frontier
    for i in 0..objectives.len() {
        for j in 0..objectives.len() {
            if i == j {
                continue;
            }
            assert!(
                !dominates(&objectives[j], &objectives[i]),
                "plan {i} is dominated by plan {j} — frontier not Pareto-optimal"
            );
        }
    }
}

// ── L1.3: Determinism ─────────────────────────────────────────────────────────

#[test]
fn nsga2_deterministic() {
    let adj = path_adj(6);
    let pop = vec![100i64; 6];
    let config = ParetoConfig {
        n_population: 8,
        n_generations: 3,
        base_seed: 99,
        balance_tolerance: 0.5,
    };

    let r1 = run_nsga2(&adj, &pop, 2, None, None, &[], config.clone()).unwrap();
    let r2 = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    assert_eq!(
        r1.frontier.len(),
        r2.frontier.len(),
        "same seed -> same frontier size"
    );

    // Sort frontiers by plan content for deterministic comparison
    let mut plans1: Vec<Vec<u32>> = r1.frontier.iter().map(|e| e.plan.clone()).collect();
    let mut plans2: Vec<Vec<u32>> = r2.frontier.iter().map(|e| e.plan.clone()).collect();
    plans1.sort();
    plans2.sort();
    assert_eq!(plans1, plans2, "same seed -> identical frontier plans");
}

// ── L1.4: NDJSON roundtrip ────────────────────────────────────────────────────

#[test]
fn ndjson_roundtrip() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let config = ParetoConfig {
        n_population: 5,
        n_generations: 2,
        base_seed: 42,
        balance_tolerance: 0.5,
    };
    let result = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    let mut buf = Vec::new();
    result.write_ndjson(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();

    let meta = ParetoResult::read_metadata_from_ndjson(&s)
        .expect("metadata must be present and parseable");

    // Check required metadata fields
    assert_eq!(meta["n_population"], 5, "n_population must match config");
    assert_eq!(meta["n_generations"], 2, "n_generations must match config");
    assert_eq!(meta["base_seed"], 42, "base_seed must match config");
    assert_eq!(meta["pareto_version"], "1.0", "pareto_version must be 1.0");
    assert_eq!(
        meta["pareto_front_size"].as_u64().unwrap() as usize,
        result.frontier.len(),
        "pareto_front_size must match frontier length"
    );

    // SHA-256 is 64 hex chars
    let sha = meta["file_sha256"].as_str().unwrap();
    assert_eq!(sha.len(), 64, "file_sha256 must be 64 hex chars");
}

// ── L1.5: NDJSON uses LF not CRLF ────────────────────────────────────────────

#[test]
fn ndjson_lf_not_crlf() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let config = ParetoConfig {
        n_population: 5,
        n_generations: 2,
        base_seed: 1,
        balance_tolerance: 0.5,
    };
    let result = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    let mut buf = Vec::new();
    result.write_ndjson(&mut buf).unwrap();

    assert!(
        !buf.windows(2).any(|w| w == b"\r\n"),
        "NDJSON output must not contain CRLF"
    );
    assert!(
        buf.iter().any(|&b| b == b'\n'),
        "NDJSON output must contain LF"
    );
}

// ── L1.6: Crossover validity ──────────────────────────────────────────────────

#[test]
fn crossover_validity() {
    let adj = path_adj(8);
    let pop = vec![100i64; 8];
    let pa = vec![1u32, 1, 1, 1, 2, 2, 2, 2];
    let pb = vec![1u32, 1, 2, 2, 1, 1, 2, 2];

    for seed in [0u64, 1, 2, 42, 99, 12345] {
        let result = crossover(&pa, &pb, &adj, &pop, 2, 0.5, seed);
        assert_eq!(result.len(), 8, "result must have 8 tracts");
        for &d in &result {
            assert!(d >= 1 && d <= 2, "invalid district: {d}");
        }
        // Must be parent_a OR a valid plan
        let is_pa = result == pa;
        let is_valid = is_plan_valid(&result, &adj, &pop, 2, 0.5);
        assert!(
            is_pa || is_valid,
            "crossover must return parent_a or valid plan (seed={seed})"
        );
    }
}

// ── L1.7: Mutation validity ───────────────────────────────────────────────────

#[test]
fn mutation_validity() {
    let adj = path_adj(8);
    let pop = vec![100i64; 8];
    let plan = vec![1u32, 1, 1, 1, 2, 2, 2, 2];

    for seed in [0u64, 1, 5, 42, 100, 999] {
        let result = mutate(&plan, &adj, &pop, 2, 0.5, seed);
        assert_eq!(result.len(), 8);
        for &d in &result {
            assert!(d >= 1 && d <= 2, "invalid district: {d}");
        }
        let is_unchanged = result == plan;
        let is_valid = is_plan_valid(&result, &adj, &pop, 2, 0.5);
        assert!(
            is_unchanged || is_valid,
            "mutation must return original or valid plan (seed={seed})"
        );
    }
}

// ── L1.8: Mutation with no valid flips ───────────────────────────────────────

#[test]
fn mutation_no_valid_flips_returns_unchanged() {
    // k=2, 2-node graph: each district is a single tract
    // Any flip would either violate balance or disconnect a district
    let adj = vec![vec![1usize], vec![0usize]];
    let pop = vec![100i64; 2];
    let plan = vec![1u32, 2];

    for seed in [0u64, 1, 42, 99] {
        let result = mutate(&plan, &adj, &pop, 2, 0.0, seed);
        assert_eq!(
            result, plan,
            "degenerate graph: mutation must return unchanged plan (seed={seed})"
        );
    }
}

// ── L1.9: Generation_found within bounds ─────────────────────────────────────

#[test]
fn generation_found_within_bounds() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let n_gen = 5;
    let config = ParetoConfig {
        n_population: 8,
        n_generations: n_gen,
        base_seed: 0,
        balance_tolerance: 0.5,
    };
    let result = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    for entry in &result.frontier {
        assert!(
            entry.generation_found <= n_gen,
            "generation_found {} must be <= n_generations {}",
            entry.generation_found,
            n_gen
        );
    }
}

// ── L1.10: dominated=false for all frontier entries ───────────────────────────

#[test]
fn frontier_all_dominated_false() {
    let adj = path_adj(6);
    let pop = vec![100i64; 6];
    let config = ParetoConfig {
        n_population: 10,
        n_generations: 3,
        base_seed: 55,
        balance_tolerance: 0.5,
    };
    let result = run_nsga2(&adj, &pop, 2, None, None, &[], config).unwrap();

    for (i, entry) in result.frontier.iter().enumerate() {
        assert!(
            !entry.dominated,
            "frontier entry {i}: dominated must be false"
        );
    }
}

// ── L1.11: Dominance test on constructed plans ────────────────────────────────

#[test]
fn dominance_sort_lower_ec_wins() {
    use bisect_pareto::objectives::Objectives;

    // Plan A: lower EC, same D_seats, same VRA_deficit
    let a = Objectives {
        ec: 100.0,
        d_seats: 5.0,
        vra_deficit: 0.0,
    };
    let b = Objectives {
        ec: 200.0,
        d_seats: 5.0,
        vra_deficit: 0.0,
    };

    let fronts = fast_non_dominated_sort(&[a, b]);
    assert_eq!(fronts.len(), 2, "A dominates B -> 2 fronts");
    assert_eq!(fronts[0], vec![0], "A (lower EC) in front 0");
    assert_eq!(fronts[1], vec![1], "B in front 1");
}
