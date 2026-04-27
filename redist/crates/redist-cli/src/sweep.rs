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

    println!("=== redist sweep: {} seeds, keep top {} by {} ===",
        args.seeds, args.keep_top, args.optimize_by);
    println!();
    println!("Would run {} plans for {} {} (year={}, version={}):",
        args.seeds, args.state, args.chamber, args.year, args.version);
    println!();

    for (i, label) in labels.iter().enumerate() {
        let seed_val = args.seed_start + i as u64;
        println!("  [{}] {}", i + 1, label);
        println!("    redist state --state {} --year {} --chamber {} --label {} --seed {} --version {} --output-base {}",
            args.state, args.year, args.chamber, label, seed_val, args.version, args.output_base);
        println!("    redist analyze --state {} --year {} --label {} --version {} --types compactness splits summary",
            args.state, args.year, label, args.version);
        println!();
    }

    println!("After running all seeds, compare with:");
    println!("  redist compare --labels {} --optimize-by {}",
        labels.iter().take(args.keep_top).cloned().collect::<Vec<_>>().join(","),
        args.optimize_by);
    println!();
    println!("Report: Would run {} plans, keeping top {} by {}",
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
}
