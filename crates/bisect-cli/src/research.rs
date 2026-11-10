//! Researcher Toolkit (`BISECT research ...`).
//!
//! Today's surface area:
//! - `BISECT research validate-ensemble` (RT Task 5 / M-02): consume an MCMC
//!   ensemble (one JSONL per chain), compute R-hat / ESS / optional Hamming
//!   autocorrelation per metric using the already-shipped
//!   `bisect_analysis::ensemble_diagnostics` math, and emit a
//!   `validate-ensemble.json` attesting whether the ensemble meets convergence
//!   thresholds. Optional `--enacted` flag computes per-metric percentile-rank
//!   of an enacted plan against the ensemble distribution — directly useful
//!   for state-court partisan-fairness litigation under the FAIRNESS_DOCTRINE
//!   strategy.
//!
//! Future subcommands (deferred):
//! - `BISECT research check-compat` (Task 3): GerryChain handshake.
//! - `BISECT research mcmc` (Task 6): wrapper around scripts/research/mcmc_ensemble.py.
//!
//! Input format for `validate-ensemble`:
//!
//! Each chain is a JSONL file. Each line is one MCMC step:
//!
//! ```json
//! {"step": 0, "metrics": {"efficiency_gap": 0.02, "mean_median": -0.01, "dem_seats": 4}}
//! {"step": 1, "metrics": {"efficiency_gap": 0.03, "mean_median": -0.02, "dem_seats": 5}}
//! ...
//! ```
//!
//! `--ensemble-dir` points at a directory; every `*.jsonl` file is treated as
//! one chain (chain_id = filename stem). R-hat requires >= 4 chains per S-03.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};



#[derive(Debug, clap::Subcommand)]
pub enum ResearchSubcommand {
    /// Compute convergence diagnostics for an MCMC ensemble (R-hat / ESS /
    /// optional percentile-rank of an enacted plan).
    ValidateEnsemble(ValidateEnsembleArgs),
}

#[derive(Debug, clap::Args)]
pub struct ValidateEnsembleArgs {
    /// Directory containing per-chain JSONL files (one chain per file). At
    /// least 4 files are required for R-hat per S-03.
    #[arg(long)]
    pub ensemble_dir: PathBuf,

    /// Path to write `validate-ensemble.json`. If omitted, writes to stdout.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// R-hat threshold above which a metric is flagged as non-converged.
    /// Default 1.05 per the standard Gelman-Rubin convention.
    #[arg(long, default_value_t = 1.05)]
    pub rhat_threshold: f64,

    /// Minimum acceptable Effective Sample Size per metric. Default 100,
    /// the conservative-but-conventional cutoff.
    #[arg(long, default_value_t = 100.0)]
    pub ess_min: f64,

    /// Optional path to an enacted plan's metrics (single JSON object,
    /// `{metric_name: value}`). When supplied, the report includes per-metric
    /// percentile-rank against the pooled ensemble distribution.
    #[arg(long)]
    pub enacted: Option<PathBuf>,

    /// Exit non-zero if any metric fails its R-hat or ESS threshold. Default
    /// is to exit 0 and surface the failures via the JSON report only.
    #[arg(long)]
    pub strict: bool,
}

// ---------------------------------------------------------------------------
// Input record: one line of the per-chain JSONL.
// ---------------------------------------------------------------------------

pub use bisect_analysis::research_ensemble::{EnsembleStep,MetricReport,ValidateEnsembleReport};
#[cfg(test)]
use bisect_analysis::research_ensemble::percentile_rank;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run_research(cmd: &ResearchSubcommand) -> anyhow::Result<()> {
    match cmd {
        ResearchSubcommand::ValidateEnsemble(args) => run_validate_ensemble(args),
    }
}

pub fn run_validate_ensemble(args: &ValidateEnsembleArgs) -> anyhow::Result<()> {
    if !args.ensemble_dir.is_dir() {
        anyhow::bail!(
            "[INPUT] --ensemble-dir does not exist or is not a directory: {}",
            args.ensemble_dir.display()
        );
    }

    // Load every *.jsonl file as one chain (chain_id = filename stem).
    let chains = load_chains_from_dir(&args.ensemble_dir)?;
    let enacted_metrics: Option<BTreeMap<String, f64>> = args.enacted.as_ref().map(|path| {
        let text=std::fs::read_to_string(path)?;
        bisect_analysis::research_ensemble::parse_enacted_metrics(&text)
    }).transpose()?;
    let report=bisect_analysis::research_ensemble::validate_ensemble(&chains,enacted_metrics.as_ref(),args.rhat_threshold,args.ess_min)?;

    let report_json = serde_json::to_string_pretty(&report)?;
    if let Some(out_path) = &args.output {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, &report_json)?;
        eprintln!("[OK] validate-ensemble -> {}", out_path.display());
    } else {
        println!("{report_json}");
    }

    if let Some(error)=bisect_analysis::research_ensemble::strict_failure(&report,args.strict) {
        anyhow::bail!("{error}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk `dir` for `*.jsonl` files, parse each as one chain (Vec<EnsembleStep>),
/// return chains in lexicographic filename order so the same dir produces a
/// deterministic chain ordering across runs.
fn load_chains_from_dir(dir: &Path) -> anyhow::Result<Vec<Vec<EnsembleStep>>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|r| r.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("jsonl"))
        .collect();
    paths.sort();

    let mut chains = Vec::with_capacity(paths.len());
    for path in &paths {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("[INPUT] cannot read chain {}: {}", path.display(), e))?;
        let steps = bisect_analysis::research_ensemble::parse_chain(&text)?;
        chains.push(steps);
    }
    Ok(chains)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build N chains of K steps each. Each chain step has a `dem_seats`
    /// metric drawn from a chain-specific deterministic sequence; the
    /// sequences are designed so chain means are nearly equal (R-hat ~ 1.0).
    fn write_well_mixed_chains(dir: &Path, n_chains: usize, n_per_chain: usize) {
        for c in 0..n_chains {
            let mut text = String::new();
            for s in 0..n_per_chain {
                // Same mean across chains (5.0), small chain-specific
                // shift that averages out — keeps R-hat tight.
                let value = 5.0
                    + ((s as f64) * 0.001)
                    + ((c as f64) * 0.0001) * if s % 2 == 0 { 1.0 } else { -1.0 };
                let mm_value = -0.01 + (s as f64) * 0.0001;
                let line = format!(
                    "{{\"step\": {s}, \"metrics\": {{\"dem_seats\": {value}, \"mean_median\": {mm_value}}}}}\n"
                );
                text.push_str(&line);
            }
            fs::write(dir.join(format!("chain-{c}.jsonl")), text).unwrap();
        }
    }

    /// Build N chains of K steps where each chain's mean is wildly different
    /// — forces R-hat well above 1.05 for the affected metric.
    fn write_unmixed_chains(dir: &Path, n_chains: usize, n_per_chain: usize) {
        for c in 0..n_chains {
            let mut text = String::new();
            // Per-chain offset of 5*c — chains never overlap in distribution.
            let offset = (c as f64) * 5.0;
            for s in 0..n_per_chain {
                let value = offset + (s as f64) * 0.001;
                let line =
                    format!("{{\"step\": {s}, \"metrics\": {{\"efficiency_gap\": {value}}}}}\n");
                text.push_str(&line);
            }
            fs::write(dir.join(format!("chain-{c}.jsonl")), text).unwrap();
        }
    }

    #[test]
    fn validate_ensemble_well_mixed_chains_all_pass() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 4, 100);
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: None,
            rhat_threshold: 1.05,
            ess_min: 50.0,
            enacted: None,
            strict: false,
        };
        let result = run_validate_ensemble(&args);
        assert!(
            result.is_ok(),
            "well-mixed ensemble must succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn validate_ensemble_unmixed_chains_flag_failure() {
        let tmp = TempDir::new().unwrap();
        write_unmixed_chains(tmp.path(), 4, 50);
        let out = tmp.path().join("report.json");
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: Some(out.clone()),
            rhat_threshold: 1.05,
            ess_min: 10.0,
            enacted: None,
            strict: false,
        };
        run_validate_ensemble(&args).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
        assert_eq!(
            report["schema_version"].as_str(),
            Some("validate-ensemble v1")
        );
        assert_eq!(report["n_chains"].as_u64(), Some(4));
        assert_eq!(report["n_per_chain"].as_u64(), Some(50));
        // The metric is so unmixed that R-hat will be far above threshold.
        let metrics = report["metrics"].as_array().unwrap();
        assert_eq!(metrics.len(), 1);
        let m = &metrics[0];
        assert_eq!(m["metric"].as_str(), Some("efficiency_gap"));
        assert!(
            m["rhat"]["above_threshold"].as_bool().unwrap_or(false),
            "wildly unmixed chains must trip R-hat"
        );
        assert!(!m["passes_thresholds"].as_bool().unwrap());
        assert!(!report["all_pass"].as_bool().unwrap());
    }

    #[test]
    fn validate_ensemble_strict_returns_error_on_failure() {
        let tmp = TempDir::new().unwrap();
        write_unmixed_chains(tmp.path(), 4, 50);
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: Some(tmp.path().join("ignored.json")),
            rhat_threshold: 1.05,
            ess_min: 10.0,
            enacted: None,
            strict: true,
        };
        let result = run_validate_ensemble(&args);
        assert!(
            result.is_err(),
            "--strict must propagate convergence failures as a non-zero exit"
        );
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("[INPUT]"));
        assert!(msg.contains("efficiency_gap"));
    }

    #[test]
    fn validate_ensemble_rejects_too_few_chains() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 3, 50); // S-03 wants >= 4
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: None,
            rhat_threshold: 1.05,
            ess_min: 10.0,
            enacted: None,
            strict: false,
        };
        let err = run_validate_ensemble(&args).unwrap_err().to_string();
        assert!(err.contains("[INPUT]"));
        assert!(
            err.contains(">=4 parallel chains") || err.contains(">=4"),
            "must mention 4-chain minimum: {err}"
        );
    }

    #[test]
    fn validate_ensemble_rejects_unequal_chain_lengths() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 4, 50);
        // Truncate one chain to 30 steps.
        let path = tmp.path().join("chain-0.jsonl");
        let text = fs::read_to_string(&path).unwrap();
        let trimmed: String = text.lines().take(30).collect::<Vec<_>>().join("\n") + "\n";
        fs::write(&path, trimmed).unwrap();

        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: None,
            rhat_threshold: 1.05,
            ess_min: 10.0,
            enacted: None,
            strict: false,
        };
        let err = run_validate_ensemble(&args).unwrap_err().to_string();
        assert!(err.contains("[INPUT]"));
        assert!(
            err.contains("differing lengths"),
            "error must mention chain-length mismatch: {err}"
        );
    }

    #[test]
    fn validate_ensemble_enacted_percentile_rank_records_position() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 4, 100);
        // Enacted plan with a dem_seats value WAY below the ensemble mean (5.0).
        let enacted_path = tmp.path().join("enacted.json");
        fs::write(&enacted_path, r#"{"dem_seats": 3.0, "mean_median": -0.5}"#).unwrap();

        let out = tmp.path().join("report.json");
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: Some(out.clone()),
            rhat_threshold: 1.5, // generous so the well-mixed case passes
            ess_min: 10.0,
            enacted: Some(enacted_path),
            strict: false,
        };
        run_validate_ensemble(&args).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
        let metrics = report["metrics"].as_array().unwrap();
        let dem_seats = metrics
            .iter()
            .find(|m| m["metric"].as_str() == Some("dem_seats"))
            .unwrap();
        let mean_median = metrics
            .iter()
            .find(|m| m["metric"].as_str() == Some("mean_median"))
            .unwrap();
        // dem_seats=3 is BELOW every ensemble sample (~5.0), so percentile rank = 0.0.
        let pr_seats = dem_seats["enacted_percentile_rank"].as_f64().unwrap();
        assert!(pr_seats < 0.01, "enacted dem_seats=3 should rank at the bottom of an ensemble centered on 5: pr={pr_seats}");
        // mean_median=-0.5 is BELOW every ensemble sample (~-0.01), so pr ~= 0.
        let pr_mm = mean_median["enacted_percentile_rank"].as_f64().unwrap();
        assert!(
            pr_mm < 0.01,
            "enacted mean_median=-0.5 should rank at the bottom: pr={pr_mm}"
        );
    }

    #[test]
    fn validate_ensemble_skips_extra_jsonl_files_via_extension_filter() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 4, 50);
        // Drop a notes.txt + a non-JSONL .json file; both must be ignored.
        fs::write(tmp.path().join("notes.txt"), "some notes\n").unwrap();
        fs::write(tmp.path().join("manifest.json"), "{}").unwrap();
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: None,
            rhat_threshold: 1.5,
            ess_min: 10.0,
            enacted: None,
            strict: false,
        };
        let result = run_validate_ensemble(&args);
        assert!(
            result.is_ok(),
            "non-jsonl files must be skipped: {:?}",
            result.err()
        );
    }

    #[test]
    fn percentile_rank_basic_cases() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile_rank(&samples, 3.0), 0.6); // 1,2,3 are <= 3 -> 3/5
        assert_eq!(percentile_rank(&samples, 0.0), 0.0);
        assert_eq!(percentile_rank(&samples, 5.0), 1.0);
        assert_eq!(percentile_rank(&samples, 10.0), 1.0);
        assert!(percentile_rank(&[], 5.0).is_nan());
    }

    // ── percentile_rank: additional edge cases ────────────────────────────────

    #[test]
    fn percentile_rank_single_sample_at_value_is_one() {
        // The only sample IS the value → 1/1 = 1.0.
        assert_eq!(percentile_rank(&[42.0], 42.0), 1.0);
    }

    #[test]
    fn percentile_rank_single_sample_below_value_is_one() {
        // Sample (1.0) is <= value (100.0) → 1/1 = 1.0.
        assert_eq!(percentile_rank(&[1.0], 100.0), 1.0);
    }

    #[test]
    fn percentile_rank_single_sample_above_value_is_zero() {
        // Sample (10.0) > value (5.0) → 0/1 = 0.0.
        assert_eq!(percentile_rank(&[10.0], 5.0), 0.0);
    }

    #[test]
    fn percentile_rank_all_below_is_zero() {
        let samples: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        // value = 0: no sample <= 0 → 0.0.
        assert_eq!(percentile_rank(&samples, 0.0), 0.0);
    }

    #[test]
    fn percentile_rank_all_at_or_below_is_one() {
        let samples: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        assert_eq!(percentile_rank(&samples, 100.0), 1.0);
    }

    #[test]
    fn percentile_rank_duplicate_values_handled_correctly() {
        // All samples equal. If value == sample → all <= value → rank = 1.0.
        let samples = vec![5.0, 5.0, 5.0, 5.0];
        assert_eq!(percentile_rank(&samples, 5.0), 1.0);
        // Value below all samples.
        assert_eq!(percentile_rank(&samples, 4.9), 0.0);
    }

    #[test]
    fn percentile_rank_fractional_result_correct() {
        // 10 samples; exactly 3 are <= 3.0.
        let samples: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        let pr = percentile_rank(&samples, 3.0);
        assert!((pr - 0.3).abs() < 1e-12, "expected 0.3, got {pr}");
    }

    // ── run_validate_ensemble: ensemble robustness ────────────────────────────

    #[test]
    fn validate_ensemble_exactly_four_chains_passes_min_requirement() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 4, 50);
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: None,
            rhat_threshold: 1.5,
            ess_min: 10.0,
            enacted: None,
            strict: false,
        };
        let result = run_validate_ensemble(&args);
        assert!(
            result.is_ok(),
            "exactly 4 chains must satisfy S-03 requirement: {:?}",
            result.err()
        );
    }

    #[test]
    fn validate_ensemble_six_chains_accepted() {
        let tmp = TempDir::new().unwrap();
        write_well_mixed_chains(tmp.path(), 6, 50);
        let args = ValidateEnsembleArgs {
            ensemble_dir: tmp.path().to_path_buf(),
            output: None,
            rhat_threshold: 1.5,
            ess_min: 10.0,
            enacted: None,
            strict: false,
        };
        assert!(
            run_validate_ensemble(&args).is_ok(),
            "6 chains must be accepted"
        );
    }

    #[test]
    fn validate_ensemble_non_existent_dir_is_error() {
        let args = ValidateEnsembleArgs {
            ensemble_dir: std::path::PathBuf::from("/nonexistent/path/to/ensemble"),
            output: None,
            rhat_threshold: 1.05,
            ess_min: 100.0,
            enacted: None,
            strict: false,
        };
        let err = run_validate_ensemble(&args).unwrap_err().to_string();
        assert!(
            err.contains("[INPUT]"),
            "missing dir must emit [INPUT] error: {err}"
        );
    }
}
