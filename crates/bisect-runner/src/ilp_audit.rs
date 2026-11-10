use crate::args::IlpAuditArgs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IlpAuditRecord {
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
pub struct IlpAuditSummary {
    pub checked: usize,
    pub passed: usize,
    pub failed: usize,
    fallback_required: usize,
    outcomes: BTreeMap<String, usize>,
    proof_statuses: BTreeMap<String, usize>,
    exact_search_strategies: BTreeMap<String, usize>,
    records: Vec<IlpAuditRecord>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub report_selection: Option<Vec<String>>,
}

pub fn run_ilp_audit(args: &IlpAuditArgs) -> Result<(), String> {
    let reports = collect_reports(args)?;
    if reports.is_empty() {
        return Err("no ILP solve reports provided; pass REPORT paths or --dir DIR".to_string());
    }

    let mut summary = summarize_reports(reports);
    if let Some(path) = &args.verify_summary {
        let base = args
            .dir
            .as_deref()
            .or_else(|| path.parent())
            .unwrap_or_else(|| Path::new(""));
        let bytes=std::fs::read(path).map_err(|e|format!("read ilp audit summary {}: {e}",path.display()))?;
        verify_and_retain_supplied_summary(&bytes,&mut summary,base)?;
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
    verify_ilp_audit_summary_for_dir_receipt(report_dir,summary_path).map(|_|())
}

pub fn verify_ilp_audit_summary_for_dir_receipt(report_dir:&Path,summary_path:&Path)->Result<IlpAuditSummary,String>{
    let mut reports = Vec::new();
    let bytes=std::fs::read(summary_path).map_err(|e|format!("read ILP summary: {e}"))?;
    let selection=summary_report_selection(&bytes)?;
    if let Some(paths)=&selection {reports.extend(paths.iter().map(|p|report_dir.join(p)));}else{collect_json_reports(report_dir,&mut reports)?;}
    reports.sort();
    reports.dedup();
    if reports.is_empty() {
        return Err(format!(
            "no ILP solve reports found under {}",
            report_dir.display()
        ));
    }

    let mut summary = summarize_reports(reports);summary.report_selection=selection;
    verify_summary_file(summary_path, &summary, report_dir)?;

    if summary.failed > 0 {
        return Err(format!(
            "{} ILP solve report(s) failed verification",
            summary.failed
        ));
    }
    Ok(summary)
}

#[derive(Debug,Default,Deserialize)]
#[serde(default, bound(deserialize = "C: Deserialize<'de> + Default"))]
pub struct ManifestIlpBindingsWithCounts<C> {
    pub ilp_audit_summary_path:Option<String>,
    pub ilp_audit_summary_sha256:Option<String>,
    pub ilp_solve_report_dir:Option<String>,
    pub ilp_solve_report_count:Option<C>,
}
/// Native execution-facing binding retains its pointer-sized domain.
pub type ManifestIlpBindings = ManifestIlpBindingsWithCounts<usize>;
/// Read-only metadata binding for native 64-bit count claims on wasm32.
pub type ReadOnlyManifestIlpBindings = ManifestIlpBindingsWithCounts<u64>;
#[derive(Debug,Serialize)]
pub struct ManifestIlpSummaryVerification {
    pub summary_path:String,
    pub report_directory:String,
    pub summary:IlpAuditSummary,
}
/// Native manifest digest and directory-default checks; adapters provide source access.
pub fn verify_manifest_ilp_summary<C:Copy + std::fmt::Display + TryInto<u64>>(
    bindings:&ManifestIlpBindingsWithCounts<C>,
    hash:impl Fn(&str)->Result<String,String>,
    verify:impl Fn(&str,&str)->Result<IlpAuditSummary,String>,
)->Result<Option<ManifestIlpSummaryVerification>,String>{
    let Some(summary)=bindings.ilp_audit_summary_path.as_deref()else{return Ok(None)};
    if let Some(expected)=bindings.ilp_audit_summary_sha256.as_deref().filter(|s|!s.is_empty()){
        let actual=hash(summary).map_err(|e|format!("hash ILP audit summary: {e}"))?;
        if actual!=expected{return Err(format!("ILP audit summary SHA-256 mismatch for {summary}: manifest={expected}, current={actual}"));}
    }
    let directory=bindings.ilp_solve_report_dir.clone().or_else(||Path::new(summary).parent().map(|p|p.to_string_lossy().into_owned())).ok_or("cannot resolve ILP solve report directory")?;
    let receipt=verify(&directory,summary).map_err(|e|format!("ILP audit summary verification failed: {e}"))?;
    if let Some(count)=bindings.ilp_solve_report_count {
        let matches=match (count.try_into(),u64::try_from(receipt.checked)) {
            (Ok(expected),Ok(actual))=>expected==actual,
            _=>false,
        };
        if !matches{return Err(format!("ILP report count mismatch: manifest={count}, audited={}",receipt.checked));}
    }
    Ok(Some(ManifestIlpSummaryVerification{summary_path:summary.into(),report_directory:directory,summary:receipt}))
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

/// Portable explicit scope for summaries generated from the current invocation.
/// Historical summaries without this field retain whole-directory selection.
pub fn summary_report_selection(bytes:&[u8])->Result<Option<Vec<String>>,String>{
    let summary:IlpAuditSummary=serde_json::from_slice(bytes).map_err(|e|format!("parse ilp audit summary: {e}"))?;
    if let Some(paths)=&summary.report_selection {
        let mut sorted=paths.clone();sorted.sort();sorted.dedup();
        if paths.is_empty()||sorted!=*paths||paths.len()!=summary.checked||paths.iter().any(|p|p.chars().any(char::is_control)||p.contains('\\')||p.contains(':')||p.split('/').any(|c|c.is_empty()||c=="."||c=="..")||!is_solve_report_name(Path::new(p))){return Err("Invalid ILP summary report selection".into());}
    }
    Ok(summary.report_selection)
}

pub fn write_ilp_audit_summary_for_reports(report_dir:&Path,mut reports:Vec<PathBuf>,out:&Path)->Result<(),String>{
    reports.sort();reports.dedup();if reports.is_empty(){return Err("No current-invocation ILP reports".into());}
    let mut selection=reports.iter().map(|p|p.strip_prefix(report_dir).map(|p|p.components().map(|c|c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")).map_err(|_|"ILP report outside current report directory".to_string())).collect::<Result<Vec<_>,_>>()?;selection.sort();
    let mut summary=summarize_reports(reports);summary.report_selection=Some(selection);
    let bytes=serde_json::to_vec_pretty(&summary).map_err(|e|e.to_string())?;summary_report_selection(&bytes)?;
    write_summary(out,std::str::from_utf8(&bytes).map_err(|e|e.to_string())?)?;
    if summary.failed>0{return Err(format!("{} ILP solve report(s) failed verification",summary.failed));}Ok(())
}

fn summarize_reports(reports: Vec<PathBuf>) -> IlpAuditSummary {
    let mut records = Vec::with_capacity(reports.len());
    for report in reports {
        records.push(verify_report(&report));
    }

    summarize_records(records)
}

fn summarize_records(records: Vec<IlpAuditRecord>) -> IlpAuditSummary {
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
        report_selection:None,
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
    verify_supplied_summary(&bytes,expected,path_base)
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
    path.is_file() && is_solve_report_name(path)
}
/// Native report-name filter over known file selections; no filesystem query.
pub fn is_solve_report_name(path:&Path)->bool{
    path.extension().and_then(|ext| ext.to_str()) == Some("json")
        && path.file_name().and_then(|name| name.to_str()) != Some("audit-summary.json")
}

fn verify_report(report: &Path) -> IlpAuditRecord {
    record_from_sources(report, read_solve_report(report), |_| {
        bisect_ilp::verify_model_artifact_for_report(report).map_err(|e|e.to_string())
    })
}

fn record_from_sources(report: &Path, parsed: Result<bisect_ilp::IlpSolveReport,String>, verify: impl FnOnce(&bisect_ilp::IlpSolveReport)->Result<bisect_ilp::VerifiedModelArtifact,String>) -> IlpAuditRecord {
    let solve_report = match parsed {
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

    match verify(&solve_report) {
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

/// Audit explicitly selected report/model bytes with the native record and summary kernels.
pub fn audit_supplied_reports(reports: &BTreeMap<String,Vec<u8>>, models: &BTreeMap<String,Vec<u8>>) -> IlpAuditSummary {
    use sha2::{Digest,Sha256};
    let records=reports.iter().map(|(name,bytes)|{
        let path=Path::new(name);
        let parsed=serde_json::from_slice(bytes).map_err(|e|format!("parse solve report {name}: {e}"));
        let mut record=record_from_sources(path,parsed,|report|{
            if let Some(artifact)=&report.model_artifact {
                // Browser selections use portable relative names; never interpret drive/UNC paths.
                if artifact.path.contains('\\')||artifact.path.contains(':') {return Err("Selected model reference must be a portable relative path.".into());}
            }
            bisect_ilp::verify_model_artifact_with_digest(path,report,|resolved|{
                let key=resolved.to_string_lossy().replace('\\',"/");
                models.get(&key).map(|bytes|format!("{:x}",Sha256::digest(bytes))).ok_or_else(||std::io::Error::new(std::io::ErrorKind::NotFound,"model artifact not selected"))
            }).map_err(|e|e.to_string())
        });
        // A portable selection has the same receipt paths on Windows and wasm32.
        if let Some(path)=&mut record.model_path{*path=path.replace('\\',"/");}
        if let Some(error)=&mut record.error{*error=error.replace('\\',"/");}
        record
    }).collect();
    summarize_records(records)
}

/// Compare a supplied summary using the same native normalization and equality rule.
/// Retain a declared scope only after it matches all independently audited records.
/// On failure, computed summary state remains unchanged.
pub fn verify_and_retain_supplied_summary(bytes:&[u8],expected:&mut IlpAuditSummary,path_base:&Path)->Result<(),String>{
    verify_supplied_summary(bytes,expected,path_base)?;
    expected.report_selection=summary_report_selection(bytes)?;Ok(())
}

pub fn verify_supplied_summary(bytes:&[u8],expected:&IlpAuditSummary,path_base:&Path)->Result<(),String>{
    let actual:IlpAuditSummary=serde_json::from_slice(bytes).map_err(|e|format!("parse ilp audit summary: {e}"))?;
    let selection=summary_report_selection(bytes)?;
    let mut expected=normalize_summary_paths(expected,path_base);
    if let Some(paths)=&selection {let mut reports=expected.records.iter().map(|r|r.report.clone()).collect::<Vec<_>>();reports.sort();if reports!=*paths{return Err("ILP summary report selection differs from audited reports".into());}}
    expected.report_selection=selection;
    if normalize_summary_paths(&actual,path_base)!=expected{return Err("ilp audit summary mismatch: does not match current per-node reports".into());}
    Ok(())
}


/// A historical path is data only: it never becomes a filesystem capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeSummaryPathBinding {
    pub schema_version: &'static str,
    pub source_summary_sha256: String,
    pub source_path_prefix: String,
    pub bindings: Vec<NativeSummaryPathMatch>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeSummaryPathMatch {
    pub native_report: String,
    pub selected_report: String,
    pub native_model_path: Option<String>,
    pub selected_model_path: Option<String>,
}
fn portable_summary_name(path: &str) -> bool {
    !path.is_empty() && path.len() <= 240
        && path.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-/".contains(&b))
        && path.split('/').all(|p| !p.is_empty() && ![".", "..", "__proto__", "constructor", "prototype"].contains(&p))
}
// This text is never interpreted as a desktop filesystem location. Native
// producers may record relative, parent-relative, drive or UNC prefixes. Only
// complete selected names and one shared literal prefix affect comparison.
fn historical_summary_path_text(path: &str) -> String {
    path.replace('\\', "/")
}

/// Match a complete, explicitly selected native summary without changing its bytes.
/// Only one common historical text prefix may differ. All other typed fields, record
/// order, selected names, hashes and counters must match the independently audited
/// records. Old portable comparison stays first and unchanged. No path is opened.
pub fn verify_and_retain_portable_supplied_summary(
    bytes: &[u8], expected: &mut IlpAuditSummary,
) -> Result<Option<NativeSummaryPathBinding>, String> {
    use sha2::{Digest, Sha256};
    if bytes.len() > 1024 * 1024 { return Err("ILP summary exceeds 1 MiB.".into()); }
    let original_error = match verify_and_retain_supplied_summary(bytes, expected, Path::new("")) {
        Ok(()) => return Ok(None),
        Err(error) => error,
    };
    let candidate = (|| -> Option<(IlpAuditSummary, NativeSummaryPathBinding)> {
        let mut actual: IlpAuditSummary = serde_json::from_slice(bytes).ok()?;
        let raw: serde_json::Value = serde_json::from_slice(bytes).ok()?;
        // The compatibility branch accepts an actual native schema, not ignored
        // extra fields or ambiguous null/default representations.
        if serde_json::to_value(&actual).ok()? != raw { return None; }
        let selection = summary_report_selection(bytes).ok()??;
        let mut reports = expected.records.iter().map(|r| r.report.clone()).collect::<Vec<_>>();
        if reports.iter().any(|p| !portable_summary_name(p)) { return None; }
        reports.sort();
        if reports != selection || actual.records.len() != expected.records.len() { return None; }
        let mut common_prefix: Option<String> = None;
        let mut bindings = Vec::new();
        for (given, selected) in actual.records.iter_mut().zip(&expected.records) {
            let native_report = given.report.clone();
            let recorded = historical_summary_path_text(&native_report);
            let prefix = recorded.strip_suffix(&selected.report)?.to_owned();
            if !prefix.is_empty() && !prefix.ends_with('/') { return None; }
            if let Some(previous) = &common_prefix {
                if previous != &prefix { return None; }
            } else { common_prefix = Some(prefix.clone()); }
            let native_model_path = given.model_path.clone();
            match (&given.model_path, &selected.model_path) {
                (Some(source), Some(target)) if portable_summary_name(target) => {
                    if historical_summary_path_text(source) != format!("{prefix}{target}") { return None; }
                }
                (None, None) => {}
                _ => return None,
            }
            bindings.push(NativeSummaryPathMatch {
                native_report, selected_report: selected.report.clone(),
                native_model_path, selected_model_path: selected.model_path.clone(),
            });
            given.report = selected.report.clone();
            given.model_path = selected.model_path.clone();
        }
        let mut audited = expected.clone();
        audited.report_selection = Some(selection);
        if actual != audited { return None; }
        Some((audited, NativeSummaryPathBinding {
            schema_version: "bisect-ilp-native-summary-path-binding-v1",
            source_summary_sha256: format!("{:x}", Sha256::digest(bytes)),
            source_path_prefix: common_prefix?, bindings,
        }))
    })();
    match candidate {
        Some((audited, binding)) => {
            expected.report_selection = audited.report_selection;
            Ok(Some(binding))
        }
        None => Err(original_error),
    }
}

#[cfg(test)]
mod manifest_binding_tests {
    use super::*;
    #[test]
    fn readonly_wide_count_is_compared_without_truncation(){
        let binding:ReadOnlyManifestIlpBindings=serde_json::from_str(
            r#"{"ilp_audit_summary_path":"nodes/audit-summary.json","ilp_solve_report_count":18446744073709551615}"#
        ).unwrap();
        let mut summary=summarize_records(vec![]);summary.checked=1;
        let error=verify_manifest_ilp_summary(&binding,|_|panic!("no digest requested"),|_,_|Ok(summary.clone())).unwrap_err();
        assert_eq!(error,"ILP report count mismatch: manifest=18446744073709551615, audited=1");
    }
    #[test]
    fn declared_report_count_must_match_verified_summary(){
        for count in [0,1,3] {
            let b=ManifestIlpBindings{ilp_audit_summary_path:Some("nodes/audit-summary.json".into()),ilp_solve_report_count:Some(count),..Default::default()};
            let mut summary=summarize_records(vec![]);summary.checked=1;
            let result=verify_manifest_ilp_summary(&b,|_|panic!("no digest requested"),|_,_|Ok(summary.clone()));
            if count==1 {assert_eq!(result.unwrap().unwrap().summary.checked,1);}else{assert!(result.unwrap_err().contains("ILP report count mismatch"));}
        }
        let b:ManifestIlpBindings=serde_json::from_str(r#"{"ilp_audit_summary_path":"nodes/audit-summary.json","ilp_solve_report_count":2}"#).unwrap();
        assert_eq!(b.ilp_solve_report_count,Some(2));
        for json in [r#"{"ilp_solve_report_count":-1}"#,r#"{"ilp_solve_report_count":1.5}"#,r#"{"ilp_solve_report_count":"2"}"#]{assert!(serde_json::from_str::<ManifestIlpBindings>(json).is_err());}
    }
    #[test]
    fn absent_summary_does_not_open_sources(){assert!(verify_manifest_ilp_summary(&ManifestIlpBindings::default(),|_|panic!("unexpected hash"),|_,_|panic!("unexpected audit")).unwrap().is_none());}
    #[test]
    fn default_report_directory_is_summary_parent(){let b=ManifestIlpBindings{ilp_audit_summary_path:Some("nodes/audit-summary.json".into()),..Default::default()};let r=verify_manifest_ilp_summary(&b,|_|panic!("no expected digest"),|dir,path|{assert_eq!(dir,"nodes");assert_eq!(path,"nodes/audit-summary.json");Ok(summarize_records(vec![]))}).unwrap().unwrap();assert_eq!(r.report_directory,"nodes");}
    #[test]
    fn digest_failure_stops_directory_verification(){let b=ManifestIlpBindings{ilp_audit_summary_path:Some("audit-summary.json".into()),ilp_audit_summary_sha256:Some("expected".into()),..Default::default()};assert!(verify_manifest_ilp_summary(&b,|_|Ok("changed".into()),|_,_|panic!("unexpected directory audit")).unwrap_err().contains("SHA-256 mismatch"));}
    #[test]
    fn native_report_filter_excludes_only_exact_summary_basename(){assert!(is_solve_report_name(Path::new("nodes/node.json")));assert!(!is_solve_report_name(Path::new("nodes/audit-summary.json")));assert!(!is_solve_report_name(Path::new("node.JSON")));assert!(is_solve_report_name(Path::new("manifest.json")));assert!(is_solve_report_name(Path::new("renamed-summary.json")));}
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
    fn supplied_audit_matches_native_files_and_rejects_stale_summary() {
        let tmp=tempfile::TempDir::new().unwrap();
        let path=write_report(tmp.path(),"node_root",b"exact selected model bytes",sha256(b"exact selected model bytes"));
        let reports=BTreeMap::from([("node_root.json".into(),std::fs::read(&path).unwrap())]);
        let models=BTreeMap::from([("node_root.lp".into(),b"exact selected model bytes".to_vec())]);
        let memory=audit_supplied_reports(&reports,&models);
        let disk=summarize_reports(vec![path]);
        assert_eq!(memory,normalize_summary_paths(&disk,tmp.path()));
        let json=serde_json::to_vec(&memory).unwrap();
        verify_supplied_summary(&json,&memory,Path::new("")).unwrap();
        let bad_models=BTreeMap::from([("node_root.lp".into(),b"changed bytes".to_vec())]);
        let bad=audit_supplied_reports(&reports,&bad_models);
        assert_eq!(bad.failed,1);
        assert!(verify_supplied_summary(&json,&bad,Path::new("")).is_err());
        assert_eq!(audit_supplied_reports(&reports,&BTreeMap::new()).failed,1);
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

#[cfg(test)]
mod native_summary_path_binding_tests {
    use super::*;
    fn observed() -> IlpAuditSummary {
        summarize_records(["a", "b"].iter().map(|name| IlpAuditRecord {
            report: format!("{name}/node.json"), ok: true,
            outcome: Some("exact-plan".into()), proof_status: Some("proved-optimal".into()),
            fallback_required: Some(false), exact_search_strategy: Some("k2-branch-and-bound".into()),
            model_path: Some(format!("{name}/node.lp")), format: Some("cplex-lp".into()),
            sha256: Some("a".repeat(64)), error: None,
        }).collect())
    }
    fn historical(prefix: &str) -> serde_json::Value {
        let mut summary = observed();
        summary.report_selection = Some(summary.records.iter().map(|r|r.report.clone()).collect());
        for row in &mut summary.records {
            row.report = format!("{}/{}", prefix.trim_end_matches('/'), row.report);
            row.model_path = row.model_path.as_ref().map(|p|format!("{}/{p}", prefix.trim_end_matches('/')));
        }
        serde_json::to_value(summary).unwrap()
    }
    #[test]
    fn portable_exact_comparison_and_output_remain_unchanged() {
        let mut expected = observed(); let before=expected.clone();
        let bytes=serde_json::to_vec(&expected).unwrap();
        assert_eq!(verify_and_retain_portable_supplied_summary(&bytes,&mut expected).unwrap(),None);
        assert_eq!(expected,before);
    }
    #[test]
    fn absolute_prefixes_bind_full_names_without_opening_or_rewriting() {
        use sha2::{Digest,Sha256};
        for prefix in ["C:/lab/audit","/home/research/audit","//server/share/audit","/","C:/","/tmp/folder Ω","//?/C:/lab/audit","//?/UNC/server/share/audit"] {
            let mut expected=observed();let actual=historical(prefix);let bytes=serde_json::to_vec(&actual).unwrap();
            let binding=verify_and_retain_portable_supplied_summary(&bytes,&mut expected).unwrap().unwrap();
            assert_eq!(binding.source_summary_sha256,format!("{:x}",Sha256::digest(&bytes)));
            assert_eq!(binding.bindings.len(),2);assert_eq!(binding.bindings[0].selected_report,"a/node.json");
            assert_eq!(binding.bindings[0].native_report,actual["records"][0]["report"].as_str().unwrap());
            assert_eq!(expected.report_selection,Some(vec!["a/node.json".into(),"b/node.json".into()]));
            assert_eq!(serde_json::to_vec(&actual).unwrap(),bytes);
        }
        let mut actual=historical("C:/lab/audit");
        for row in actual["records"].as_array_mut().unwrap(){for key in ["report","model_path"]{row[key]=row[key].as_str().unwrap().replace('/',"\\").into();}}
        let result=verify_and_retain_portable_supplied_summary(&serde_json::to_vec(&actual).unwrap(),&mut observed()).unwrap().unwrap();
        assert_eq!(result.source_path_prefix,"C:/lab/audit/");
    }
    #[test]
    fn changed_metadata_names_scope_or_prefix_never_mutate_audited_state() {
        let original=historical("C:/lab/audit");let mut cases=Vec::new();
        macro_rules! altered {($name:expr,$edit:expr)=>{{let mut value=original.clone();($edit)(&mut value);cases.push(($name,value));}}}
        altered!("unknown summary authority",|v:&mut serde_json::Value|v["confirmed"]=true.into());
        altered!("unknown record authority",|v:&mut serde_json::Value|v["records"][0]["handle"]="x".into());
        altered!("missing scope",|v:&mut serde_json::Value|{v.as_object_mut().unwrap().remove("report_selection");});
        altered!("duplicate scope",|v:&mut serde_json::Value|v["report_selection"]=serde_json::json!(["a/node.json","a/node.json"]));
        altered!("wrong scope",|v:&mut serde_json::Value|v["report_selection"]=serde_json::json!(["a/node.json","c/node.json"]));
        altered!("reordered records",|v:&mut serde_json::Value|v["records"].as_array_mut().unwrap().reverse());
        altered!("extra record",|v:&mut serde_json::Value|{let row=v["records"][0].clone();v["records"].as_array_mut().unwrap().push(row);});
        altered!("missing record",|v:&mut serde_json::Value|{v["records"].as_array_mut().unwrap().pop();});
        for (key,value) in [("checked",1),("passed",1),("failed",1),("fallback_required",1)] {
            let mut actual=original.clone();actual[key]=value.into();cases.push((key,actual));
        }
        for (key,value) in [("sha256",serde_json::json!("b".repeat(64))),("outcome",serde_json::json!("changed")),("proof_status",serde_json::json!("changed")),("fallback_required",serde_json::json!(true)),("format",serde_json::json!("changed")),("exact_search_strategy",serde_json::json!("changed")),("error",serde_json::json!("changed"))] {
            let mut actual=original.clone();actual["records"][0][key]=value;cases.push((key,actual));
        }
        for path in ["C:/other/a/node.json","C:/lab/audit/a/renamed.json","C:/lab/../audit/a/node.json","C:/lab//audit/a/node.json","C:lab/audit/a/node.json","a/node.json","https://host/a/node.json","//server/a/node.json","C:/lab/audit/a/node.json\n"] {
            let mut actual=original.clone();actual["records"][0]["report"]=path.into();cases.push((path,actual));
        }
        for path in ["C:/other/a/node.lp","C:/lab/audit/a/../node.lp","C:/lab/audit/a/renamed.lp","c:/lab/audit/a/node.lp"] {
            let mut actual=original.clone();actual["records"][0]["model_path"]=path.into();cases.push((path,actual));
        }
        altered!("omitted model",|v:&mut serde_json::Value|{v["records"][0].as_object_mut().unwrap().remove("model_path");});
        for (name,actual) in cases {
            let mut expected=observed();let before=expected.clone();
            assert!(verify_and_retain_portable_supplied_summary(&serde_json::to_vec(&actual).unwrap(),&mut expected).is_err(),"accepted {name}");
            assert_eq!(expected,before,"mutated expected on {name}");
        }
    }
    #[test]
    fn long_extended_native_paths_use_the_existing_summary_byte_budget() {
        let prefix=format!("//?/C:/{}",vec!["x".repeat(100);64].join("/"));
        let actual=historical(&prefix);let bytes=serde_json::to_vec(&actual).unwrap();
        assert!(bytes.len()<1024*1024);let binding=verify_and_retain_portable_supplied_summary(&bytes,&mut observed()).unwrap().unwrap();
        assert_eq!(binding.source_path_prefix,format!("{prefix}/"));
    }
    #[test]
    fn recorded_locations_are_opaque_text_and_never_selected_model_names() {
        for prefix in ["//server", "//?/GLOBALROOT/Device/audit", "C:lab/audit", "https://historical.invalid/audit", "../recorded-output", "outputs/v1"] {
            let actual=historical(prefix);let mut expected=observed();
            let binding=verify_and_retain_portable_supplied_summary(&serde_json::to_vec(&actual).unwrap(),&mut expected).unwrap().unwrap();
            assert_eq!(binding.source_path_prefix,format!("{prefix}/"));
            assert!(expected.records.iter().all(|r|portable_summary_name(&r.report)&&r.model_path.as_ref().is_some_and(|p|portable_summary_name(p))));
            assert_eq!(binding.bindings[0].selected_report,"a/node.json");
        }
    }
    #[test]
    fn existing_summary_byte_budget_rejects_oversized_history_before_binding() {
        let actual=historical(&format!("C:/{}","x".repeat(300_000)));
        let bytes=serde_json::to_vec(&actual).unwrap();assert!(bytes.len()>1024*1024);
        let mut expected=observed();let before=expected.clone();
        assert!(verify_and_retain_portable_supplied_summary(&bytes,&mut expected).is_err());
        assert_eq!(expected,before);
    }
    #[test]
    fn explicit_all_selected_roster_cannot_be_narrowed_by_saved_scope() {
        let actual=historical("C:/lab/audit");let mut expected=observed();
        let mut extra=expected.records[0].clone();extra.report="old/node.json".into();extra.model_path=Some("old/node.lp".into());
        expected=summarize_records(expected.records.into_iter().chain([extra]).collect());let before=expected.clone();
        assert!(verify_and_retain_portable_supplied_summary(&serde_json::to_vec(&actual).unwrap(),&mut expected).is_err());
        assert_eq!(expected,before);
    }
}
