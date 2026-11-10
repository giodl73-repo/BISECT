//! Shared native MCMC summary-trace diagnostics. No ensemble generation or legal certification.
use std::collections::BTreeMap;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
pub(crate) fn validate(sources:&BTreeMap<String,String>,enacted_text:Option<&str>,rhat_threshold:f64,ess_min:f64,strict:bool)->Result<Value,String>{
    if !(4..=32).contains(&sources.len()){return Err("Supply 4–32 parallel chain JSONL files.".into());}
    let bytes=sources.iter().map(|(n,s)|n.len().saturating_add(s.len())).sum::<usize>().saturating_add(enacted_text.map(str::len).unwrap_or(0));
    if bytes>3*1024*1024{return Err("Research inputs must total at most 3 MiB.".into());}
    let mut chains=Vec::new();let mut metric_count=0;
    for (name,text) in sources{
        if name.len()>120||name==".jsonl"||!name.ends_with(".jsonl")||name.chars().any(|c|c.is_control()||matches!(c,'/'|'\\'|':')){return Err("Chains need plain JSONL basenames of at most 120 UTF-8 bytes without paths or controls.".into());}
        let chain=bisect_analysis::research_ensemble::parse_chain(text).map_err(|e|format!("{name}: {e}"))?;
        if chain.len()>5000{return Err("Browser diagnostics support at most 5,000 steps per chain.".into());}
        for step in &chain{if step.metrics.len()>64||step.metrics.keys().any(|n|n.is_empty()||n.len()>120){return Err("Supply at most 64 nonempty metric names of at most 120 bytes.".into());}metric_count=metric_count.max(step.metrics.len());}
        chains.push(chain);
    }
    let n=chains.iter().map(Vec::len).sum::<usize>();
    if n.saturating_mul(n).saturating_mul(metric_count)>200_000_000{return Err("Research diagnostics exceed the browser autocorrelation work limit.".into());}
    let enacted=enacted_text.map(bisect_analysis::research_ensemble::parse_enacted_metrics).transpose().map_err(|e|e.to_string())?;
    let report=bisect_analysis::research_ensemble::validate_ensemble(&chains,enacted.as_ref(),rhat_threshold,ess_min).map_err(|e|e.to_string())?;
    let report_json=serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?;
    let strict_error=bisect_analysis::research_ensemble::strict_failure(&report,strict);
    let hashes:BTreeMap<_,_>=sources.iter().map(|(n,s)|(n,format!("{:x}",Sha256::digest(s.as_bytes())))).collect();
    Ok(json!({"schema_version":"bisect-research-ensemble-output-v1","report":report,"report_json":report_json,
        "strict":strict,"exit_code":if strict_error.is_some(){1}else{0},"status":if strict_error.is_some(){"strict-failed"}else{"completed"},"strict_error":strict_error,
        "source_sha256":hashes,"enacted_sha256":enacted_text.map(|s|format!("{:x}",Sha256::digest(s.as_bytes()))),"rhat_threshold":rhat_threshold,"ess_min":ess_min,
        "scope":"Native researcher MCMC summary-trace R-hat, pooled ESS and optional enacted percentile ranks. Chains are concatenated in lexicographic filename order for ESS; this preserves the CLI method and does not account separately for chain boundaries. Native ESS below_threshold uses its fixed 100 cutoff; passes_thresholds uses the supplied ESS minimum. Equal-length chains require every step to record every metric. Burn-in removal and thinning are supplied by the practitioner, not performed here. Constant identical traces can pass these diagnostics without demonstrating exploration. Source hashes establish byte integrity, not ensemble authenticity, valid sampling, optimality or legal correctness. This operation does not generate MCMC samples or evaluate partition-space Hamming autocorrelation."}))
}
