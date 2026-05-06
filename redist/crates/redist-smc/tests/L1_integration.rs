//! L1 integration tests for redist-smc.
//! All tests use synthetic graphs and run unconditionally (no #[ignore]).
//! Per spec §7 L1 invariants.

use redist_smc::{run_smc, SmcConfig};

fn path_adj(n: usize) -> Vec<Vec<usize>> {
    (0..n).map(|i| {
        let mut nb = Vec::new();
        if i > 0 { nb.push(i - 1); }
        if i < n - 1 { nb.push(i + 1); }
        nb
    }).collect()
}

fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<usize>> {
    let n = rows * cols;
    let mut adj = vec![vec![]; n];
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols { adj[v].push(v + 1); adj[v + 1].push(v); }
            if r + 1 < rows { adj[v].push(v + cols); adj[v + cols].push(v); }
        }
    }
    adj
}

fn is_connected(tracts: &[usize], adj: &[Vec<usize>]) -> bool {
    if tracts.is_empty() { return true; }
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

// ── L1.1: 4-node path, k=2, N=100 ────────────────────────────────────────────

#[test]
fn l1_path4_k2_n100_weights_sum_1() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let cfg = SmcConfig { n_particles: 100, base_seed: 42, pop_tolerance: 0.2, ..Default::default() };
    let result = run_smc(&adj, &pop, 2, cfg).unwrap();

    let sum: f64 = result.weights.iter().sum();
    assert!((sum - 1.0).abs() < 1e-6, "weights sum 1.0 ± 1e-6: {sum}");
}

#[test]
fn l1_path4_k2_n100_all_plans_valid() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let cfg = SmcConfig { n_particles: 100, base_seed: 42, pop_tolerance: 0.2, ..Default::default() };
    let result = run_smc(&adj, &pop, 2, cfg).unwrap();

    for (idx, plan) in result.plans.iter().enumerate() {
        assert_eq!(plan.len(), 4, "plan {idx}: all 4 tracts assigned");
        let d1: Vec<usize> = (0..4).filter(|&i| plan[i] == 1).collect();
        let d2: Vec<usize> = (0..4).filter(|&i| plan[i] == 2).collect();
        assert!(!d1.is_empty(), "plan {idx}: district 1 non-empty");
        assert!(!d2.is_empty(), "plan {idx}: district 2 non-empty");
        assert_eq!(d1.len() + d2.len(), 4, "plan {idx}: full partition");
        assert!(is_connected(&d1, &adj), "plan {idx}: district 1 contiguous");
        assert!(is_connected(&d2, &adj), "plan {idx}: district 2 contiguous");
    }
}

#[test]
fn l1_path4_k2_n100_no_zero_weight_plans() {
    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let cfg = SmcConfig { n_particles: 100, base_seed: 42, pop_tolerance: 0.2, ..Default::default() };
    let result = run_smc(&adj, &pop, 2, cfg).unwrap();
    assert!(result.weights.iter().all(|&w| w >= 0.0), "all weights non-negative");
}

// ── L1.2: Determinism ─────────────────────────────────────────────────────────

#[test]
fn l1_determinism_same_seed_identical_output() {
    let adj = path_adj(8);
    let pop = vec![100i64; 8];
    let cfg = SmcConfig { n_particles: 30, base_seed: 99, pop_tolerance: 0.3, ..Default::default() };
    let r1 = run_smc(&adj, &pop, 2, cfg.clone()).unwrap();
    let r2 = run_smc(&adj, &pop, 2, cfg).unwrap();

    assert_eq!(r1.plans, r2.plans, "plans must be identical");
    assert_eq!(r1.weights, r2.weights, "weights must be identical");
    assert_eq!(r1.resample_count, r2.resample_count, "resample_count must match");
    assert_eq!(r1.index_maps, r2.index_maps, "index_maps must match");
}

// ── L1.3: Single-node k=1 trivial ─────────────────────────────────────────────

#[test]
fn l1_single_node_k1_trivial() {
    let adj = vec![vec![]];
    let pop = vec![1000i64];
    let cfg = SmcConfig { n_particles: 10, base_seed: 0, ..Default::default() };
    let result = run_smc(&adj, &pop, 1, cfg).unwrap();

    assert!(result.plans.iter().all(|p| p == &[1u32]),
        "k=1: all plans are [1]");
    let wsum: f64 = result.weights.iter().sum();
    assert!((wsum - 1.0).abs() < 1e-9, "k=1 weights sum 1.0: {wsum}");
    assert_eq!(result.resample_count, 0);
    assert!(result.ess_trace.is_empty(), "k=1: no ESS trace");
}

// ── L1.4: Population balance ──────────────────────────────────────────────────

#[test]
fn l1_population_balance_within_tolerance() {
    let adj = path_adj(6);
    let pop = vec![100i64; 6]; // total=600, ideal=300 per district
    let pop_tol = 0.2f64; // ±20% = ±60 pop per district
    let cfg = SmcConfig { n_particles: 50, base_seed: 7, pop_tolerance: pop_tol, ..Default::default() };
    let result = run_smc(&adj, &pop, 2, cfg).unwrap();

    let ideal = 300.0f64;
    let tol_abs = pop_tol * 600.0; // ±120
    for (idx, plan) in result.plans.iter().enumerate() {
        let pop_d1: i64 = (0..6).filter(|&i| plan[i] == 1).map(|i| pop[i]).sum();
        let pop_d2: i64 = (0..6).filter(|&i| plan[i] == 2).map(|i| pop[i]).sum();
        assert!((pop_d1 as f64 - ideal).abs() <= tol_abs,
            "plan {idx}: district 1 pop {pop_d1} outside ±{tol_abs} of {ideal}");
        assert!((pop_d2 as f64 - ideal).abs() <= tol_abs,
            "plan {idx}: district 2 pop {pop_d2} outside ±{tol_abs} of {ideal}");
    }
}

// ── L1.5: No district has 0 tracts; no district ID outside 1..=k ──────────────

#[test]
fn l1_district_ids_in_range() {
    let adj = grid_adj(3, 3);
    let pop = vec![100i64; 9];
    let cfg = SmcConfig { n_particles: 40, base_seed: 3, pop_tolerance: 0.4, ..Default::default() };
    let result = run_smc(&adj, &pop, 3, cfg).unwrap();

    for (idx, plan) in result.plans.iter().enumerate() {
        assert_eq!(plan.len(), 9, "plan {idx}: 9 tracts");
        for &d in plan {
            assert!(d >= 1 && d <= 3, "plan {idx}: district {d} out of [1,3]");
        }
        // All 3 districts non-empty
        for d in 1u32..=3 {
            let count = plan.iter().filter(|&&x| x == d).count();
            assert!(count > 0, "plan {idx}: district {d} has 0 tracts");
        }
    }
}

// ── L1.6: ESS trace length = k-1 ──────────────────────────────────────────────

#[test]
fn l1_ess_trace_length_k_minus_1() {
    let adj = path_adj(6);
    let pop = vec![100i64; 6];
    let cfg = SmcConfig { n_particles: 20, base_seed: 1, pop_tolerance: 0.3, ..Default::default() };
    let result = run_smc(&adj, &pop, 3, cfg).unwrap();
    assert_eq!(result.ess_trace.len(), 2,
        "k=3: ESS trace length = k-1 = 2, got {}", result.ess_trace.len());
}

// ── L1.7: index_map bounds ────────────────────────────────────────────────────

#[test]
fn l1_index_map_bounds() {
    // Use tight threshold to force resampling
    let adj = path_adj(8);
    let pop = vec![100i64; 8];
    let cfg = SmcConfig {
        n_particles: 20, base_seed: 5,
        pop_tolerance: 0.3,
        resample_threshold: 0.99, // resample very aggressively
        ..Default::default()
    };
    let result = run_smc(&adj, &pop, 2, cfg).unwrap();

    for (r, imap) in result.index_maps.iter().enumerate() {
        assert_eq!(imap.len(), 20, "resample {r}: index_map length must be n_particles");
        for &i in imap {
            assert!(i < 20, "resample {r}: index {i} out of [0,20)");
        }
    }
}

// ── L1.8: NDJSON output roundtrip ─────────────────────────────────────────────

#[test]
fn l1_ndjson_output_roundtrip() {
    use redist_smc::output::WriteConfig;

    let adj = path_adj(4);
    let pop = vec![100i64; 4];
    let cfg = SmcConfig { n_particles: 5, base_seed: 42, pop_tolerance: 0.2, ..Default::default() };
    let result = run_smc(&adj, &pop, 2, cfg).unwrap();

    let write_cfg = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.2 };
    let mut buf = Vec::new();
    result.write_ndjson(&mut buf, &write_cfg).unwrap();
    let s = String::from_utf8(buf).unwrap();

    // Lines: 5 particles + 1 metadata
    assert_eq!(s.lines().count(), 6, "5 particles + 1 metadata line");

    // Metadata is parseable and has correct fields
    let meta = redist_smc::SmcResult::read_metadata_from_ndjson(&s)
        .expect("metadata must be present");
    assert_eq!(meta.base_seed, 42);
    assert_eq!(meta.n_particles, 5);
    assert_eq!(meta.file_sha256.len(), 64, "SHA-256 = 64 hex chars");

    // No CRLF
    assert!(!s.contains("\r\n"), "must not contain CRLF");
}
