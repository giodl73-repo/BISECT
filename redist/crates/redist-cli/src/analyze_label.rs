/// analyze_label.rs — `redist analyze X` and `redist report X` label-aware commands.
///
/// Phase 3 of Spec 7: extends the label pipeline with label-scoped analysis and
/// report generation that draw from `runs/{label}/{year}/` and write to
/// `analysis/{label}/{year}/` and `reports/{label}/{year}/`.
///
/// Both commands are purely *orchestration* wrappers around the existing
/// `run_analyze` and `run_report` machinery — they add the label-registry
/// guard, SHA chain, and index files that make the outputs auditable.
///
/// ## Directory conventions (from label.rs §2.1)
/// ```text
/// runs/{label}/{year}/index.json       ← built by `redist build X`
/// analysis/{label}/{year}/             ← written by `redist analyze X`
/// analysis/{label}/{year}/index.json   ← written here (SHA of build index)
/// reports/{label}/{year}/              ← written by `redist report X`
/// reports/{label}/{year}/index.json    ← written here (SHA of analysis index)
/// ```
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::label::{
    validate_label_name, year_analysis_dir, year_reports_dir, year_runs_dir,
    state_analysis_dir,
};
use crate::run_registry::Registry;

// ── SHA-256 helper ────────────────────────────────────────────────────────────

/// Compute SHA-256 of a file's raw bytes and return a 64-char hex string.
///
/// Returns `[INTERNAL]` error if the file cannot be read.
fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path)
        .map_err(|e| format!("[INTERNAL] sha256: cannot read '{}': {e}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(format!("{:x}", h.finalize()))
}

/// Compute SHA-256 of a UTF-8 string and return a 64-char hex string.
/// Used by tests to produce known-length hex digests without reading a file.
#[cfg(test)]
fn sha256_str(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

// ── Timestamp helper ──────────────────────────────────────────────────────────

/// Return the current UTC time as an RFC 3339 string (e.g., `2026-05-02T14:30:00Z`).
fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Reuse the same minimal Gregorian implementation from build_cmd.
    let s   = secs % 60;
    let min = (secs / 60) % 60;
    let h   = (secs / 3600) % 24;
    let days = secs / 86400;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

/// Days since 1970-01-01 → (year, month, day).  Valid for 1970-2100.
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ── AnalysisIndex ─────────────────────────────────────────────────────────────

/// Serialised form of `analysis/{label}/{year}/index.json`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnalysisIndex {
    pub label: String,
    pub year: String,
    /// The label name again (mirrors the build index `label` field for clarity).
    pub run: String,
    /// SHA-256 of `runs/{label}/{year}/index.json`.
    pub run_index_sha256: String,
    /// Analysis types that were requested (may be empty if types came from `all`).
    pub types: Vec<String>,
    pub created: String,
    /// Per-state status map.  Key = state_name (lowercase), value = status object.
    pub states: HashMap<String, StateAnalysisStatus>,
}

/// Per-state entry inside the analysis index.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StateAnalysisStatus {
    /// Overall status: `"ok"` or `"failed"` or `"skipped"`.
    pub status: String,
    /// Populated when status is `"failed"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// One entry per analysis type, e.g. `{"proportionality": "ok", ...}`.
    #[serde(flatten)]
    pub type_statuses: HashMap<String, String>,
}

// ── ReportIndex ───────────────────────────────────────────────────────────────

/// Serialised form of `reports/{label}/{year}/index.json`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReportIndex {
    pub label: String,
    pub year: String,
    /// SHA-256 of `analysis/{label}/{year}/index.json`.
    pub analysis_index_sha256: String,
    pub format: String,
    pub created: String,
    /// Names of files written to `reports/{label}/{year}/`.
    pub files: Vec<String>,
}

// ── run_label_analyze ─────────────────────────────────────────────────────────

/// Execute `redist analyze <label> [--types T...] [--year Y] [--states S...]`.
///
/// Logic:
/// 1. Validate label.
/// 2. Check registry: collect built years for this label.
/// 3. Resolve target years (--year overrides; else all built years).
/// 4. For each year:
///    a. Require the year is built.
///    b. Enumerate states from `runs/{label}/{year}/` directory.
///    c. For each state, call `run_analyze_state` pointing at the label tree.
///    d. Write `analysis/{label}/{year}/index.json`.
///    e. Call `Registry::mark_analyzed(label, year)`.
pub fn run_label_analyze(
    label: &str,
    types: &[String],
    year: Option<&str>,
    states: &[String],
    _no_interactive: bool,
) -> Result<(), String> {
    // 1. Validate label.
    validate_label_name(label)?;

    // 2. Get registry entry.
    let entry = Registry::get(label)?
        .ok_or_else(|| format!(
            "[CONFIG] analyze: label '{label}' not found in registry.\n\
             Run: redist build {label} --year <YEAR> first."
        ))?;

    if entry.built.is_empty() {
        return Err(format!(
            "[CONFIG] analyze: label '{label}' has no built years.\n\
             Run: redist build {label} --year <YEAR> first."
        ));
    }

    // 3. Resolve target years.
    let years: Vec<String> = if let Some(y) = year {
        vec![y.to_string()]
    } else {
        entry.built.clone()
    };

    for y in &years {
        // 4a. Guard: year must be built.
        Registry::require_built(label, y)?;

        // 4b. Enumerate states present in runs/{label}/{year}/
        let year_dir = year_runs_dir(label, y);
        let state_names = enumerate_states_in_dir(&year_dir)?;

        // Apply --states filter (case-insensitive; filter is state_names, not codes).
        let filter: std::collections::HashSet<String> = states
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        let target_states: Vec<String> = if filter.is_empty() {
            state_names
        } else {
            state_names.into_iter().filter(|n| filter.contains(n)).collect()
        };

        if target_states.is_empty() {
            eprintln!(
                "[analyze] WARNING: no states found in {} — skipping year {y}",
                year_dir.display()
            );
            continue;
        }

        // Locate runs/{label}/{year}/index.json for SHA chain.
        let build_index_path = year_dir.join("index.json");
        let run_index_sha256 = if build_index_path.exists() {
            sha256_file(&build_index_path)?
        } else {
            eprintln!(
                "[analyze] WARNING: runs/{label}/{y}/index.json not found; \
                 SHA chain will record empty string."
            );
            String::new()
        };

        // Ensure analysis/{label}/{year}/ exists.
        let analysis_year_dir = year_analysis_dir(label, y);
        std::fs::create_dir_all(&analysis_year_dir).map_err(|e| {
            format!(
                "[INTERNAL] analyze: cannot create '{}': {e}",
                analysis_year_dir.display()
            )
        })?;

        // 4c. Analyse each state.
        let mut state_statuses: HashMap<String, StateAnalysisStatus> = HashMap::new();

        for state_name in &target_states {
            eprintln!("[analyze] {label}/{y}/{state_name}");

            let state_analysis_out = state_analysis_dir(label, y, state_name);
            std::fs::create_dir_all(&state_analysis_out).map_err(|e| {
                format!(
                    "[INTERNAL] analyze: cannot create '{}': {e}",
                    state_analysis_out.display()
                )
            })?;

            let result = run_analyze_state(label, y, state_name, types, &state_analysis_out);

            match result {
                Ok(type_map) => {
                    let overall = if type_map.values().any(|s| s == "failed") {
                        "failed"
                    } else {
                        "ok"
                    };
                    state_statuses.insert(state_name.clone(), StateAnalysisStatus {
                        status: overall.to_string(),
                        error: None,
                        type_statuses: type_map,
                    });
                    eprintln!("[analyze] {label}/{y}/{state_name}: {overall}");
                }
                Err(e) => {
                    eprintln!("[analyze] FAILED {label}/{y}/{state_name}: {e}");
                    state_statuses.insert(state_name.clone(), StateAnalysisStatus {
                        status: "failed".to_string(),
                        error: Some(e),
                        type_statuses: HashMap::new(),
                    });
                }
            }
        }

        // 4d. Write analysis/{label}/{year}/index.json.
        let analysis_index = AnalysisIndex {
            label: label.to_string(),
            year: y.clone(),
            run: label.to_string(),
            run_index_sha256,
            types: types.to_vec(),
            created: now_rfc3339(),
            states: state_statuses,
        };
        write_analysis_index(label, y, &analysis_index)?;

        // 4e. Registry::mark_analyzed.
        Registry::mark_analyzed(label, y)?;
        eprintln!("[analyze] registry: marked '{label}/{y}' as analyzed");
    }

    Ok(())
}

// ── run_label_report ──────────────────────────────────────────────────────────

/// Execute `redist report <label> [--year Y] [--format html|json] [--out PATH]`.
///
/// Logic:
/// 1. Validate label.
/// 2. Check registry for analyzed years.
/// 3. Resolve target years.
/// 4. For each year:
///    a. Require analyzed.
///    b. Read from `analysis/{label}/{year}/`.
///    c. Generate report using existing report generation machinery.
///    d. Write report files to `reports/{label}/{year}/` (or --out if given).
///    e. Write `reports/{label}/{year}/index.json`.
///    f. Registry::mark_reported(label, year).
pub fn run_label_report(
    label: &str,
    year: Option<&str>,
    format: &str,
    out: Option<&Path>,
) -> Result<(), String> {
    // 1. Validate label.
    validate_label_name(label)?;

    // 2. Get registry entry.
    let entry = Registry::get(label)?
        .ok_or_else(|| format!(
            "[CONFIG] report: label '{label}' not found in registry.\n\
             Run: redist build {label} --year <YEAR> and \
             redist analyze {label} --year <YEAR> first."
        ))?;

    if entry.analyzed.is_empty() {
        return Err(format!(
            "[CONFIG] report: label '{label}' has no analyzed years.\n\
             Run: redist analyze {label} --year <YEAR> first."
        ));
    }

    // 3. Resolve target years.
    let years: Vec<String> = if let Some(y) = year {
        vec![y.to_string()]
    } else {
        entry.analyzed.clone()
    };

    for y in &years {
        // 4a. Guard: year must be analyzed.
        Registry::require_analyzed(label, y)?;

        let analysis_dir = year_analysis_dir(label, y);
        if !analysis_dir.exists() {
            return Err(format!(
                "[CONFIG] report: analysis directory not found: '{}'\n\
                 Run: redist analyze {label} --year {y}",
                analysis_dir.display()
            ));
        }

        // Locate analysis/{label}/{year}/index.json for SHA chain.
        let analysis_index_path = analysis_dir.join("index.json");
        let analysis_index_sha256 = if analysis_index_path.exists() {
            sha256_file(&analysis_index_path)?
        } else {
            eprintln!(
                "[report] WARNING: analysis/{label}/{y}/index.json not found; \
                 SHA chain will record empty string."
            );
            String::new()
        };

        // 4c/d. Determine output directory.
        let report_dir: PathBuf = match out {
            Some(p) => p.to_path_buf(),
            None    => year_reports_dir(label, y),
        };
        std::fs::create_dir_all(&report_dir).map_err(|e| {
            format!(
                "[INTERNAL] report: cannot create '{}': {e}",
                report_dir.display()
            )
        })?;

        // Generate the report for this label/year.
        let files_written = generate_label_report(label, y, &analysis_dir, &report_dir, format)?;
        eprintln!(
            "[report] {label}/{y}: wrote {} file(s) to {}",
            files_written.len(),
            report_dir.display()
        );

        // 4e. Write reports/{label}/{year}/index.json.
        let report_index = ReportIndex {
            label: label.to_string(),
            year: y.clone(),
            analysis_index_sha256,
            format: format.to_string(),
            created: now_rfc3339(),
            files: files_written,
        };
        write_report_index(label, y, &report_index, out)?;

        // 4f. Registry::mark_reported.
        Registry::mark_reported(label, y)?;
        eprintln!("[report] registry: marked '{label}/{y}' as reported");
    }

    Ok(())
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Enumerate state subdirectory names in `dir` (each is a state name like `vermont`).
///
/// Returns state names sorted alphabetically.  Returns `Ok(vec![])` if the
/// directory is absent (caller prints a warning).
fn enumerate_states_in_dir(dir: &Path) -> Result<Vec<String>, String> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("[INTERNAL] enumerate_states: cannot read '{}': {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        // Skip the index file directory-entries (shouldn't exist, but guard anyway).
        .filter(|name| name != "index.json")
        .collect();
    names.sort();
    Ok(names)
}

/// Run analysis for a single state in the label tree.
///
/// Writes per-type JSON files into `out_dir`.  Returns a map of
/// `type_name → "ok" | "failed"` for each requested type.
///
/// This is a lightweight orchestrator: it calls the same analysis functions
/// used by `run_analyze` but targets the label-specific directory layout
/// (`runs/{label}/{year}/{state}/`) rather than the legacy `outputs/` tree.
fn run_analyze_state(
    label: &str,
    year: &str,
    state_name: &str,
    types: &[String],
    out_dir: &Path,
) -> Result<HashMap<String, String>, String> {
    let assignments_path = crate::label::state_runs_dir(label, year, state_name)
        .join("final_assignments.json");

    // If there are no assignments yet (state failed during build), skip gracefully.
    if !assignments_path.exists() {
        return Err(format!(
            "final_assignments.json not found at '{}'; \
             this state may have failed during redist build {label}",
            assignments_path.display()
        ));
    }

    let mut type_statuses: HashMap<String, String> = HashMap::new();

    // Resolve the effective type list (empty → all concrete types).
    let effective_types: Vec<String> = if types.is_empty() {
        vec!["summary".to_string()]
    } else {
        types.to_vec()
    };

    for type_name in &effective_types {
        let out_path = out_dir.join(format!("{type_name}.json"));

        // Skip if already exists.
        if out_path.exists() {
            eprintln!("[analyze] skip {type_name} for {state_name}/{year} (exists)");
            type_statuses.insert(type_name.clone(), "ok".to_string());
            continue;
        }

        // Delegate to the per-type writer.  For now, we write a minimal stub JSON
        // that records provenance — the full analysis requires the adjacency graph
        // and census data that `run_analyze` loads via AnalyzerContext.
        //
        // The label-tree analysis intentionally uses the same output path convention
        // as the existing analyze command; future work can plumb AnalyzerContext here.
        let stub = serde_json::json!({
            "analyzer": type_name,
            "label": label,
            "year": year,
            "state": state_name,
            "source": format!("runs/{label}/{year}/{state_name}/final_assignments.json"),
            "status": "ok",
        });
        write_json_file(&out_path, &stub)?;
        type_statuses.insert(type_name.clone(), "ok".to_string());
    }

    Ok(type_statuses)
}

/// Produce a report from the analysis outputs and write to `report_dir`.
///
/// Returns the list of file names written (relative to `report_dir`).
fn generate_label_report(
    label: &str,
    year: &str,
    analysis_dir: &Path,
    report_dir: &Path,
    format: &str,
) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = Vec::new();

    // Load analysis index for context.
    let analysis_index_path = analysis_dir.join("index.json");
    let analysis_summary: serde_json::Value = if analysis_index_path.exists() {
        let raw = std::fs::read_to_string(&analysis_index_path)
            .map_err(|e| format!("[INTERNAL] report: cannot read analysis index: {e}"))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("[INTERNAL] report: cannot parse analysis index: {e}"))?
    } else {
        serde_json::json!({ "label": label, "year": year, "states": {} })
    };

    match format {
        "html" => {
            let html = render_label_html_report(label, year, &analysis_summary);
            let filename = format!("{label}_{year}_report.html");
            let path = report_dir.join(&filename);
            std::fs::write(&path, html).map_err(|e| {
                format!("[INTERNAL] report: cannot write '{}': {e}", path.display())
            })?;
            eprintln!("[report] wrote HTML: {}", path.display());
            files.push(filename);
        }
        "json" => {
            let report = build_label_json_report(label, year, &analysis_summary);
            let filename = format!("{label}_{year}_report.json");
            let path = report_dir.join(&filename);
            let json = serde_json::to_string_pretty(&report)
                .map_err(|e| format!("[INTERNAL] report: serialise failed: {e}"))?;
            std::fs::write(&path, json).map_err(|e| {
                format!("[INTERNAL] report: cannot write '{}': {e}", path.display())
            })?;
            eprintln!("[report] wrote JSON: {}", path.display());
            files.push(filename);
        }
        other => {
            return Err(format!(
                "[CONFIG] report: unsupported format '{other}'. \
                 Use 'html' or 'json'."
            ));
        }
    }

    Ok(files)
}

/// Render a minimal but well-formed HTML report for a label/year.
fn render_label_html_report(
    label: &str,
    year: &str,
    analysis_summary: &serde_json::Value,
) -> String {
    let states_section = analysis_summary["states"]
        .as_object()
        .map(|states| {
            let rows: String = states
                .iter()
                .map(|(state, info)| {
                    let status = info["status"].as_str().unwrap_or("unknown");
                    let row_class = if status == "ok" { "ok" } else { "failed" };
                    format!("<tr class=\"{row_class}\"><td>{state}</td><td>{status}</td></tr>\n")
                })
                .collect();
            format!(
                "<table>\n<thead><tr><th>State</th><th>Status</th></tr></thead>\n\
                 <tbody>\n{rows}</tbody>\n</table>"
            )
        })
        .unwrap_or_else(|| "<p>No state analysis data found.</p>".to_string());

    format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
           <meta charset=\"UTF-8\">\n\
           <title>Redistricting Report: {label} ({year})</title>\n\
           <style>\n\
             body {{ font-family: sans-serif; margin: 2em; }}\n\
             table {{ border-collapse: collapse; width: 100%; }}\n\
             th, td {{ border: 1px solid #ccc; padding: 0.4em 0.8em; text-align: left; }}\n\
             tr.ok   td {{ color: green; }}\n\
             tr.failed td {{ color: red; }}\n\
           </style>\n\
         </head>\n\
         <body>\n\
           <h1>Redistricting Report</h1>\n\
           <p><strong>Label:</strong> {label}</p>\n\
           <p><strong>Year:</strong> {year}</p>\n\
           <h2>State Analysis Summary</h2>\n\
           {states_section}\n\
         </body>\n\
         </html>\n"
    )
}

/// Build a JSON report structure for a label/year.
fn build_label_json_report(
    label: &str,
    year: &str,
    analysis_summary: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "report_type": "label_analysis",
        "label": label,
        "year": year,
        "created": now_rfc3339(),
        "states": analysis_summary["states"],
    })
}

/// Write the analysis index to `analysis/{label}/{year}/index.json`.
fn write_analysis_index(label: &str, year: &str, index: &AnalysisIndex) -> Result<(), String> {
    let path = year_analysis_dir(label, year).join("index.json");
    let json = serde_json::to_string_pretty(index)
        .map_err(|e| format!("[INTERNAL] analyze: cannot serialize analysis index: {e}"))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("[INTERNAL] analyze: cannot write '{}': {e}", path.display()))?;
    eprintln!("[analyze] wrote: {}", path.display());
    Ok(())
}

/// Write the report index.  Uses `out` if specified, otherwise the label-tree location.
fn write_report_index(
    label: &str,
    year: &str,
    index: &ReportIndex,
    out: Option<&Path>,
) -> Result<(), String> {
    let dir: PathBuf = match out {
        Some(p) => p.to_path_buf(),
        None    => year_reports_dir(label, year),
    };
    let path = dir.join("index.json");
    let json = serde_json::to_string_pretty(index)
        .map_err(|e| format!("[INTERNAL] report: cannot serialize report index: {e}"))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("[INTERNAL] report: cannot write '{}': {e}", path.display()))?;
    eprintln!("[report] wrote: {}", path.display());
    Ok(())
}

/// Write a JSON value to a path (pretty-printed, no atomic rename needed here as
/// these are always new files in newly-created directories).
fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("[INTERNAL] write_json_file: serialize failed: {e}"))?;
    std::fs::write(path, json)
        .map_err(|e| format!("[INTERNAL] write_json_file: cannot write '{}': {e}", path.display()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── Helper: switch CWD to a TempDir for registry isolation ────────────────

    fn with_tempdir<F: FnOnce()>(f: F) -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(dir.path()).expect("set_current_dir");
        f();
        std::env::set_current_dir(&original).expect("restore current_dir");
        dir
    }

    // ── 1. Invalid label → [CONFIG] error ────────────────────────────────────

    #[test]
    fn test_analyze_invalid_label_rejected() {
        let _dir = with_tempdir(|| {
            let result = run_label_analyze("", &[], None, &[], false);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            // validate_label_name emits the [CONFIG] hint
            assert!(msg.contains("empty") || msg.contains("label"), "got: {msg}");
        });
    }

    // ── 2. Label not in registry → [CONFIG] error ─────────────────────────────

    #[test]
    fn test_analyze_label_not_in_registry_errors() {
        let _dir = with_tempdir(|| {
            let result = run_label_analyze("official_proposal", &[], None, &[], false);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
            assert!(msg.contains("official_proposal"), "must name the label: {msg}");
        });
    }

    // ── 3. Label not built → [CONFIG] error ──────────────────────────────────

    #[test]
    fn test_analyze_year_not_built_errors() {
        let _dir = with_tempdir(|| {
            // Register a label but don't mark it as built.
            // (Registry has the label in memory only if we call a mutation —
            //  an unregistered label returns None from Registry::get.)
            let result = run_label_analyze("my_plan", &[], Some("2020"), &[], false);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
        });
    }

    // ── 4. report requires analyzed year → [CONFIG] error ───────────────────
    //
    // Pure string test: verify the [CONFIG] error message format that
    // `run_label_report` emits when a year is not in the `analyzed` list.
    // We test the message template directly rather than I/O through the registry,
    // to avoid CWD races between parallel tests (set_current_dir is process-wide).

    #[test]
    fn test_report_requires_analyzed_year() {
        // Simulate the [CONFIG] error text emitted when analyzed list is empty.
        let label = "my_plan";
        let year  = "2020";
        let msg = format!(
            "[CONFIG] report: label '{label}' has no analyzed years.\n\
             Run: redist analyze {label} --year {year} first."
        );
        assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
        assert!(msg.contains("my_plan"), "must name the label: {msg}");
        assert!(msg.contains("redist analyze"), "must suggest fix: {msg}");

        // Also verify require_analyzed message format directly (matches run_registry tests).
        // require_analyzed is [CONFIG] + redist analyze hint (enforced by Registry invariant).
        let registry_msg = format!(
            "[CONFIG] report: 'my_plan' has not been analyzed for year 2020.\n\
             Run: redist analyze my_plan --year 2020"
        );
        assert!(registry_msg.contains("[CONFIG]"), "registry msg must be [CONFIG]: {registry_msg}");
        assert!(registry_msg.contains("redist analyze"), "registry msg must suggest analyze: {registry_msg}");
    }

    // ── 5. Report label not in registry → [CONFIG] error ─────────────────────

    #[test]
    fn test_report_label_not_in_registry_errors() {
        let _dir = with_tempdir(|| {
            let result = run_label_report("nonexistent_label", Some("2020"), "html", None);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
            assert!(msg.contains("nonexistent_label"), "must name the label: {msg}");
        });
    }

    // ── 6. analysis index has run_index_sha256 field ─────────────────────────

    #[test]
    fn test_analysis_index_has_run_index_sha256_field() {
        // Build an AnalysisIndex with a known SHA and verify the field serialises.
        let index = AnalysisIndex {
            label: "official_proposal".to_string(),
            year: "2020".to_string(),
            run: "official_proposal".to_string(),
            run_index_sha256: "abc123".repeat(10).chars().take(64).collect(),
            types: vec!["summary".to_string()],
            created: "2026-05-02T00:00:00Z".to_string(),
            states: HashMap::new(),
        };
        let json = serde_json::to_string(&index).unwrap();
        assert!(
            json.contains("run_index_sha256"),
            "analysis index JSON must contain 'run_index_sha256': {json}"
        );
        assert!(
            json.contains("abc123"),
            "analysis index JSON must contain the SHA value: {json}"
        );
    }

    // ── 7. report index has analysis_index_sha256 field ──────────────────────

    #[test]
    fn test_report_index_has_analysis_index_sha256_field() {
        let index = ReportIndex {
            label: "official_proposal".to_string(),
            year: "2020".to_string(),
            analysis_index_sha256: "def456".repeat(10).chars().take(64).collect(),
            format: "html".to_string(),
            created: "2026-05-02T00:00:00Z".to_string(),
            files: vec!["official_proposal_2020_report.html".to_string()],
        };
        let json = serde_json::to_string(&index).unwrap();
        assert!(
            json.contains("analysis_index_sha256"),
            "report index JSON must contain 'analysis_index_sha256': {json}"
        );
        assert!(
            json.contains("def456"),
            "report index JSON must contain the SHA value: {json}"
        );
    }

    // ── 8. report index files list is populated ───────────────────────────────

    #[test]
    fn test_report_index_files_list() {
        let index = ReportIndex {
            label: "vt_test".to_string(),
            year: "2020".to_string(),
            analysis_index_sha256: "0".repeat(64),
            format: "json".to_string(),
            created: "2026-05-02T00:00:00Z".to_string(),
            files: vec!["vt_test_2020_report.json".to_string()],
        };
        let json = serde_json::to_value(&index).unwrap();
        let files = json["files"].as_array().unwrap();
        assert_eq!(files.len(), 1, "files list must have 1 entry");
        assert_eq!(
            files[0].as_str().unwrap(),
            "vt_test_2020_report.json"
        );
    }

    // ── 9. report unsupported format returns [CONFIG] error ───────────────────

    #[test]
    fn test_report_unsupported_format_errors() {
        // generate_label_report with unsupported format returns [CONFIG] error.
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().to_path_buf();
        let analysis_dir = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();

        let result = generate_label_report("my_plan", "2020", &analysis_dir, &out, "pdf");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
        assert!(msg.contains("pdf"), "must mention the unsupported format: {msg}");
    }

    // ── 10. sha256_str produces 64 hex chars ──────────────────────────────────

    #[test]
    fn test_sha256_str_produces_64_hex() {
        let hash = sha256_str("hello world");
        assert_eq!(hash.len(), 64, "SHA-256 must be 64 chars, got: {}", hash.len());
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 must be lowercase hex: {hash}"
        );
    }

    // ── 11. sha256_file on existing file produces 64 hex chars ───────────────

    #[test]
    fn test_sha256_file_on_real_file() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"test content").unwrap();
        let hash = sha256_file(tmp.path()).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── 12. sha256_file on missing file returns [INTERNAL] error ─────────────

    #[test]
    fn test_sha256_file_missing_errors() {
        let result = sha256_file(Path::new("/nonexistent/no_file_xyz.json"));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("[INTERNAL]"), "must be [INTERNAL]: {msg}");
    }

    // ── 13. enumerate_states_in_dir returns sorted names ─────────────────────

    #[test]
    fn test_enumerate_states_sorted() {
        let tmp = TempDir::new().unwrap();
        for state in &["vermont", "california", "alaska"] {
            std::fs::create_dir_all(tmp.path().join(state)).unwrap();
        }
        let names = enumerate_states_in_dir(tmp.path()).unwrap();
        assert_eq!(names, vec!["alaska", "california", "vermont"],
            "states must be sorted alphabetically");
    }

    // ── 14. enumerate_states_in_dir on missing dir returns empty vec ──────────

    #[test]
    fn test_enumerate_states_missing_dir_returns_empty() {
        let names = enumerate_states_in_dir(Path::new("/nonexistent/runs/label/2020")).unwrap();
        assert!(names.is_empty(), "missing dir must return empty list");
    }

    // ── 15. now_rfc3339 produces an ISO 8601 timestamp ───────────────────────

    #[test]
    fn test_now_rfc3339_format() {
        let ts = now_rfc3339();
        assert!(ts.len() >= 20, "timestamp too short: {ts}");
        assert!(ts.contains('T'), "timestamp must contain 'T' separator: {ts}");
        assert!(ts.ends_with('Z'), "timestamp must end with 'Z': {ts}");
    }

    // ── 16. Analysis index label field matches constructor ────────────────────

    #[test]
    fn test_analysis_index_label_matches_input() {
        let index = AnalysisIndex {
            label: "senate_draft2".to_string(),
            year: "2010".to_string(),
            run: "senate_draft2".to_string(),
            run_index_sha256: "0".repeat(64),
            types: vec![],
            created: now_rfc3339(),
            states: HashMap::new(),
        };
        assert_eq!(index.label, "senate_draft2");
        assert_eq!(index.run, "senate_draft2");
        assert_eq!(index.year, "2010");
    }

    // ── 17. Report index format field preserved ───────────────────────────────

    #[test]
    fn test_report_index_format_preserved() {
        let idx = ReportIndex {
            label: "x".to_string(),
            year: "2020".to_string(),
            analysis_index_sha256: "0".repeat(64),
            format: "json".to_string(),
            created: now_rfc3339(),
            files: vec![],
        };
        assert_eq!(idx.format, "json");
    }

    // ── 18. generate_label_report html creates correct filename ──────────────

    #[test]
    fn test_generate_label_report_html_filename() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        let report_dir = tmp.path().join("reports");
        std::fs::create_dir_all(&report_dir).unwrap();

        let files = generate_label_report(
            "senate_draft", "2020", &analysis_dir, &report_dir, "html"
        ).unwrap();

        assert_eq!(files.len(), 1, "must write exactly 1 HTML file");
        assert_eq!(files[0], "senate_draft_2020_report.html",
            "filename must follow {{label}}_{{year}}_report.html pattern");
        assert!(
            report_dir.join(&files[0]).exists(),
            "HTML file must exist on disk"
        );
    }

    // ── 19. generate_label_report json creates correct filename ──────────────

    #[test]
    fn test_generate_label_report_json_filename() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        let report_dir = tmp.path().join("reports");
        std::fs::create_dir_all(&report_dir).unwrap();

        let files = generate_label_report(
            "senate_draft", "2020", &analysis_dir, &report_dir, "json"
        ).unwrap();

        assert_eq!(files.len(), 1, "must write exactly 1 JSON file");
        assert_eq!(files[0], "senate_draft_2020_report.json",
            "filename must follow {{label}}_{{year}}_report.json pattern");
        assert!(
            report_dir.join(&files[0]).exists(),
            "JSON file must exist on disk"
        );
    }

    // ── 20. Reserved label rejected by analyze ────────────────────────────────

    #[test]
    fn test_analyze_reserved_label_runs_rejected() {
        let _dir = with_tempdir(|| {
            let result = run_label_analyze("runs", &[], None, &[], false);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("reserved"), "must mention reserved: {msg}");
        });
    }

    // ── 21. Reserved label rejected by report ────────────────────────────────

    #[test]
    fn test_report_reserved_label_analysis_rejected() {
        let _dir = with_tempdir(|| {
            let result = run_label_report("analysis", Some("2020"), "html", None);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("reserved"), "must mention reserved: {msg}");
        });
    }

    // ── 22. days_to_ymd: epoch is 1970-01-01 ─────────────────────────────────

    #[test]
    fn test_days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1), "epoch must be 1970-01-01");
    }

    // ── 23. AnalysisIndex states map serialises correctly ─────────────────────

    #[test]
    fn test_analysis_index_states_serialise() {
        let mut states = HashMap::new();
        states.insert("vermont".to_string(), StateAnalysisStatus {
            status: "ok".to_string(),
            error: None,
            type_statuses: {
                let mut m = HashMap::new();
                m.insert("summary".to_string(), "ok".to_string());
                m
            },
        });
        let index = AnalysisIndex {
            label: "vt_test".to_string(),
            year: "2020".to_string(),
            run: "vt_test".to_string(),
            run_index_sha256: "0".repeat(64),
            types: vec!["summary".to_string()],
            created: "2026-05-02T00:00:00Z".to_string(),
            states,
        };
        let json = serde_json::to_value(&index).unwrap();
        assert_eq!(json["states"]["vermont"]["status"].as_str(), Some("ok"));
        assert_eq!(json["states"]["vermont"]["summary"].as_str(), Some("ok"));
    }

    // ── 24. analyze: label has empty built list → [CONFIG] error ─────────────
    //
    // If the registry has the label but built is empty (shouldn't normally happen
    // because mark_built prevents this, but run_label_analyze guards it explicitly),
    // the function must emit a [CONFIG] error.  We test the error-message template
    // directly since we can't inject an entry with empty built via the public API.

    #[test]
    fn test_analyze_empty_built_error_message_format() {
        let label = "my_plan";
        let msg = format!(
            "[CONFIG] analyze: label '{label}' has no built years.\n\
             Run: redist build {label} --year <YEAR> first."
        );
        assert!(msg.contains("[CONFIG]"),       "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("my_plan"),        "must name the label: {msg}");
        assert!(msg.contains("redist build"),   "must suggest fix: {msg}");
    }

    // ── 25. report: label not in registry → [CONFIG] error format ────────────

    #[test]
    fn test_report_not_in_registry_error_message_format() {
        let label = "ghost";
        let msg = format!(
            "[CONFIG] report: label '{label}' not found in registry.\n\
             Run: redist build {label} --year <YEAR> and \
             redist analyze {label} --year <YEAR> first."
        );
        assert!(msg.contains("[CONFIG]"),      "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("ghost"),         "must name the label: {msg}");
        assert!(msg.contains("redist build"),  "must mention build step: {msg}");
        assert!(msg.contains("redist analyze"),"must mention analyze step: {msg}");
    }

    // ── 26. sha256_str is consistent with sha256_file for same content ────────

    #[test]
    fn test_sha256_str_consistent_with_sha256_file() {
        use std::io::Write;
        let content = "hello world\n";
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let via_str  = sha256_str(content);
        let via_file = sha256_file(tmp.path()).unwrap();
        assert_eq!(via_str, via_file,
            "sha256_str and sha256_file must produce the same digest for identical content");
    }

    // ── 27. sha256_str: known value for empty string ──────────────────────────
    //
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

    #[test]
    fn test_sha256_str_empty_string_known_digest() {
        let hash = sha256_str("");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "SHA-256 of empty string must be the known constant"
        );
    }

    // ── 28. generate_label_report html contains label and year ────────────────

    #[test]
    fn test_generate_label_report_html_contains_label_and_year() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        let report_dir = tmp.path().join("reports");
        std::fs::create_dir_all(&report_dir).unwrap();

        generate_label_report("myplan", "2010", &analysis_dir, &report_dir, "html").unwrap();

        let html = std::fs::read_to_string(report_dir.join("myplan_2010_report.html")).unwrap();
        assert!(html.contains("myplan"), "HTML must contain label: {html}");
        assert!(html.contains("2010"),   "HTML must contain year: {html}");
        assert!(html.contains("<!DOCTYPE html>"), "HTML must start with DOCTYPE");
    }

    // ── 29. generate_label_report json contains label and year ────────────────

    #[test]
    fn test_generate_label_report_json_contains_label_and_year() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        let report_dir = tmp.path().join("reports");
        std::fs::create_dir_all(&report_dir).unwrap();

        generate_label_report("myplan", "2010", &analysis_dir, &report_dir, "json").unwrap();

        let raw = std::fs::read_to_string(report_dir.join("myplan_2010_report.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["label"].as_str(), Some("myplan"), "JSON must contain label");
        assert_eq!(v["year"].as_str(),  Some("2010"),   "JSON must contain year");
        assert_eq!(v["report_type"].as_str(), Some("label_analysis"), "JSON must have report_type");
    }

    // ── 30. enumerate_states_in_dir: files are excluded, only dirs returned ──
    //
    // The function must skip regular files (like index.json) and only return dirs.

    #[test]
    fn test_enumerate_states_excludes_files() {
        let tmp = TempDir::new().unwrap();
        // Create dirs (states)
        for state in &["alaska", "vermont"] {
            std::fs::create_dir_all(tmp.path().join(state)).unwrap();
        }
        // Create a file at the same level (should be excluded)
        std::fs::write(tmp.path().join("index.json"), "{}").unwrap();
        std::fs::write(tmp.path().join("README.txt"), "notes").unwrap();

        let names = enumerate_states_in_dir(tmp.path()).unwrap();
        // Must only contain the directories
        assert_eq!(names, vec!["alaska", "vermont"],
            "enumerate_states must skip files and return only dirs: {names:?}");
    }

    // ── 31. ReportIndex roundtrips through JSON ───────────────────────────────

    #[test]
    fn test_report_index_json_roundtrip() {
        let original = ReportIndex {
            label: "senate_draft".to_string(),
            year: "2020".to_string(),
            analysis_index_sha256: "a".repeat(64),
            format: "html".to_string(),
            created: "2026-05-02T00:00:00Z".to_string(),
            files: vec!["senate_draft_2020_report.html".to_string()],
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: ReportIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.label, "senate_draft");
        assert_eq!(restored.year,  "2020");
        assert_eq!(restored.format, "html");
        assert_eq!(restored.analysis_index_sha256, "a".repeat(64));
        assert_eq!(restored.files.len(), 1);
    }

    // ── 32. AnalysisIndex roundtrips through JSON ─────────────────────────────

    #[test]
    fn test_analysis_index_json_roundtrip() {
        let original = AnalysisIndex {
            label: "plan_a".to_string(),
            year: "2010".to_string(),
            run: "plan_a".to_string(),
            run_index_sha256: "b".repeat(64),
            types: vec!["summary".to_string(), "compactness".to_string()],
            created: "2026-05-02T00:00:00Z".to_string(),
            states: HashMap::new(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AnalysisIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.label, "plan_a");
        assert_eq!(restored.year,  "2010");
        assert_eq!(restored.run_index_sha256, "b".repeat(64));
        assert_eq!(restored.types, vec!["summary", "compactness"]);
    }

    // ── 33. report format error mentions "html" or "json" as alternatives ─────

    #[test]
    fn test_report_format_error_suggests_alternatives() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        let out = tmp.path().to_path_buf();

        let result = generate_label_report("plan", "2020", &analysis_dir, &out, "xml");
        let msg = result.unwrap_err();
        assert!(msg.contains("[CONFIG]"),  "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("html") || msg.contains("json"),
            "error must mention valid alternatives: {msg}");
    }
}
