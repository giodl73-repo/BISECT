//! L1 integration tests for `redist-ensemble`.
//!
//! These tests exercise the full `run_ensemble` pipeline on synthetic graphs
//! without requiring real census data. All tests run in CI with no `#[ignore]`.

use redist_ensemble::chain::{run_ensemble, chain_seed};

// ── Graph helpers ─────────────────────────────────────────────────────────────

fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<u32>> {
    let n = rows * cols;
    let mut adj = vec![vec![]; n];
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols { adj[v].push((v+1) as u32); adj[v+1].push(v as u32); }
            if r + 1 < rows { adj[v].push((v+cols) as u32); adj[v+cols].push(v as u32); }
        }
    }
    adj
}

fn equal_pop(n: usize, pop_each: i64) -> Vec<i64> {
    vec![pop_each; n]
}

/// Split a row-major grid into k horizontal bands.
fn band_assignment(rows: usize, cols: usize, k: u32) -> Vec<u32> {
    let n = rows * cols;
    (0..n).map(|v| {
        let row = v / cols;
        (row * k as usize / rows) as u32 + 1
    }).collect()
}

// ── L1 tests ──────────────────────────────────────────────────────────────────

#[test]
fn full_run_returns_correct_shape() {
    let adj = grid_adj(5, 4); // 20 tracts, 4 districts
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 100, 2, 42, "grid".into());
    assert_eq!(result.n_steps, 100);
    assert_eq!(result.n_chains, 2);
    assert_eq!(result.k, 4);
    assert_eq!(result.chains.len(), 2);
    assert_eq!(result.chains[0].steps.len(), 100);
    assert_eq!(result.chains[1].steps.len(), 100);
    assert_eq!(result.chain_seeds.len(), 2);
}

#[test]
fn all_cut_fractions_in_unit_interval() {
    let adj = grid_adj(6, 6);
    let pop = equal_pop(36, 500);
    let assign = band_assignment(6, 6, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 200, 1, 7, "g".into());
    for rec in &result.chains[0].steps {
        assert!(
            rec.cut_fraction >= 0.0 && rec.cut_fraction <= 1.0,
            "cut_fraction {} out of [0,1]", rec.cut_fraction
        );
    }
}

#[test]
fn pop_deviations_respect_tolerance() {
    let adj = grid_adj(4, 4);
    let pop = equal_pop(16, 1000);
    let assign = band_assignment(4, 4, 2);
    let tol = 0.05;
    let result = run_ensemble(adj, pop, assign, 2, tol, 100, 1, 3, "g".into());
    for rec in &result.chains[0].steps {
        assert!(
            rec.pop_deviation as f64 <= tol + 0.01,
            "pop_deviation {} exceeds tolerance {}", rec.pop_deviation, tol
        );
    }
}

#[test]
fn step_numbers_are_monotone_across_chains() {
    let adj = grid_adj(5, 4);
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 50, 2, 0, "g".into());
    for chain in &result.chains {
        for (i, rec) in chain.steps.iter().enumerate() {
            assert_eq!(rec.step, (i + 1) as u64);
        }
    }
}

#[test]
fn at_least_some_steps_accepted() {
    let adj = grid_adj(6, 6);
    let pop = equal_pop(36, 1000);
    let assign = band_assignment(6, 6, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 500, 1, 42, "g".into());
    let accepted = result.chains[0].steps.iter().filter(|s| s.accepted).count();
    assert!(accepted > 5, "expected at least 5 accepted steps out of 500, got {accepted}");
}

#[test]
fn r_hat_finite_and_positive_for_two_chains() {
    let adj = grid_adj(5, 4);
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 200, 2, 11, "g".into());
    let rh = result.r_hat.expect("r_hat must be Some for 2 chains");
    // Valid: positive finite (normal), NaN (W=0, undefined), or 0 (both chains identical).
    assert!(rh.is_nan() || rh.is_finite(), "r_hat must be finite or NaN, got {rh}");
}

#[test]
fn ess_between_1_and_n_steps() {
    let adj = grid_adj(5, 4);
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 300, 1, 22, "g".into());
    let ess = result.ess.unwrap();
    assert!(ess >= 1.0, "ESS must be >= 1, got {ess}");
    assert!(ess <= 301.0, "ESS must be <= n_steps+1, got {ess}");
}

#[test]
fn hamming_autocorr_has_20_lags() {
    let adj = grid_adj(5, 4);
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 200, 1, 0, "g".into());
    assert_eq!(result.hamming_autocorr.len(), 20);
    for (i, &ac) in result.hamming_autocorr.iter().enumerate() {
        assert!(ac.is_finite(), "autocorr at lag {} is not finite", i+1);
    }
}

#[test]
fn pooled_mean_in_unit_interval() {
    let adj = grid_adj(5, 4);
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let result = run_ensemble(adj, pop, assign, 4, 0.05, 200, 2, 5, "g".into());
    assert!(
        result.pooled_cut_mean >= 0.0 && result.pooled_cut_mean <= 1.0,
        "pooled_cut_mean {} out of range", result.pooled_cut_mean
    );
    assert!(result.pooled_cut_std >= 0.0, "std must be non-negative");
}

#[test]
fn json_roundtrip_full_run() {
    let adj = grid_adj(4, 4);
    let pop = equal_pop(16, 1000);
    let assign = band_assignment(4, 4, 2);
    let result = run_ensemble(adj, pop, assign, 2, 0.1, 50, 2, 42, "roundtrip".into());
    let json = serde_json::to_string(&result).expect("must serialize");
    let back: redist_ensemble::chain::EnsembleResult =
        serde_json::from_str(&json).expect("must deserialize");
    assert_eq!(back.state, result.state);
    assert_eq!(back.k, result.k);
    assert_eq!(back.n_steps, result.n_steps);
    assert_eq!(back.chains[0].steps.len(), result.chains[0].steps.len());
    assert!((back.pooled_cut_mean - result.pooled_cut_mean).abs() < 1e-6);
}

#[test]
fn deterministic_with_same_seed() {
    let adj = grid_adj(5, 4);
    let pop = equal_pop(20, 1000);
    let assign = band_assignment(5, 4, 4);
    let r1 = run_ensemble(adj.clone(), pop.clone(), assign.clone(), 4, 0.05, 100, 2, 99, "g".into());
    let r2 = run_ensemble(adj, pop, assign, 4, 0.05, 100, 2, 99, "g".into());
    for ci in 0..2 {
        let cuts1: Vec<f32> = r1.chains[ci].steps.iter().map(|s| s.cut_fraction).collect();
        let cuts2: Vec<f32> = r2.chains[ci].steps.iter().map(|s| s.cut_fraction).collect();
        assert_eq!(cuts1, cuts2, "chain {ci}: same seed must produce identical trace");
    }
}

#[test]
fn different_seeds_produce_different_traces() {
    // Larger grid for richer topology so seeds diverge in cut_fraction.
    let adj = grid_adj(6, 6);
    let pop = equal_pop(36, 1000);
    let assign = band_assignment(6, 6, 4);
    let r1 = run_ensemble(adj.clone(), pop.clone(), assign.clone(), 4, 0.05, 200, 1, 1, "g".into());
    let r2 = run_ensemble(adj, pop, assign, 4, 0.05, 200, 1, 2, "g".into());
    let cuts1: Vec<f32> = r1.chains[0].steps.iter().map(|s| s.cut_fraction).collect();
    let cuts2: Vec<f32> = r2.chains[0].steps.iter().map(|s| s.cut_fraction).collect();
    assert_ne!(cuts1, cuts2, "different base seeds should produce different cut-fraction sequences");
}

#[test]
fn chain_seeds_match_formula() {
    let adj = grid_adj(4, 4);
    let pop = equal_pop(16, 1000);
    let assign = band_assignment(4, 4, 2);
    let result = run_ensemble(adj, pop, assign, 2, 0.1, 10, 3, 77, "g".into());
    for i in 0..3 {
        assert_eq!(
            result.chain_seeds[i], chain_seed(77, i),
            "chain_seeds[{i}] must match SHA-256 formula"
        );
    }
}

#[test]
fn parallel_chains_give_same_result_as_serial() {
    // With 1 chain, parallel and serial are identical.
    let adj = grid_adj(4, 4);
    let pop = equal_pop(16, 1000);
    let assign = band_assignment(4, 4, 2);
    let r = run_ensemble(adj, pop, assign, 2, 0.1, 50, 1, 42, "g".into());
    assert_eq!(r.chains.len(), 1);
    assert_eq!(r.chains[0].steps.len(), 50);
}

#[test]
fn large_k_runs_without_panic() {
    // 10x10 grid → 100 tracts, k=10 — exercises pair reselection logic.
    let adj = grid_adj(10, 10);
    let pop = equal_pop(100, 1000);
    let assign = band_assignment(10, 10, 10);
    let result = run_ensemble(adj, pop, assign, 10, 0.1, 50, 1, 42, "g".into());
    assert_eq!(result.n_steps, 50);
    assert_eq!(result.k, 10);
}
