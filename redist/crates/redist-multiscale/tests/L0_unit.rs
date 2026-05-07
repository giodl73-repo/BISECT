//! L0/L1/L2 integration tests for the redist-multiscale crate.
//!
//! L0: inline unit properties (no external data).
//! L1: synthetic adjacency — small graphs, observable behaviour.
//! L2: real-data TX k=38 (marked #[ignore], requires census data).

use redist_multiscale::{
    hierarchy::HierarchyLevel,
    chain::{MultiScaleConfig, MultiScaleError},
    rebalance::rebalance,
    seeds::step_seed,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a simple path graph: 0-1-2-..-(n-1).
fn path_adj_usize(n: usize) -> Vec<Vec<usize>> {
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

// ── HierarchyLevel tests ──────────────────────────────────────────────────────

/// from_fine must preserve total population: sum(coarse_pop) == sum(fine_pop).
#[test]
fn hierarchy_construction_preserves_population() {
    // 6-tract path grouped 2+2+2 into 3 coarse nodes.
    let fine_adj = path_adj_usize(6);
    let fine_pop = vec![100i64, 200, 150, 75, 125, 50];
    let partition = vec![0usize, 0, 1, 1, 2, 2];
    let level = HierarchyLevel::from_fine(&fine_adj, &fine_pop, &partition, 3);

    let total_fine: i64 = fine_pop.iter().sum();
    let total_coarse: i64 = level.pop.iter().sum();
    assert_eq!(
        total_fine, total_coarse,
        "coarse population must equal fine population total"
    );
}

// ── rebalance tests ───────────────────────────────────────────────────────────

/// rebalance on an already-balanced plan must return true and leave the plan unchanged.
#[test]
fn rebalance_already_balanced_noop() {
    let adj = path_adj_usize(4);
    let pop = vec![100i64; 4];
    // Districts 1,1,2,2 — perfectly balanced.
    let mut asgn = vec![1u32, 1, 2, 2];
    let original = asgn.clone();
    let ok = rebalance(&mut asgn, &adj, &pop, 2, 0.1, 200);
    assert!(ok, "already-balanced plan must return true");
    assert_eq!(asgn, original, "balanced plan must be unchanged");
}

/// rebalance must fix a simple imbalance (3 tracts in district 1, 1 in district 2).
#[test]
fn rebalance_fixes_simple_imbalance() {
    let adj = path_adj_usize(4);
    let pop = vec![100i64; 4];
    // Imbalanced: 3 tracts in district 1, 1 in district 2.
    let mut asgn = vec![1u32, 1, 1, 2];
    let result = rebalance(&mut asgn, &adj, &pop, 2, 0.1, 200);
    assert!(result, "rebalance must succeed on fixable imbalance");
    let pop1: i64 = asgn
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d == 1)
        .map(|(t, _)| pop[t])
        .sum();
    let pop2: i64 = asgn
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d == 2)
        .map(|(t, _)| pop[t])
        .sum();
    // After rebalance, both districts should be within 10% of ideal (200).
    assert!(
        (pop1 - 200).abs() <= 40,
        "district 1 pop {pop1} not within 10% of 200"
    );
    assert!(
        (pop2 - 200).abs() <= 40,
        "district 2 pop {pop2} not within 10% of 200"
    );
}

// ── seeds tests ───────────────────────────────────────────────────────────────

/// "MSC_STEP_" prefix must produce a different seed value than "MS_STEP_" for the
/// same inputs — computed directly with SHA-256 to verify prefix distinctness.
#[test]
fn step_seed_distinct_from_merge_split_prefix() {
    use sha2::{Digest, Sha256};

    // Compute the MSC_STEP_ seed via the public API.
    let msc_seed = step_seed(42, 0, 0);

    // Compute what "MS_STEP_" would give for the same inputs (Merge-Split prefix).
    let mut h = Sha256::new();
    h.update(b"MS_STEP_");
    h.update(0u64.to_le_bytes());
    h.update(b"_");
    h.update(0u32.to_le_bytes());
    h.update(b"_");
    h.update(42u64.to_le_bytes());
    let d = h.finalize();
    let ms_seed = u64::from_le_bytes(d[..8].try_into().unwrap());

    assert_ne!(
        msc_seed, ms_seed,
        "MSC_STEP_ prefix must produce different value than MS_STEP_ for same inputs"
    );
}

// ── MultiScaleConfig tests ────────────────────────────────────────────────────

/// Default MultiScaleConfig must have coarse_tol == 2.0 * pop_tolerance.
#[test]
fn multiscale_config_coarse_tol_default() {
    let cfg = MultiScaleConfig::default();
    assert!(
        (cfg.coarse_tol - 2.0 * cfg.pop_tolerance).abs() < 1e-12,
        "coarse_tol must equal 2 x pop_tolerance by default, got coarse_tol={} pop_tolerance={}",
        cfg.coarse_tol,
        cfg.pop_tolerance
    );
}

/// MissingCoarseAdjacency error must mention block-group in its Display string.
#[test]
fn multiscale_error_mentions_block_group() {
    let err = MultiScaleError::MissingCoarseAdjacency;
    let msg = err.to_string();
    assert!(
        msg.contains("block-group") || msg.contains("block_group"),
        "MissingCoarseAdjacency error must mention block-group: {msg}"
    );
}

// ── L2: real-data mixing test (ignored by default) ────────────────────────────

/// L2: TX k=38, multi-scale mixing faster than standard ReCom.
/// Requires: real TX 2020 tract + block-group adjacency files.
/// Skipped unless --include-ignored is passed.
#[test]
#[ignore]
fn multiscale_tx_mixing_faster_than_recom() {
    // Placeholder: load TX 2020 adjacency, run MultiScaleChain vs plain ReCom for 5000 steps,
    // compare autocorrelation of partisan-lean metric. Assert ESS(multiscale) > ESS(recom).
}
