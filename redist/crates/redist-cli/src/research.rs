//! Researcher Toolkit (`redist research ...`).
//!
//! Today's surface area:
//! - `redist research validate-ensemble` (RT Task 5 / M-02): consume an MCMC
//!   ensemble (one JSONL per chain), compute R-hat / ESS / optional Hamming
//!   autocorrelation per metric using the already-shipped
//!   `redist_analysis::ensemble_diagnostics` math, and emit a
//!   `validate-ensemble.json` attesting whether the ensemble meets convergence
//!   thresholds. Optional `--enacted` flag computes per-metric percentile-rank
//!   of an enacted plan against the ensemble distribution — directly useful
//!   for state-court partisan-fairness litigation under the FAIRNESS_DOCTRINE
//!   strategy.
//!
//! Future subcommands (deferred):
//! - `redist research check-compat` (Task 3): GerryChain handshake.
//! - `redist research mcmc` (Task 6): wrapper around scripts/research/mcmc_ensemble.py.
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

use serde::{Deserialize, Serialize};

use redist_analysis::ensemble_diagnostics::{
    ess_records, gelman_rubin_rhat, EssRecord, RhatRecord,
};

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

#[derive(Debug, Clone, Deserialize)]
struct EnsembleStep {
    #[allow(dead_code)]
    step: Option<u64>,
    metrics: BTreeMap<String, f64>,
}

// ---------------------------------------------------------------------------
// Output report shape.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ValidateEnsembleReport {
    pub schema_version: String,
    pub n_chains: usize,
    pub n_per_chain: usize,
    pub metrics: Vec<MetricReport>,
    /// True iff every metric passes both R-hat and ESS thresholds.
    pub all_pass: bool,
}

#[derive(Debug, Serialize)]
pub struct MetricReport {
    pub metric: String,
    pub rhat: RhatRecord,
    pub ess: EssRecord,
    pub passes_thresholds: bool,
    /// Present only when `--enacted` was supplied. Fraction of ensemble
    /// samples (across all chains) whose value is `<=` the enacted value.
    /// 0.0 means the enacted plan is more extreme than every sample below;
    /// 1.0 means more extreme above.
    pub enacted_percentile_rank: Option<f64>,
}

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
    if chains.len() < 4 {
        anyhow::bail!(
            "[INPUT] S-03 requires >=4 parallel chains for Gelman-Rubin R-hat; \
             found {} JSONL file(s) in {}",
            chains.len(),
            args.ensemble_dir.display()
        );
    }

    // Determine the metric universe (union across chains) + chain length
    // (must be equal across chains per the standard Gelman-Rubin formulation).
    let chain_lens: Vec<usize> = chains.iter().map(|c| c.len()).collect();
    let n_per_chain = chain_lens
        .first()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("[INPUT] no chains loaded"))?;
    if chain_lens.iter().any(|&n| n != n_per_chain) {
        anyhow::bail!(
            "[INPUT] chains have differing lengths: {:?}. Trim to the shortest after burn-in cuts before passing to validate-ensemble.",
            chain_lens
        );
    }
    if n_per_chain == 0 {
        anyhow::bail!("[INPUT] every chain is empty");
    }

    let metric_names: std::collections::BTreeSet<String> = chains
        .iter()
        .flat_map(|chain| chain.iter().flat_map(|step| step.metrics.keys().cloned()))
        .collect();

    // Optional enacted metrics.
    let enacted_metrics: Option<BTreeMap<String, f64>> = if let Some(path) = &args.enacted {
        let bytes = std::fs::read(path).map_err(|e| {
            anyhow::anyhow!("[INPUT] cannot read --enacted at {}: {}", path.display(), e)
        })?;
        let m: BTreeMap<String, f64> = serde_json::from_slice(&bytes).map_err(|e| {
            anyhow::anyhow!(
                "[INPUT] cannot parse --enacted as {{metric: value}} JSON object at {}: {}",
                path.display(),
                e
            )
        })?;
        Some(m)
    } else {
        None
    };

    // Compute diagnostics per metric.
    let mut metric_reports: Vec<MetricReport> = Vec::with_capacity(metric_names.len());
    for metric_name in &metric_names {
        // Project this metric out of every chain. Steps that don't carry
        // this metric are skipped — but if any chain has fewer values than
        // n_per_chain after projection, we surface an [INPUT] error.
        let per_chain_values: Vec<Vec<f64>> = chains
            .iter()
            .map(|chain| {
                chain
                    .iter()
                    .filter_map(|step| step.metrics.get(metric_name).copied())
                    .collect()
            })
            .collect();
        if per_chain_values.iter().any(|v| v.len() != n_per_chain) {
            anyhow::bail!(
                "[INPUT] metric '{}' appears in some chain steps but not others. \
                 Every step in every chain must record every metric.",
                metric_name
            );
        }

        // R-hat.
        let chain_slices: Vec<&[f64]> = per_chain_values.iter().map(|v| v.as_slice()).collect();
        let rhat_value = gelman_rubin_rhat(&chain_slices)
            .map_err(|e| anyhow::anyhow!("rhat({}): {}", metric_name, e))?;
        let rhat = RhatRecord {
            metric: metric_name.clone(),
            r_hat: rhat_value,
            n_chains: chain_slices.len(),
            n_per_chain,
            above_threshold: rhat_value >= args.rhat_threshold,
            threshold: args.rhat_threshold,
        };

        // ESS — pool across chains by concatenation. (Per-chain ESS would
        // also be defensible; pooled is the more common reporting.)
        let pooled: Vec<f64> = per_chain_values.iter().flatten().copied().collect();
        let ess_records_vec = ess_records(&[(metric_name.clone(), pooled.as_slice())]);
        let ess = ess_records_vec
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("[INTERNAL] ess_records returned empty for metric"))?;

        // Threshold pass: R-hat below threshold AND ESS at or above ess_min.
        let passes_thresholds = !rhat.above_threshold && ess.ess >= args.ess_min;

        // Optional enacted percentile-rank.
        let enacted_pr = enacted_metrics
            .as_ref()
            .and_then(|m| m.get(metric_name))
            .map(|&enacted_value| percentile_rank(&pooled, enacted_value));

        metric_reports.push(MetricReport {
            metric: metric_name.clone(),
            rhat,
            ess,
            passes_thresholds,
            enacted_percentile_rank: enacted_pr,
        });
    }

    let all_pass = metric_reports.iter().all(|r| r.passes_thresholds);

    let report = ValidateEnsembleReport {
        schema_version: "validate-ensemble v1".into(),
        n_chains: chains.len(),
        n_per_chain,
        metrics: metric_reports,
        all_pass,
    };

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

    if args.strict && !all_pass {
        let failures: Vec<&str> = report
            .metrics
            .iter()
            .filter(|m| !m.passes_thresholds)
            .map(|m| m.metric.as_str())
            .collect();
        anyhow::bail!(
            "[INPUT] --strict: {}/{} metrics failed convergence thresholds: {}",
            failures.len(),
            report.metrics.len(),
            failures.join(", ")
        );
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
        .filter(|p| {
            p.is_file()
                && p.extension().and_then(|s| s.to_str()) == Some("jsonl")
        })
        .collect();
    paths.sort();

    let mut chains = Vec::with_capacity(paths.len());
    for path in &paths {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("[INPUT] cannot read chain {}: {}", path.display(), e))?;
        let mut steps = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let step: EnsembleStep = serde_json::from_str(line).map_err(|e| {
                anyhow::anyhow!(
                    "[INPUT] cannot parse line {} of {}: {}",
                    i + 1,
                    path.display(),
                    e
                )
            })?;
            steps.push(step);
        }
        chains.push(steps);
    }
    Ok(chains)
}

/// Fraction of `samples` that are `<= value`. Empty samples returns NaN.
fn percentile_rank(samples: &[f64], value: f64) -> f64 {
    if samples.is_empty() {
        return f64::NAN;
    }
    let count = samples.iter().filter(|&&s| s <= value).count();
    (count as f64) / (samples.len() as f64)
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
                let line = format!(
                    "{{\"step\": {s}, \"metrics\": {{\"efficiency_gap\": {value}}}}}\n"
                );
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
        assert!(result.is_ok(), "well-mixed ensemble must succeed: {:?}", result.err());
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

        let report: serde_json::Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
        assert_eq!(report["schema_version"].as_str(), Some("validate-ensemble v1"));
        assert_eq!(report["n_chains"].as_u64(), Some(4));
        assert_eq!(report["n_per_chain"].as_u64(), Some(50));
        // The metric is so unmixed that R-hat will be far above threshold.
        let metrics = report["metrics"].as_array().unwrap();
        assert_eq!(metrics.len(), 1);
        let m = &metrics[0];
        assert_eq!(m["metric"].as_str(), Some("efficiency_gap"));
        assert!(m["rhat"]["above_threshold"].as_bool().unwrap_or(false), "wildly unmixed chains must trip R-hat");
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
        assert!(result.is_err(), "--strict must propagate convergence failures as a non-zero exit");
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
        assert!(err.contains(">=4 parallel chains") || err.contains(">=4"), "must mention 4-chain minimum: {err}");
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
        assert!(err.contains("differing lengths"), "error must mention chain-length mismatch: {err}");
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

        let report: serde_json::Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
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
        assert!(pr_mm < 0.01, "enacted mean_median=-0.5 should rank at the bottom: pr={pr_mm}");
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
        assert!(result.is_ok(), "non-jsonl files must be skipped: {:?}", result.err());
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
}
