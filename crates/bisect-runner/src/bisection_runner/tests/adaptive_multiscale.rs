use super::*;

// ── AdaptiveMultiScale tests ──────────────────────────────────────────────

// L0: missing geoids must return Err containing "GEOID".
#[test]
fn multiscale_adaptive_missing_geoids_returns_err() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let cfg = AdaptiveConfig::default();
    let result = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        42,
        cfg,
        None,
        MultiscaleFineLevel::Tract,
        "county",
        None,
    );
    assert!(
        result.is_err(),
        "run_multiscale_adaptive with no geoids must return Err"
    );
    let msg = result.unwrap_err();
    assert!(
        msg.contains("GEOID"),
        "error must mention GEOID, got: {msg}"
    );
}

// L0: T=200, adapt_interval=50 -> alpha_trace.len() == 4.
#[test]
fn multiscale_adaptive_alpha_trace_length() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let cfg = AdaptiveConfig {
        total_steps: 200,
        adapt_interval: 50,
        ..AdaptiveConfig::default()
    };
    let (_, result) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        42,
        cfg,
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("multiscale_adaptive must succeed on 4x4 grid");
    assert_eq!(
        result.alpha_trace.len(),
        4,
        "T=200 adapt_interval=50 must produce exactly 4 adaptation rounds, \
             got {}",
        result.alpha_trace.len()
    );
}

// L0: adapt_interval > total_steps -> no adaptation, alpha_trace empty, final_alpha == initial_alpha.
#[test]
fn multiscale_adaptive_adapt_interval_gt_steps_no_adaptation() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let initial_alpha = 0.30;
    let cfg = AdaptiveConfig {
        total_steps: 10,
        adapt_interval: 1000,
        initial_alpha,
        ..AdaptiveConfig::default()
    };
    let (_, result) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        42,
        cfg,
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("multiscale_adaptive must succeed");
    assert!(
        result.alpha_trace.is_empty(),
        "adapt_interval > total_steps must produce empty alpha_trace"
    );
    assert!(
        (result.final_alpha - initial_alpha).abs() < 1e-12,
        "adapt_interval > total_steps: final_alpha must equal initial_alpha, \
             got {}",
        result.final_alpha
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn multiscale_adaptive_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let make_cfg = || AdaptiveConfig {
        total_steps: 100,
        adapt_interval: 25,
        ..AdaptiveConfig::default()
    };
    let (plan1, res1) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        77,
        make_cfg(),
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("first run must succeed");
    let (plan2, res2) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        77,
        make_cfg(),
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("second run must succeed");
    assert_eq!(plan1, plan2, "same seed must produce identical plan");
    assert_eq!(
        res1.alpha_trace, res2.alpha_trace,
        "same seed must produce identical alpha_trace"
    );
}

// L0: "multiscale-adaptive" must parse as SearchMode::MultiScaleAdaptive via clap ValueEnum.
#[test]
fn multiscale_adaptive_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed = SearchMode::from_str("multiscale-adaptive", true)
        .expect("SearchMode must parse 'multiscale-adaptive'");
    assert_eq!(parsed, SearchMode::MultiScaleAdaptive);
}
