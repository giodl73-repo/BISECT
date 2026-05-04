/// `redist sweep` — N-seed optimization planning tool (v1: command generator).
///
/// Validates sweep parameters, generates plan labels, and prints the equivalent
/// `redist state` shell commands for the user to run. Does NOT execute them in v1.
/// Use `--run` in a future version to auto-execute.
use crate::args::SweepArgs;

const VALID_METRICS: &[&str] = &["compactness", "splits", "deviation"];

/// Validate sweep arguments and return an error message if invalid.
pub fn validate_sweep_args(args: &SweepArgs) -> Result<(), String> {
    if args.seeds < 1 {
        return Err("--seeds must be at least 1".into());
    }
    if args.keep_top > args.seeds {
        return Err(format!(
            "--keep-top ({}) cannot exceed --seeds ({})",
            args.keep_top, args.seeds
        ));
    }
    if !VALID_METRICS.contains(&args.optimize_by.as_str()) {
        return Err(format!(
            "invalid --optimize-by '{}'; valid values: {}",
            args.optimize_by,
            VALID_METRICS.join(", ")
        ));
    }
    Ok(())
}

/// Generate plan labels following the naming convention:
/// `{state}_{chamber}_seed_{seed_number}`
pub fn generate_sweep_labels(args: &SweepArgs) -> Vec<String> {
    let state = args.state.to_lowercase();
    let chamber = &args.chamber;
    (args.seed_start..args.seed_start + args.seeds as u64)
        .map(|seed| format!("{state}_{chamber}_seed_{seed}"))
        .collect()
}

/// Run the sweep planning command.
pub fn run_sweep(args: &SweepArgs) -> anyhow::Result<()> {
    validate_sweep_args(args)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let labels = generate_sweep_labels(args);

    eprintln!("=== redist sweep: {} seeds, keep top {} by {} ===",
        args.seeds, args.keep_top, args.optimize_by);
    eprintln!();
    eprintln!("Would run {} plans for {} {} (year={}, version={}):",
        args.seeds, args.state, args.chamber, args.year, args.version);
    eprintln!();

    for (i, label) in labels.iter().enumerate() {
        let seed_val = args.seed_start + i as u64;
        eprintln!("  [{}] {}", i + 1, label);
        println!("redist state --state {} --year {} --chamber {} --label {} --seed {} --version {} --output-base {}",
            args.state, args.year, args.chamber, label, seed_val, args.version, args.output_base);
        println!("redist analyze --state {} --year {} --label {} --version {} --types compactness splits summary",
            args.state, args.year, label, args.version);
        eprintln!();
    }

    eprintln!("After running all seeds, compare with:");
    eprintln!("  redist compare --labels {} --optimize-by {}",
        labels.iter().take(args.keep_top).cloned().collect::<Vec<_>>().join(","),
        args.optimize_by);
    eprintln!();
    eprintln!("Report: Would run {} plans, keeping top {} by {}",
        args.seeds, args.keep_top, args.optimize_by);

    if args.run {
        eprintln!("NOTE: --run flag is reserved for v2 auto-execution. Not implemented yet.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::SweepArgs;

    fn make_sweep_args(state: &str, seeds: usize, keep_top: usize, optimize_by: &str) -> SweepArgs {
        SweepArgs {
            state: state.into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            seeds,
            seed_start: 1,
            keep_top,
            optimize_by: optimize_by.into(),
            version: "sweep".into(),
            output_base: "outputs".into(),
            run: false,
        }
    }

    #[test]
    fn test_sweep_args_parse() {
        // Basic parsing via SweepArgs struct construction
        let args = make_sweep_args("WA", 10, 3, "compactness");
        assert_eq!(args.state, "WA");
        assert_eq!(args.seeds, 10);
        assert_eq!(args.keep_top, 3);
        assert_eq!(args.optimize_by, "compactness");
    }

    #[test]
    fn test_sweep_keep_top_exceeds_seeds_is_caught() {
        let args = make_sweep_args("WA", 5, 10, "compactness"); // keep_top > seeds
        let result = validate_sweep_args(&args);
        assert!(result.is_err(), "keep_top > seeds must return error");
        let msg = result.unwrap_err();
        assert!(msg.contains("keep-top") || msg.contains("keep_top"),
            "error must mention keep-top: {msg}");
        assert!(msg.contains("seeds") || msg.contains("10"),
            "error must mention seeds: {msg}");
    }

    #[test]
    fn test_sweep_optimize_by_invalid_is_caught() {
        let args = make_sweep_args("WA", 10, 3, "gerrymandering"); // invalid metric
        let result = validate_sweep_args(&args);
        assert!(result.is_err(), "invalid optimize_by must return error");
        let msg = result.unwrap_err();
        assert!(msg.contains("gerrymandering") || msg.contains("invalid"),
            "error must mention the invalid value: {msg}");
    }

    #[test]
    fn test_sweep_generates_correct_labels() {
        let args = make_sweep_args("WA", 3, 2, "splits");
        let labels = generate_sweep_labels(&args);
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0], "wa_congressional_seed_1");
        assert_eq!(labels[1], "wa_congressional_seed_2");
        assert_eq!(labels[2], "wa_congressional_seed_3");
    }

    #[test]
    fn test_sweep_generates_labels_with_custom_seed_start() {
        let mut args = make_sweep_args("TX", 2, 1, "deviation");
        args.seed_start = 42;
        let labels = generate_sweep_labels(&args);
        assert_eq!(labels[0], "tx_congressional_seed_42");
        assert_eq!(labels[1], "tx_congressional_seed_43");
    }

    #[test]
    fn test_sweep_valid_metrics_accepted() {
        for metric in &["compactness", "splits", "deviation"] {
            let args = make_sweep_args("CA", 5, 3, metric);
            let result = validate_sweep_args(&args);
            assert!(result.is_ok(), "metric '{}' must be valid: {:?}", metric, result.err());
        }
    }

    #[test]
    fn test_sweep_keep_top_equal_seeds_is_ok() {
        let args = make_sweep_args("VT", 5, 5, "compactness"); // keep_top == seeds: ok
        let result = validate_sweep_args(&args);
        assert!(result.is_ok(), "keep_top == seeds must be ok: {:?}", result.err());
    }

    // ── Task 212: generated commands go to stdout ─────────────────────────────

    #[test]
    fn test_sweep_generates_redist_state_commands() {
        // The generated labels should contain the state and chamber
        let mut args = make_sweep_args("WA", 3, 1, "compactness");
        args.chamber = "house".to_string();
        let labels = generate_sweep_labels(&args);
        assert_eq!(labels.len(), 3);
        assert!(
            labels[0].contains("wa") || labels[0].contains("WA"),
            "label must reference state: {}", labels[0]
        );
        assert!(
            labels[0].contains("house"),
            "label must reference chamber: {}", labels[0]
        );
    }

    #[test]
    fn test_sweep_labels_contain_seed_numbers() {
        let args = make_sweep_args("TX", 3, 2, "splits");
        let labels = generate_sweep_labels(&args);
        // Each label should contain the seed number
        assert!(labels[0].contains("1"), "first label must contain seed 1: {}", labels[0]);
        assert!(labels[1].contains("2"), "second label must contain seed 2: {}", labels[1]);
        assert!(labels[2].contains("3"), "third label must contain seed 3: {}", labels[2]);
    }

    // ── 15 additional L0 tests ───────────────────────────────────────────────

    // -- validate_sweep_args -------------------------------------------------

    #[test]
    fn test_validate_sweep_zero_seeds_is_error() {
        let args = make_sweep_args("WA", 0, 0, "compactness");
        let result = validate_sweep_args(&args);
        assert!(result.is_err(), "zero seeds must return error");
        let msg = result.unwrap_err();
        assert!(msg.contains("seeds") || msg.contains("1"),
            "error must mention minimum seeds: {msg}");
    }

    #[test]
    fn test_validate_sweep_keep_top_zero_with_positive_seeds_is_ok() {
        // keep_top=0 <= seeds=5: boundary case, should be allowed
        let args = make_sweep_args("WA", 5, 0, "compactness");
        let result = validate_sweep_args(&args);
        assert!(result.is_ok(), "keep_top=0 with seeds>0 must be valid: {:?}", result.err());
    }

    #[test]
    fn test_validate_sweep_keep_top_one_of_one_seed_is_ok() {
        let args = make_sweep_args("VT", 1, 1, "splits");
        assert!(validate_sweep_args(&args).is_ok(), "seeds=1 keep_top=1 must be valid");
    }

    #[test]
    fn test_validate_sweep_all_valid_metrics() {
        for metric in VALID_METRICS {
            let args = make_sweep_args("CA", 3, 2, metric);
            let result = validate_sweep_args(&args);
            assert!(result.is_ok(), "metric '{}' must be valid: {:?}", metric, result.err());
        }
    }

    #[test]
    fn test_validate_sweep_empty_metric_is_invalid() {
        let args = make_sweep_args("WA", 5, 2, "");
        let result = validate_sweep_args(&args);
        assert!(result.is_err(), "empty metric string must fail");
    }

    #[test]
    fn test_validate_sweep_case_sensitive_metric() {
        // Metric names are case-sensitive: "Compactness" != "compactness"
        let args = make_sweep_args("WA", 3, 1, "Compactness");
        let result = validate_sweep_args(&args);
        assert!(result.is_err(), "Capitalized metric must fail (case-sensitive)");
    }

    // -- generate_sweep_labels -----------------------------------------------

    #[test]
    fn test_generate_sweep_labels_single_seed() {
        let args = make_sweep_args("VT", 1, 1, "compactness");
        let labels = generate_sweep_labels(&args);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0], "vt_congressional_seed_1");
    }

    #[test]
    fn test_generate_sweep_labels_state_lowercased() {
        // State code must always be lower-cased in the label
        let args = make_sweep_args("CA", 2, 1, "splits");
        let labels = generate_sweep_labels(&args);
        for label in &labels {
            assert!(label.starts_with("ca_"), "label must start with 'ca_': {label}");
            assert!(!label.contains("CA"), "label must not contain uppercase 'CA': {label}");
        }
    }

    #[test]
    fn test_generate_sweep_labels_large_seed_count() {
        let mut args = make_sweep_args("TX", 100, 10, "compactness");
        args.seed_start = 0;
        let labels = generate_sweep_labels(&args);
        assert_eq!(labels.len(), 100, "100 seeds must produce 100 labels");
        assert_eq!(labels[0], "tx_congressional_seed_0");
        assert_eq!(labels[99], "tx_congressional_seed_99");
    }

    #[test]
    fn test_generate_sweep_labels_seed_start_zero() {
        let mut args = make_sweep_args("WA", 3, 2, "compactness");
        args.seed_start = 0;
        let labels = generate_sweep_labels(&args);
        assert_eq!(labels[0], "wa_congressional_seed_0");
        assert_eq!(labels[1], "wa_congressional_seed_1");
        assert_eq!(labels[2], "wa_congressional_seed_2");
    }

    #[test]
    fn test_generate_sweep_labels_senate_chamber() {
        let mut args = make_sweep_args("WA", 2, 1, "compactness");
        args.chamber = "senate".to_string();
        let labels = generate_sweep_labels(&args);
        assert!(labels[0].contains("senate"), "senate chamber must appear in label: {}", labels[0]);
        assert!(!labels[0].contains("congressional"), "congressional must not appear: {}", labels[0]);
    }

    // -- run_sweep integration (pure logic only) -----------------------------

    #[test]
    fn test_run_sweep_returns_ok_for_valid_args() {
        let args = make_sweep_args("VT", 3, 2, "compactness");
        let result = run_sweep(&args);
        assert!(result.is_ok(), "run_sweep with valid args must return Ok: {:?}", result.err());
    }

    #[test]
    fn test_run_sweep_returns_err_for_invalid_metric() {
        let args = make_sweep_args("VT", 3, 2, "invalid_metric");
        let result = run_sweep(&args);
        assert!(result.is_err(), "run_sweep with invalid metric must return Err");
    }

    #[test]
    fn test_run_sweep_run_flag_does_not_error() {
        // --run flag is reserved for v2 (currently a no-op note); must not cause Err
        let mut args = make_sweep_args("VT", 2, 1, "splits");
        args.run = true;
        let result = run_sweep(&args);
        assert!(result.is_ok(), "run_sweep with --run flag must still return Ok: {:?}", result.err());
    }

    #[test]
    fn test_generate_sweep_labels_count_matches_seeds() {
        for seeds in [1, 5, 10, 50] {
            let args = make_sweep_args("WA", seeds, seeds, "compactness");
            let labels = generate_sweep_labels(&args);
            assert_eq!(labels.len(), seeds,
                "label count must equal seeds={seeds}");
        }
    }
}
