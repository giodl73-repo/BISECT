use crate::args::IlpAuditArgs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IlpAuditRecord {
    report: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exact_search_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IlpAuditSummary {
    checked: usize,
    passed: usize,
    failed: usize,
    fallback_required: usize,
    outcomes: BTreeMap<String, usize>,
    proof_statuses: BTreeMap<String, usize>,
    exact_search_strategies: BTreeMap<String, usize>,
    records: Vec<IlpAuditRecord>,
}

pub fn run_ilp_audit(args: &IlpAuditArgs) -> Result<(), String> {
    let reports = collect_reports(args)?;
    if reports.is_empty() {
        return Err("no ILP solve reports provided; pass REPORT paths or --dir DIR".to_string());
    }

    let summary = summarize_reports(reports);
    if let Some(path) = &args.verify_summary {
        let base = args
            .dir
            .as_deref()
            .or_else(|| path.parent())
            .unwrap_or_else(|| Path::new(""));
        verify_summary_file(path, &summary, base)?;
    }
    emit_summary(args, &summary)?;

    if summary.failed > 0 {
        return Err(format!(
            "{} ILP solve report(s) failed verification",
            summary.failed
        ));
    }
    Ok(())
}

pub fn verify_ilp_audit_summary_for_dir(
    report_dir: &Path,
    summary_path: &Path,
) -> Result<(), String> {
    let mut reports = Vec::new();
    collect_json_reports(report_dir, &mut reports)?;
    reports.sort();
    reports.dedup();
    if reports.is_empty() {
        return Err(format!(
            "no ILP solve reports found under {}",
            report_dir.display()
        ));
    }

    let summary = summarize_reports(reports);
    verify_summary_file(summary_path, &summary, report_dir)?;

    if summary.failed > 0 {
        return Err(format!(
            "{} ILP solve report(s) failed verification",
            summary.failed
        ));
    }
    Ok(())
}

pub fn write_ilp_audit_summary_for_dir(report_dir: &Path, out: &Path) -> Result<(), String> {
    let mut reports = Vec::new();
    collect_json_reports(report_dir, &mut reports)?;
    reports.sort();
    reports.dedup();
    if reports.is_empty() {
        return Err(format!(
            "no ILP solve reports found under {}",
            report_dir.display()
        ));
    }

    let summary = summarize_reports(reports);
    let json = serde_json::to_string_pretty(&summary)
        .map_err(|e| format!("serialize ilp audit summary: {e}"))?;
    write_summary(out, &json)?;

    if summary.failed > 0 {
        return Err(format!(
            "{} ILP solve report(s) failed verification",
            summary.failed
        ));
    }
    Ok(())
}

fn summarize_reports(reports: Vec<PathBuf>) -> IlpAuditSummary {
    let mut records = Vec::with_capacity(reports.len());
    for report in reports {
        records.push(verify_report(&report));
    }

    let passed = records.iter().filter(|record| record.ok).count();
    let failed = records.len() - passed;
    let fallback_required = records
        .iter()
        .filter(|record| record.fallback_required == Some(true))
        .count();
    let outcomes = count_optional_strings(records.iter().map(|record| record.outcome.as_deref()));
    let proof_statuses =
        count_optional_strings(records.iter().map(|record| record.proof_status.as_deref()));
    let exact_search_strategies = count_optional_strings(
        records
            .iter()
            .map(|record| record.exact_search_strategy.as_deref()),
    );
    IlpAuditSummary {
        checked: records.len(),
        passed,
        failed,
        fallback_required,
        outcomes,
        proof_statuses,
        exact_search_strategies,
        records,
    }
}

fn emit_summary(args: &IlpAuditArgs, summary: &IlpAuditSummary) -> Result<(), String> {
    let json_summary = if args.json || args.out.is_some() {
        Some(
            serde_json::to_string_pretty(&summary)
                .map_err(|e| format!("serialize ilp audit summary: {e}"))?,
        )
    } else {
        None
    };

    if let Some(out) = &args.out {
        write_summary(out, json_summary.as_deref().unwrap_or_default())?;
    }

    if args.json {
        let json = json_summary.as_deref().unwrap_or_default();
        println!("{json}");
    } else {
        for record in &summary.records {
            if record.ok {
                println!(
                    "OK {} [{} / {}] fallback_required={} -> {} ({})",
                    record.report,
                    record.outcome.as_deref().unwrap_or("<unknown-outcome>"),
                    record.proof_status.as_deref().unwrap_or("<unknown-proof>"),
                    record.fallback_required.unwrap_or(false),
                    record.model_path.as_deref().unwrap_or("<unknown>"),
                    record.sha256.as_deref().unwrap_or("<no hash>")
                );
            } else {
                println!(
                    "FAIL {}: {}",
                    record.report,
                    record
                        .error
                        .as_deref()
                        .unwrap_or("unknown verification error")
                );
            }
        }
        println!(
            "ILP audit: checked={}, passed={}, failed={}, fallback_required={}",
            summary.checked, summary.passed, summary.failed, summary.fallback_required
        );
        print_counts("outcomes", &summary.outcomes);
        print_counts("proof_statuses", &summary.proof_statuses);
        print_counts("exact_search_strategies", &summary.exact_search_strategies);
    }
    Ok(())
}

fn verify_summary_file(
    path: &Path,
    expected: &IlpAuditSummary,
    path_base: &Path,
) -> Result<(), String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("read ilp audit summary {}: {e}", path.display()))?;
    let actual: IlpAuditSummary = serde_json::from_slice(&bytes)
        .map_err(|e| format!("parse ilp audit summary {}: {e}", path.display()))?;
    if normalize_summary_paths(&actual, path_base) != normalize_summary_paths(expected, path_base) {
        return Err(format!(
            "ilp audit summary mismatch: {} does not match current per-node reports",
            path.display()
        ));
    }
    Ok(())
}

fn normalize_summary_paths(summary: &IlpAuditSummary, path_base: &Path) -> IlpAuditSummary {
    let mut normalized = summary.clone();
    for record in &mut normalized.records {
        record.report = normalize_path_for_compare(&record.report, path_base);
        if let Some(model_path) = &record.model_path {
            record.model_path = Some(normalize_path_for_compare(model_path, path_base));
        }
    }
    normalized
}

fn normalize_path_for_compare(path: &str, path_base: &Path) -> String {
    let path = PathBuf::from(path);
    let comparable = path.strip_prefix(path_base).unwrap_or(&path);
    comparable
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn write_summary(path: &Path, json: &str) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create ilp audit summary dir {}: {e}", parent.display()))?;
    }
    let tmp_path = path.with_extension("tmp.json");
    std::fs::write(&tmp_path, json)
        .map_err(|e| format!("write ilp audit summary tmp {}: {e}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| format!("publish ilp audit summary {}: {e}", path.display()))
}

fn count_optional_strings<'a>(
    values: impl Iterator<Item = Option<&'a str>>,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values.flatten() {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

fn print_counts(label: &str, counts: &BTreeMap<String, usize>) {
    if counts.is_empty() {
        return;
    }
    let rendered = counts
        .iter()
        .map(|(key, count)| format!("{key}={count}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("{label}: {rendered}");
}

fn collect_reports(args: &IlpAuditArgs) -> Result<Vec<PathBuf>, String> {
    let mut reports = args.reports.clone();
    if let Some(dir) = &args.dir {
        collect_json_reports(dir, &mut reports)?;
    }
    reports.sort();
    reports.dedup();
    Ok(reports)
}

fn collect_json_reports(dir: &Path, reports: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("read ilp audit directory {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read ilp audit directory entry: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("read file type {}: {e}", path.display()))?;
        if file_type.is_dir() {
            collect_json_reports(&path, reports)?;
        } else if is_solve_report_json_path(&path) {
            reports.push(path);
        }
    }
    Ok(())
}

fn is_solve_report_json_path(path: &Path) -> bool {
    path.is_file()
        && path.extension().and_then(|ext| ext.to_str()) == Some("json")
        && path.file_name().and_then(|name| name.to_str()) != Some("audit-summary.json")
}

fn verify_report(report: &Path) -> IlpAuditRecord {
    let solve_report = match read_solve_report(report) {
        Ok(report) => report,
        Err(err) => {
            return IlpAuditRecord {
                report: report.display().to_string(),
                ok: false,
                outcome: None,
                proof_status: None,
                fallback_required: None,
                exact_search_strategy: None,
                model_path: None,
                format: None,
                sha256: None,
                error: Some(err),
            };
        }
    };

    match bisect_ilp::verify_model_artifact_for_report(report) {
        Ok(verified) => IlpAuditRecord {
            report: report.display().to_string(),
            ok: true,
            outcome: Some(solve_report.audit_summary.outcome),
            proof_status: Some(solve_report.audit_summary.proof_status),
            fallback_required: Some(solve_report.audit_summary.fallback_required),
            exact_search_strategy: solve_report.audit_summary.exact_search_strategy,
            model_path: Some(verified.path.display().to_string()),
            format: Some(verified.format),
            sha256: Some(verified.sha256),
            error: None,
        },
        Err(err) => IlpAuditRecord {
            report: report.display().to_string(),
            ok: false,
            outcome: Some(solve_report.audit_summary.outcome),
            proof_status: Some(solve_report.audit_summary.proof_status),
            fallback_required: Some(solve_report.audit_summary.fallback_required),
            exact_search_strategy: solve_report.audit_summary.exact_search_strategy,
            model_path: None,
            format: None,
            sha256: None,
            error: Some(err.to_string()),
        },
    }
}

fn read_solve_report(report: &Path) -> Result<bisect_ilp::IlpSolveReport, String> {
    let bytes = std::fs::read(report)
        .map_err(|e| format!("read solve report {}: {e}", report.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("parse solve report {}: {e}", report.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bisect_ilp::{IlpModelArtifact, IlpSolveReport};
    use sha2::{Digest, Sha256};

    fn write_report(dir: &Path, name: &str, lp_bytes: &[u8], sha256: String) -> PathBuf {
        let formulation = bisect_ilp::build_formulation(&[vec![1], vec![0]], &[1, 1], 2, 0.05);
        let result = bisect_ilp::solve(
            &formulation,
            &[vec![1], vec![0]],
            &[1, 1],
            2,
            0.05,
            bisect_ilp::IlpSolver::FormulationOnly,
            0.01,
        );
        let lp_path = dir.join(format!("{name}.lp"));
        std::fs::write(&lp_path, lp_bytes).unwrap();
        let report = IlpSolveReport::with_model_artifact(
            formulation,
            result,
            IlpModelArtifact {
                format: "cplex-lp".to_string(),
                path: format!("{name}.lp"),
                sha256,
            },
        );
        let report_path = dir.join(format!("{name}.json"));
        std::fs::write(&report_path, report.to_json_string().unwrap()).unwrap();
        report_path
    }

    fn sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn ilp_audit_counts_optional_strings() {
        let counts = count_optional_strings(
            [
                Some("exact-plan"),
                None,
                Some("formulation-only"),
                Some("exact-plan"),
            ]
            .into_iter(),
        );
        assert_eq!(counts.get("exact-plan"), Some(&2));
        assert_eq!(counts.get("formulation-only"), Some(&1));
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn ilp_audit_accepts_matching_report() {
        let tmp = tempfile::TempDir::new().unwrap();
        let report_path = write_report(tmp.path(), "node_root", b"lp text", sha256(b"lp text"));
        let record = verify_report(&report_path);
        assert!(record.ok);
        assert_eq!(record.outcome.as_deref(), Some("formulation-only"));
        assert_eq!(record.proof_status.as_deref(), Some("not-solved"));
        assert_eq!(record.fallback_required, Some(true));
        let args = IlpAuditArgs {
            reports: Vec::new(),
            dir: Some(tmp.path().to_path_buf()),
            json: true,
            out: None,
            verify_summary: None,
        };
        run_ilp_audit(&args).expect("matching ILP report should pass");
    }

    #[test]
    fn ilp_audit_writes_summary_artifact() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_report(tmp.path(), "node_root", b"lp text", sha256(b"lp text"));
        let out = tmp.path().join("audit").join("audit-summary.json");
        let args = IlpAuditArgs {
            reports: Vec::new(),
            dir: Some(tmp.path().to_path_buf()),
            json: false,
            out: Some(out.clone()),
            verify_summary: None,
        };
        run_ilp_audit(&args).expect("matching ILP report should pass");
        let json = std::fs::read_to_string(&out).expect("read audit summary");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse audit summary");
        assert_eq!(value["checked"], 1);
        assert_eq!(value["passed"], 1);
        assert_eq!(value["failed"], 0);
        assert_eq!(value["fallback_required"], 1);
        assert_eq!(value["outcomes"]["formulation-only"], 1);
        assert_eq!(value["proof_statuses"]["not-solved"], 1);

        let verify_args = IlpAuditArgs {
            reports: Vec::new(),
            dir: Some(tmp.path().to_path_buf()),
            json: false,
            out: None,
            verify_summary: Some(out),
        };
        run_ilp_audit(&verify_args).expect("fresh audit summary should verify");
    }

    #[test]
    fn ilp_audit_rejects_stale_summary_artifact() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_report(tmp.path(), "node_root", b"lp text", sha256(b"lp text"));
        let stale = tmp.path().join("audit-summary.json");
        std::fs::write(
            &stale,
            r#"{"checked":0,"passed":0,"failed":0,"fallback_required":0,"outcomes":{},"proof_statuses":{},"exact_search_strategies":{},"records":[]}"#,
        )
        .unwrap();
        let args = IlpAuditArgs {
            reports: Vec::new(),
            dir: Some(tmp.path().to_path_buf()),
            json: false,
            out: None,
            verify_summary: Some(stale),
        };
        let err = run_ilp_audit(&args).expect_err("stale audit summary should fail");
        assert!(err.contains("summary mismatch"));
    }

    #[test]
    fn ilp_audit_rejects_hash_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let report_path = write_report(tmp.path(), "node_root", b"lp text", sha256(b"different"));
        let record = verify_report(&report_path);
        assert!(!record.ok);
        assert_eq!(record.outcome.as_deref(), Some("formulation-only"));
        assert_eq!(record.proof_status.as_deref(), Some("not-solved"));
        assert_eq!(record.fallback_required, Some(true));
        assert!(record.error.as_deref().unwrap().contains("hash mismatch"));
        let args = IlpAuditArgs {
            reports: Vec::new(),
            dir: Some(tmp.path().to_path_buf()),
            json: false,
            out: None,
            verify_summary: None,
        };
        let err = run_ilp_audit(&args).expect_err("hash mismatch should fail audit");
        assert!(err.contains("failed verification"));
    }
}
