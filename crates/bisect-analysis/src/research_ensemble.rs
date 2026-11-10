//! Native researcher ensemble validation in memory. ESS concatenates chains in source order.
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::ensemble_diagnostics::{ess_records, gelman_rubin_rhat, EssRecord, RhatRecord};
#[derive(Debug, Clone, Deserialize)]
pub struct EnsembleStep {
    #[allow(dead_code)]
    step: Option<u64>,
    #[serde(deserialize_with = "unique_metrics")]
    pub metrics: BTreeMap<String, f64>,
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

/// CLI strict mode retains the report, then fails if any metric misses thresholds.
pub fn strict_failure(report: &ValidateEnsembleReport, strict: bool) -> Option<String> {
    if !strict || report.all_pass { return None; }
    let failures: Vec<&str> = report.metrics.iter().filter(|m| !m.passes_thresholds)
        .map(|m| m.metric.as_str()).collect();
    Some(format!("[INPUT] --strict: {}/{} metrics failed convergence thresholds: {}",
        failures.len(), report.metrics.len(), failures.join(", ")))
}

pub fn validate_ensemble(
    chains: &[Vec<EnsembleStep>], enacted_metrics: Option<&BTreeMap<String,f64>>,
    rhat_threshold:f64, ess_min:f64,
) -> anyhow::Result<ValidateEnsembleReport> {
    anyhow::ensure!(rhat_threshold.is_finite() && rhat_threshold>0.0 && ess_min.is_finite() && ess_min>=0.0, "[INPUT] diagnostic thresholds must be finite and R-hat positive / ESS nonnegative");
    if let Some(metrics)=enacted_metrics { anyhow::ensure!(metrics.values().all(|v|v.is_finite()), "[INPUT] enacted metrics must be finite"); }
    if chains.len() < 4 {
        anyhow::bail!(
            "[INPUT] S-03 requires >=4 parallel chains for Gelman-Rubin R-hat; \
             found {} JSONL file(s) in {}",
            chains.len(),
            "supplied inputs"
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
            above_threshold: rhat_value >= rhat_threshold,
            threshold: rhat_threshold,
        };

        // ESS — pool across chains by concatenation. (Per-chain ESS would
        // also be defensible; pooled is the more common reporting.)
        let pooled: Vec<f64> = per_chain_values.iter().flatten().copied().collect();
        let ess_records_vec = ess_records(&[(metric_name.clone(), pooled.as_slice())])
            .map_err(|e| anyhow::anyhow!("ess({}): {}", metric_name, e))?;
        let ess = ess_records_vec
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("[INTERNAL] ess_records returned empty for metric"))?;

        // Threshold pass: R-hat below threshold AND ESS at or above ess_min.
        let passes_thresholds = !rhat.above_threshold && ess.ess >= ess_min;

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

    anyhow::ensure!(!metric_reports.is_empty(), "[INPUT] ensemble metrics must be nonempty");
    let all_pass = metric_reports.iter().all(|r| r.passes_thresholds);

    let report = ValidateEnsembleReport {
        schema_version: "validate-ensemble v1".into(),
        n_chains: chains.len(),
        n_per_chain,
        metrics: metric_reports,
        all_pass,
    };

    Ok(report)
}

/// Parse one native MCMC chain, ignoring blank lines like the CLI reader.
pub fn parse_chain(text:&str) -> anyhow::Result<Vec<EnsembleStep>> {
    text.lines().enumerate().filter(|(_,line)|!line.trim().is_empty()).map(|(i,line)|
        serde_json::from_str(line.trim()).map_err(|e|anyhow::anyhow!("[INPUT] cannot parse chain line {}: {}",i+1,e))).collect()
}

fn unique_metrics<'de,D:serde::Deserializer<'de>>(d:D)->Result<BTreeMap<String,f64>,D::Error>{
    struct Unique;
    impl<'de> serde::de::Visitor<'de> for Unique{
        type Value=BTreeMap<String,f64>;
        fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("unique metric names with finite numeric values")}
        fn visit_map<M:serde::de::MapAccess<'de>>(self,mut m:M)->Result<Self::Value,M::Error>{
            let mut out=BTreeMap::new();while let Some((name,value))=m.next_entry::<String,f64>()?{
                if !value.is_finite(){return Err(serde::de::Error::custom("Metric values must be finite"));}
                if out.insert(name.clone(),value).is_some(){return Err(serde::de::Error::custom(format!("Duplicate metric {name}")));}
            }Ok(out)
        }
    }d.deserialize_map(Unique)
}

pub fn parse_enacted_metrics(text:&str)->anyhow::Result<BTreeMap<String,f64>>{
    #[derive(Deserialize)]
    struct Metrics(#[serde(deserialize_with="unique_metrics")] BTreeMap<String,f64>);
    Ok(serde_json::from_str::<Metrics>(text)?.0)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn rejects_duplicate_metrics_in_chain_and_enacted(){
        assert!(parse_chain(r#"{"metrics":{"x":1,"x":2}}"#).is_err());
        assert!(parse_enacted_metrics(r#"{"x":1,"x":2}"#).is_err());
    }
    #[test]
    fn empty_metric_universe_cannot_pass(){
        let chains=vec![parse_chain("{\"metrics\":{}}\n{\"metrics\":{}}").unwrap();4];
        assert!(validate_ensemble(&chains,None,1.05,100.0).is_err());
    }
    #[test]
    fn identical_constant_chains_preserve_native_diagnostics(){
        let chains=vec![parse_chain(&"{\"metrics\":{\"x\":2}}\n".repeat(4)).unwrap();4];
        let report=validate_ensemble(&chains,Some(&BTreeMap::from([("x".into(),2.0)])),1.05,16.0).unwrap();
        assert_eq!(report.metrics[0].rhat.r_hat,1.0);assert_eq!(report.metrics[0].ess.ess,16.0);
        assert_eq!(report.metrics[0].enacted_percentile_rank,Some(1.0));assert!(report.all_pass);
        assert!(validate_ensemble(&chains,None,f64::NAN,0.0).is_err());
    }
}

/// Fraction of `samples` that are `<= value`. Empty samples returns NaN.
pub fn percentile_rank(samples: &[f64], value: f64) -> f64 {
    if samples.is_empty() {
        return f64::NAN;
    }
    let count = samples.iter().filter(|&&s| s <= value).count();
    (count as f64) / (samples.len() as f64)
}

