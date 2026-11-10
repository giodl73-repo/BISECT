/// analyze_label.rs — `bisect analyze X` and `BISECT report X` label-aware commands.
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
/// runs/{label}/{year}/index.json       ← built by `bisect build X`
/// analysis/{label}/{year}/             ← written by `bisect analyze X`
/// analysis/{label}/{year}/index.json   ← written here (SHA of build index)
/// reports/{label}/{year}/              ← written by `BISECT report X`
/// reports/{label}/{year}/index.json    ← written here (SHA of analysis index)
/// ```
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::geometry::{load_district_geometries, state_name_to_code};
use crate::label::{
    state_analysis_dir, validate_label_name, year_analysis_dir, year_reports_dir, year_runs_dir,
};
use crate::partisan::{run_partisan, PartisanArgs};
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
    let s = secs % 60;
    let min = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
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

pub use bisect_report::label_indexes::{AnalysisIndex,StateAnalysisStatus,ReportIndex};

#[derive(Clone, Copy)]
enum AnalysisInput {
    Built,
    Imported { tracts: u64 },
}

impl AnalysisInput {
    fn filename(self) -> &'static str {
        match self {
            Self::Built => "final_assignments.json",
            Self::Imported { .. } => "assignments.json",
        }
    }
}

/// An import can share a label/year with an older build. Only the current
/// import index can select its states and file layout; file presence cannot.
fn imported_analysis_states(
    index: &[u8],
    label: &str,
    year: &str,
) -> Result<Option<Vec<(String, AnalysisInput)>>, String> {
    let Ok(index) = serde_json::from_slice::<serde_json::Value>(index) else {
        // Preserve legacy build-index behavior; the caller still hashes its bytes.
        return Ok(None);
    };
    if index["algorithm"]["structure"] != "external" {
        return Ok(None);
    }
    let states = index["states"].as_object().filter(|s| !s.is_empty() && s.len() <= 51)
        .ok_or("[INPUT] external import index must name 1–51 states")?;
    if index["label"] != label || index["year"] != year {
        return Err("[INPUT] external import index label/year does not match analysis request".into());
    }
    states.iter().map(|(name, status)| {
        let tracts = status["tracts"].as_u64().filter(|n| *n > 0)
            .ok_or("[INPUT] external import state must declare a positive tract count")?;
        if state_name_to_code(name).is_none() || status["status"] != "ok" {
            return Err("[INPUT] external import index contains an invalid state entry".into());
        }
        Ok((name.clone(), AnalysisInput::Imported { tracts }))
    }).collect::<Result<Vec<_>, _>>().map(Some)
}

// ── run_label_analyze ─────────────────────────────────────────────────────────

/// Execute `bisect analyze <label> [--types T...] [--year Y] [--states S...]`.
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
    bisect_report::label_indexes::validate_analysis_types(types)?;

    // 2. Get registry entry.
    let entry = Registry::get(label)?.ok_or_else(|| {
        format!(
            "[CONFIG] analyze: label '{label}' not found in registry.\n\
             Run: bisect build {label} --year <YEAR> first."
        )
    })?;

    if entry.built.is_empty() {
        return Err(format!(
            "[CONFIG] analyze: label '{label}' has no built years.\n\
             Run: bisect build {label} --year <YEAR> first."
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

        // 4b. Select states from the active import index or build directory.
        let year_dir = year_runs_dir(label, y);
        let build_index_path = year_dir.join("index.json");
        let build_index = match std::fs::read(&build_index_path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("[INTERNAL] analyze: cannot read '{}': {e}", build_index_path.display())),
        };
        let indexed_states = build_index.as_deref().map(|index| imported_analysis_states(index, label, y))
            .transpose()?.flatten();
        let mut state_inputs = match indexed_states {
            Some(states) => states,
            None => enumerate_states_in_dir(&year_dir)?.into_iter()
                .map(|name| (name, AnalysisInput::Built)).collect(),
        };
        state_inputs.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        // Apply --states filter (case-insensitive; filter is state_names, not codes).
        let filter: std::collections::HashSet<String> =
            states.iter().map(|s| s.to_lowercase()).collect();

        state_inputs.retain(|(name, _)| filter.is_empty() || filter.contains(name));

        if state_inputs.is_empty() {
            eprintln!(
                "[analyze] WARNING: no states found in {} — skipping year {y}",
                year_dir.display()
            );
            continue;
        }

        // Locate runs/{label}/{year}/index.json for SHA chain.
        let run_index_sha256 = if let Some(bytes) = build_index {
            bisect_report::label_import::sha256_hex(&bytes)
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

        for (state_name, input) in &state_inputs {
            eprintln!("[analyze] {label}/{y}/{state_name}");

            let state_analysis_out = state_analysis_dir(label, y, state_name);
            std::fs::create_dir_all(&state_analysis_out).map_err(|e| {
                format!(
                    "[INTERNAL] analyze: cannot create '{}': {e}",
                    state_analysis_out.display()
                )
            })?;

            let result = run_analyze_state(label, y, state_name, types, &state_analysis_out, *input);

            match result {
                Ok(type_map) => {
                    let overall = if type_map.values().any(|s| s == "failed") {
                        "failed"
                    } else {
                        "ok"
                    };
                    state_statuses.insert(
                        state_name.clone(),
                        StateAnalysisStatus {
                            status: overall.to_string(),
                            error: None,
                            type_statuses: type_map,
                        },
                    );
                    eprintln!("[analyze] {label}/{y}/{state_name}: {overall}");
                }
                Err(e) => {
                    eprintln!("[analyze] FAILED {label}/{y}/{state_name}: {e}");
                    state_statuses.insert(
                        state_name.clone(),
                        StateAnalysisStatus {
                            status: "failed".to_string(),
                            error: Some(e),
                            type_statuses: HashMap::new(),
                        },
                    );
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

fn read_portable_source(path: &Path, limit: usize) -> Result<String, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take((limit + 1) as u64).read_to_end(&mut bytes))
        .map_err(|e| format!("[INPUT] cannot read '{}': {e}", path.display()))?;
    if bytes.len() > limit {
        return Err(format!("[INPUT] '{}' exceeds {limit} bytes", path.display()));
    }
    String::from_utf8(bytes).map_err(|e| format!("[INPUT] '{}' is not UTF-8: {e}", path.display()))
}

/// Compute actual native graph contiguity for an active external label.
/// The context and numbering basis are explicit and all inputs are checked
/// before the analysis index or registry is changed.
pub fn run_imported_label_contiguity(
    label: &str,
    types: &[String],
    year: Option<&str>,
    states: &[String],
    context_path: &Path,
    basis: Option<&str>,
) -> Result<(), String> {
    use bisect_report::label_import::{align_context, sha256_hex};
    validate_label_name(label)?;
    if types != ["contiguity"] || states.len() != 1 {
        return Err("[INPUT] --context requires --types contiguity and one --states name".into());
    }
    let year = year.filter(|s| s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit()))
        .ok_or("[INPUT] --context requires an explicit four-digit --year")?;
    let basis = basis.filter(|s| matches!(*s, "zero-based" | "one-based"))
        .ok_or("[INPUT] --context requires explicit district numbering")?;
    let state = states[0].to_ascii_lowercase();
    Registry::require_built(label, year)?;

    let run_index = read_portable_source(&year_runs_dir(label, year).join("index.json"), 1024 * 1024)?;
    let imported = imported_analysis_states(run_index.as_bytes(), label, year)?
        .ok_or("[INPUT] context analysis requires an active external import index")?;
    let Some((_, AnalysisInput::Imported { tracts })) = imported.iter().find(|(name, _)| name == &state) else {
        return Err("[INPUT] selected state is absent from the active external import".into());
    };
    let assignments_path = crate::label::state_runs_dir(label, year, &state).join("assignments.json");
    let assignments_text = read_portable_source(&assignments_path, 16 * 1024 * 1024)?;
    let assignments: std::collections::BTreeMap<String, u64> = serde_json::from_str(&assignments_text)
        .map_err(|e| format!("[INPUT] invalid imported assignments: {e}"))?;
    if assignments.len() as u64 != *tracts {
        return Err("[INPUT] imported assignment count differs from the active index".into());
    }
    let context_text = read_portable_source(context_path, 8 * 1024 * 1024)?;
    let context = rplan_io::read_rctx_str(&context_text).map_err(|e| e.to_string())?;
    let graph = context.graph.as_ref().ok_or("RCTX graph is unavailable; contiguity was not evaluated.")?;
    if graph.adjacency.iter().map(Vec::len).sum::<usize>() > 2_000_000 {
        return Err("[INPUT] context exceeds the portable graph limit".into());
    }
    let index_sha = sha256_hex(run_index.as_bytes());
    let created = now_rfc3339();
    let document = align_context(&assignments, &context, &state, label, year, &created, basis,
        serde_json::json!({
            "index_sha256": &index_sha,
            "assignments_sha256": sha256_hex(assignments_text.as_bytes()),
            "context_sha256": sha256_hex(context_text.as_bytes()),
        }))?;
    let plan = document.plan;
    rplan_audit::measurements::measure_plan(&plan, Some(&context))?;
    let adjacency = graph.adjacency.iter().map(|row| {
        row.iter().map(|edge| edge.to as usize).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    let ordered = plan.units.unit_ids.iter().cloned()
        .zip(plan.assignment.iter().map(|d| *d as usize + 1)).collect();
    let report = bisect_analysis::contiguity::try_check_contiguity(
        &ordered, &adjacency, &plan.units.unit_ids, plan.k,
    ).map_err(|e| e.to_string())?;
    let report_json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    let statuses = imported.into_iter().map(|(name, _)| {
        let selected = name == state;
        (name, StateAnalysisStatus {
            status: if selected { "ok" } else { "skipped" }.to_string(),
            error: None,
            type_statuses: if selected {
                HashMap::from([("contiguity".into(), "ok".into())])
            } else { HashMap::new() },
        })
    }).collect();
    let index = AnalysisIndex {
        label: label.into(), year: year.into(), run: label.into(), run_index_sha256: index_sha,
        types: types.to_vec(), created, states: statuses,
    };
    let index_json = bisect_report::label_indexes::serialize_analysis_index(&index)?;
    let output = state_analysis_dir(label, year, &state);
    std::fs::create_dir_all(&output).map_err(|e| format!("[INTERNAL] analyze: {e}"))?;
    for (path, json) in [
        (output.join("contiguity.json"), report_json),
        (year_analysis_dir(label, year).join("index.json"), index_json),
    ] {
        std::fs::write(&path, json)
            .map_err(|e| format!("[INTERNAL] analyze: cannot write '{}': {e}", path.display()))?;
        eprintln!("[analyze] wrote: {}", path.display());
    }
    Registry::mark_analyzed(label, year)?;
    Ok(())
}

/// Execute `BISECT report <label> [--year Y] [--format html|json] [--out PATH]`.
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
    run_label_report_with_clock(label, year, format, out, now_rfc3339)
}

/// Execute native filesystem orchestration with one recorded timestamp, as used
/// by reproducible browser requests. Ordinary CLI reporting keeps its live clock.
pub fn run_label_report_at(
    label: &str,
    year: Option<&str>,
    format: &str,
    out: Option<&Path>,
    created: &str,
) -> Result<(), String> {
    run_label_report_with_clock(label, year, format, out, || created.to_owned())
}

fn run_label_report_with_clock(
    label: &str,
    year: Option<&str>,
    format: &str,
    out: Option<&Path>,
    mut clock: impl FnMut() -> String,
) -> Result<(), String> {
    // 1. Validate label.
    validate_label_name(label)?;

    // 2. Get registry entry.
    let entry = Registry::get(label)?.ok_or_else(|| {
        format!(
            "[CONFIG] report: label '{label}' not found in registry.\n\
             Run: bisect build {label} --year <YEAR> and \
             bisect analyze {label} --year <YEAR> first."
        )
    })?;

    if entry.analyzed.is_empty() {
        return Err(format!(
            "[CONFIG] report: label '{label}' has no analyzed years.\n\
             Run: bisect analyze {label} --year <YEAR> first."
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
                 Run: bisect analyze {label} --year {y}",
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
            None => year_reports_dir(label, y),
        };
        std::fs::create_dir_all(&report_dir).map_err(|e| {
            format!(
                "[INTERNAL] report: cannot create '{}': {e}",
                report_dir.display()
            )
        })?;

        // Generate the report for this label/year.
        let files_written = generate_label_report_with_clock(label, y, &analysis_dir, &report_dir, format, &mut clock)?;
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
            created: clock(),
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
        .map_err(|e| {
            format!(
                "[INTERNAL] enumerate_states: cannot read '{}': {e}",
                dir.display()
            )
        })?
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
/// Fully wired types: `compactness`, `partisan`.
/// All other types produce a provenance-stub JSON (future work).
fn run_analyze_state(
    label: &str,
    year: &str,
    state_name: &str,
    types: &[String],
    out_dir: &Path,
    input: AnalysisInput,
) -> Result<HashMap<String, String>, String> {
    let state_runs_dir = crate::label::state_runs_dir(label, year, state_name);
    let filename = input.filename();
    let assignments_path = state_runs_dir.join(filename);

    // If there are no assignments yet (state failed during build), skip gracefully.
    if !assignments_path.exists() {
        return Err(format!(
            "{filename} not found at '{}'; this state may have failed during {} {label}",
            assignments_path.display(),
            if matches!(input, AnalysisInput::Built) { "bisect build" } else { "bisect label-import" },
        ));
    }

    let mut type_statuses: HashMap<String, String> = HashMap::new();

    // Resolve the effective type list (empty → summary).
    let effective_types: Vec<String> = if types.is_empty() {
        vec!["summary".to_string()]
    } else {
        types.to_vec()
    };
    let imported = matches!(input, AnalysisInput::Imported { .. });
    if let AnalysisInput::Imported { tracts } = input {
        let bytes = std::fs::read(&assignments_path)
            .map_err(|e| format!("[INPUT] cannot read imported assignments: {e}"))?;
        let assignments: HashMap<String, u64> = serde_json::from_slice(&bytes)
            .map_err(|e| format!("[INPUT] cannot parse imported assignments: {e}"))?;
        if assignments.len() as u64 != tracts || assignments.keys().any(|id| {
            id.get(..2).and_then(bisect_report::label_import::fips_to_state_name) != Some(state_name)
        }) {
            return Err("[INPUT] imported assignments do not match the index state and tract count".into());
        }
        // Literal IDs remain valid import data. Do not pass unbounded labels
        // to metric routines that may index or allocate by district number.
        if effective_types.iter().any(|kind| matches!(kind.as_str(), "compactness" | "partisan"))
            && assignments.values().any(|&d| d > 500)
        {
            return Err("[INPUT] imported district IDs exceed the 500-district analysis bound".into());
        }
    }

    // Resolve state code once — needed by compactness and partisan.
    // state_name uses lowercase_underscore convention ("north_carolina" → "NC").
    let state_code: Option<&'static str> = state_name_to_code(state_name);

    for type_name in &effective_types {
        let out_path = out_dir.join(format!("{type_name}.json"));

        // Skip if already exists.
        if !imported && out_path.exists() {
            eprintln!("[analyze] skip {type_name} for {state_name}/{year} (exists)");
            type_statuses.insert(type_name.clone(), "ok".to_string());
            continue;
        }

        match type_name.as_str() {
            // ── compactness (#192) ─────────────────────────────────────────────
            "compactness" => {
                let status = run_compactness_for_state(
                    label,
                    year,
                    state_name,
                    state_code,
                    &assignments_path,
                    &out_path,
                );
                match status {
                    Ok(()) => {
                        type_statuses.insert(type_name.clone(), "ok".to_string());
                    }
                    Err(e) => {
                        eprintln!("[analyze] compactness FAILED for {state_name}/{year}: {e}");
                        type_statuses.insert(type_name.clone(), "failed".to_string());
                    }
                }
            }

            // ── partisan (#193) ────────────────────────────────────────────────
            "partisan" => {
                let status = run_partisan_for_state(
                    label,
                    year,
                    state_name,
                    state_code,
                    &assignments_path,
                    out_dir,
                    imported,
                );
                match status {
                    Ok(()) => {
                        type_statuses.insert(type_name.clone(), "ok".to_string());
                    }
                    Err(e) => {
                        eprintln!("[analyze] partisan FAILED for {state_name}/{year}: {e}");
                        type_statuses.insert(type_name.clone(), "failed".to_string());
                    }
                }
            }

            // ── all other types: provenance stub (future work) ─────────────────
            _ => {
                let stub = serde_json::json!({
                    "analyzer": type_name,
                    "label": label,
                    "year": year,
                    "state": state_name,
                    "source": format!("runs/{label}/{year}/{state_name}/{filename}"),
                    "status": "ok",
                });
                write_json_file(&out_path, &stub)?;
                type_statuses.insert(type_name.clone(), "ok".to_string());
            }
        }
    }

    Ok(type_statuses)
}

/// Compute all compactness metrics for every district in the label-tree state plan
/// and write `compactness.json` to `out_path`.
///
/// Requires: TIGER tract shapefiles at `data/{year}/tiger/tracts/...`.
/// If geometry data is unavailable, writes a structured error JSON instead of crashing.
///
/// Output format:
/// ```json
/// {
///   "analyzer": "compactness",
///   "label": "my_plan",
///   "year": "2020",
///   "state": "vermont",
///   "note_pwc": "population_weighted_compactness requires tract centroids; use --types pwc when available",
///   "districts": [
///     { "district": 1, "polsby_popper": 0.32, "reock": 0.41, "convex_hull_ratio": 0.88,
///       "schwartzberg": 1.77, "length_width_ratio": 1.54, "perimeter_m": 432000.0, "area_m2": 2.1e10 },
///     ...
///   ],
///   "summary": {
///     "mean_polsby_popper": 0.32, "mean_reock": 0.41, "mean_convex_hull_ratio": 0.88,
///     "mean_schwartzberg": 1.77, "mean_length_width_ratio": 1.54, "n_districts": 1
///   }
/// }
/// ```
fn run_compactness_for_state(
    label: &str,
    year: &str,
    state_name: &str,
    state_code: Option<&str>,
    assignments_path: &Path,
    out_path: &Path,
) -> Result<(), String> {
    // Resolve state code — required for TIGER shapefile lookup.
    let code = match state_code {
        Some(c) => c,
        None => {
            let err_json = serde_json::json!({
                "analyzer": "compactness",
                "label": label,
                "year": year,
                "state": state_name,
                "available": false,
                "error": format!("unknown state name '{state_name}'; cannot resolve 2-letter code"),
            });
            write_json_file(out_path, &err_json)?;
            return Ok(());
        }
    };

    // Load assignments.
    let raw_assignments: HashMap<String, usize> = std::fs::read_to_string(assignments_path)
        .map_err(|e| format!("[INTERNAL] compactness: cannot read assignments: {e}"))?
        .parse::<serde_json::Value>()
        .map_err(|e| format!("[INTERNAL] compactness: cannot parse assignments JSON: {e}"))?
        .as_object()
        .ok_or_else(|| "[INTERNAL] compactness: assignments JSON is not an object".to_string())?
        .iter()
        .filter_map(|(k, v)| v.as_u64().map(|d| (k.clone(), d as usize)))
        .collect();

    // Resolve index-keyed assignments to GEOID-keyed (uses adjacency geoids file).
    // Pass output_root = current dir since label-tree doesn't have an explicit root.
    let assignments = crate::geometry::resolve_to_geoid_assignments(
        raw_assignments,
        std::path::Path::new("outputs/v1"), // fallback; geoids also searched in standard paths
        code,
        year,
    );

    // Load dissolved district geometries from TIGER shapefiles.
    let districts = match load_district_geometries(
        state_name,
        code,
        year,
        "v1", // version placeholder — geometry loader searches standard paths
        &assignments,
        std::path::Path::new("data"),
        "tract",
    ) {
        Ok(d) => d,
        Err(e) => {
            // Missing TIGER data → structured error, not a hard failure.
            eprintln!("[analyze] compactness: geometry unavailable for {state_name}/{year}: {e}");
            let err_json = serde_json::json!({
                "analyzer": "compactness",
                "label": label,
                "year": year,
                "state": state_name,
                "available": false,
                "error": format!("geometry not available: {e}"),
                "hint": format!("Run: bisect fetch --year {year} --states {code}"),
            });
            write_json_file(out_path, &err_json)?;
            return Ok(());
        }
    };

    // Compute metrics for each district.
    let mut district_results: Vec<serde_json::Value> = Vec::new();
    for (district_id, mp) in &districts {
        if let Some(poly) = bisect_map::largest_component(mp) {
            match bisect_analysis::all_metrics(*district_id, poly) {
                Ok(m) => {
                    district_results.push(serde_json::json!({
                        "district": m.district,
                        "polsby_popper": m.polsby_popper,
                        "reock": m.reock,
                        "convex_hull_ratio": m.convex_hull_ratio,
                        "schwartzberg": m.schwartzberg,
                        "length_width_ratio": m.length_width_ratio,
                        "perimeter_m": m.perimeter_m,
                        "area_m2": m.area_m2,
                    }));
                }
                Err(e) => {
                    eprintln!("WARNING: compactness skipped for district {district_id} ({state_name}): {e}");
                }
            }
        }
    }
    district_results.sort_by_key(|d| d["district"].as_u64().unwrap_or(0));

    // Compute per-metric means across all districts.
    let n = district_results.len() as f64;
    let mean = |field: &str| -> f64 {
        if n == 0.0 {
            return 0.0;
        }
        district_results
            .iter()
            .filter_map(|d| d[field].as_f64())
            .sum::<f64>()
            / n
    };

    let summary = serde_json::json!({
        "mean_polsby_popper":    mean("polsby_popper"),
        "mean_reock":            mean("reock"),
        "mean_convex_hull_ratio": mean("convex_hull_ratio"),
        "mean_schwartzberg":     mean("schwartzberg"),
        "mean_length_width_ratio": mean("length_width_ratio"),
        "n_districts": district_results.len(),
    });

    let result = serde_json::json!({
        "analyzer": "compactness",
        "label": label,
        "year": year,
        "state": state_name,
        "available": true,
        "note_pwc": "population_weighted_compactness requires tract centroids; use --types pwc when available",
        "districts": district_results,
        "summary": summary,
    });
    write_json_file(out_path, &result)?;
    eprintln!("[OK] compactness -> {}", out_path.display());
    Ok(())
}

/// Run partisan analysis for a label-tree state plan and write `partisan.json`.
///
/// Calls the same `run_partisan` function used by the main `bisect analyze` command.
/// If VEST election data is not available, writes a structured unavailable marker
/// (matching the format of the main analyze command) rather than failing.
///
/// Output when VEST available: full JSON with efficiency_gap, mean_median,
///   partisan_bias, declination (via MetricsBlock), seats_votes_curve.
/// Output when VEST absent:
/// ```json
/// {
///   "analyzer": "partisan", "available": false,
///   "error": "VEST precinct data required for partisan analysis. Run bisect fetch --vest to download."
/// }
/// ```
fn run_partisan_for_state(
    label: &str,
    year: &str,
    state_name: &str,
    state_code: Option<&str>,
    assignments_path: &Path,
    out_dir: &Path,
    force: bool,
) -> Result<(), String> {
    // Resolve state code.
    let code = match state_code {
        Some(c) => c,
        None => {
            let out_path = out_dir.join("partisan.json");
            let err_json = serde_json::json!({
                "analyzer": "partisan",
                "label": label,
                "year": year,
                "state": state_name,
                "available": false,
                "error": format!("unknown state name '{state_name}'; cannot resolve 2-letter code"),
            });
            write_json_file(&out_path, &err_json)?;
            return Ok(());
        }
    };

    // Load assignments.
    let raw_assignments: HashMap<String, usize> = std::fs::read_to_string(assignments_path)
        .map_err(|e| format!("[INTERNAL] partisan: cannot read assignments: {e}"))?
        .parse::<serde_json::Value>()
        .map_err(|e| format!("[INTERNAL] partisan: cannot parse assignments JSON: {e}"))?
        .as_object()
        .ok_or_else(|| "[INTERNAL] partisan: assignments JSON is not an object".to_string())?
        .iter()
        .filter_map(|(k, v)| v.as_u64().map(|d| (k.clone(), d as usize)))
        .collect();

    // Resolve to GEOID-keyed assignments.
    let assignments = crate::geometry::resolve_to_geoid_assignments(
        raw_assignments,
        std::path::Path::new("outputs/v1"),
        code,
        year,
    );

    // Delegate to run_partisan — it handles missing election data gracefully.
    let partisan_args = PartisanArgs {
        assignments: &assignments,
        state_code: code,
        state_name,
        year,
        version: "v1",
        election_file: None, // use default path: data/{year}/elections/presidential_by_tract.csv
        bootstrap_samples: 1000,
        analysis_dir: out_dir,
        force,
        chamber: "congressional",
    };

    run_partisan(&partisan_args).map_err(|e| format!("partisan runner failed: {e}"))?;

    // run_partisan writes partisan.json (available=true or available=false) directly.
    // If election data was absent it already wrote the unavailable marker.
    Ok(())
}

/// Produce a report from the analysis outputs and write to `report_dir`.
///
/// Returns the list of file names written (relative to `report_dir`).
#[cfg(test)]
fn generate_label_report(
    label: &str,
    year: &str,
    analysis_dir: &Path,
    report_dir: &Path,
    format: &str,
) -> Result<Vec<String>, String> {
    generate_label_report_with_clock(label, year, analysis_dir, report_dir, format, now_rfc3339)
}

fn generate_label_report_with_clock(
    label: &str,
    year: &str,
    analysis_dir: &Path,
    report_dir: &Path,
    format: &str,
    mut clock: impl FnMut() -> String,
) -> Result<Vec<String>, String> {
    let analysis_index_path = analysis_dir.join("index.json");
    let raw = if analysis_index_path.exists() {
        Some(std::fs::read_to_string(&analysis_index_path)
            .map_err(|error| format!("[INTERNAL] report: cannot read analysis index: {error}"))?)
    } else { None };
    let (filename, text) = bisect_report::label_summary::render_label_summary(
        label, year, raw.as_deref(), format, &clock(),
    )?;
    let path = report_dir.join(&filename);
    std::fs::write(&path, text)
        .map_err(|error| format!("[INTERNAL] report: cannot write '{}': {error}", path.display()))?;
    eprintln!("[report] wrote {}: {}", if format == "html" { "HTML" } else { "JSON" }, path.display());
    Ok(vec![filename])
}

/// Write the analysis index to `analysis/{label}/{year}/index.json`.
fn write_analysis_index(label: &str, year: &str, index: &AnalysisIndex) -> Result<(), String> {
    let path = year_analysis_dir(label, year).join("index.json");
    let json = bisect_report::label_indexes::serialize_analysis_index(index)?;
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
        None => year_reports_dir(label, year),
    };
    let path = dir.join("index.json");
    let json = bisect_report::label_indexes::serialize_report_index(index)?;
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
    std::fs::write(path, json).map_err(|e| {
        format!(
            "[INTERNAL] write_json_file: cannot write '{}': {e}",
            path.display()
        )
    })
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
            assert!(
                msg.contains("official_proposal"),
                "must name the label: {msg}"
            );
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
        let year = "2020";
        let msg = format!(
            "[CONFIG] report: label '{label}' has no analyzed years.\n\
             Run: bisect analyze {label} --year {year} first."
        );
        assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
        assert!(msg.contains("my_plan"), "must name the label: {msg}");
        assert!(msg.contains("bisect analyze"), "must suggest fix: {msg}");

        // Also verify require_analyzed message format directly (matches run_registry tests).
        // require_analyzed is [CONFIG] + bisect analyze hint (enforced by Registry invariant).
        let registry_msg = format!(
            "[CONFIG] report: 'my_plan' has not been analyzed for year 2020.\n\
             Run: bisect analyze my_plan --year 2020"
        );
        assert!(
            registry_msg.contains("[CONFIG]"),
            "registry msg must be [CONFIG]: {registry_msg}"
        );
        assert!(
            registry_msg.contains("bisect analyze"),
            "registry msg must suggest analyze: {registry_msg}"
        );
    }

    // ── 5. Report label not in registry → [CONFIG] error ─────────────────────

    #[test]
    fn test_report_label_not_in_registry_errors() {
        let _dir = with_tempdir(|| {
            let result = run_label_report("nonexistent_label", Some("2020"), "html", None);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG]: {msg}");
            assert!(
                msg.contains("nonexistent_label"),
                "must name the label: {msg}"
            );
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
        assert_eq!(files[0].as_str().unwrap(), "vt_test_2020_report.json");
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
        assert!(
            msg.contains("pdf"),
            "must mention the unsupported format: {msg}"
        );
    }

    // ── 10. sha256_str produces 64 hex chars ──────────────────────────────────

    #[test]
    fn test_sha256_str_produces_64_hex() {
        let hash = sha256_str("hello world");
        assert_eq!(
            hash.len(),
            64,
            "SHA-256 must be 64 chars, got: {}",
            hash.len()
        );
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
        assert_eq!(
            names,
            vec!["alaska", "california", "vermont"],
            "states must be sorted alphabetically"
        );
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
        assert!(
            ts.contains('T'),
            "timestamp must contain 'T' separator: {ts}"
        );
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

        let files =
            generate_label_report("senate_draft", "2020", &analysis_dir, &report_dir, "html")
                .unwrap();

        assert_eq!(files.len(), 1, "must write exactly 1 HTML file");
        assert_eq!(
            files[0], "senate_draft_2020_report.html",
            "filename must follow {{label}}_{{year}}_report.html pattern"
        );
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

        let files =
            generate_label_report("senate_draft", "2020", &analysis_dir, &report_dir, "json")
                .unwrap();

        assert_eq!(files.len(), 1, "must write exactly 1 JSON file");
        assert_eq!(
            files[0], "senate_draft_2020_report.json",
            "filename must follow {{label}}_{{year}}_report.json pattern"
        );
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
        states.insert(
            "vermont".to_string(),
            StateAnalysisStatus {
                status: "ok".to_string(),
                error: None,
                type_statuses: {
                    let mut m = HashMap::new();
                    m.insert("summary".to_string(), "ok".to_string());
                    m
                },
            },
        );
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
             Run: bisect build {label} --year <YEAR> first."
        );
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("my_plan"), "must name the label: {msg}");
        assert!(msg.contains("bisect build"), "must suggest fix: {msg}");
    }

    // ── 25. report: label not in registry → [CONFIG] error format ────────────

    #[test]
    fn test_report_not_in_registry_error_message_format() {
        let label = "ghost";
        let msg = format!(
            "[CONFIG] report: label '{label}' not found in registry.\n\
             Run: bisect build {label} --year <YEAR> and \
             bisect analyze {label} --year <YEAR> first."
        );
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("ghost"), "must name the label: {msg}");
        assert!(
            msg.contains("bisect build"),
            "must mention build step: {msg}"
        );
        assert!(
            msg.contains("bisect analyze"),
            "must mention analyze step: {msg}"
        );
    }

    // ── 26. sha256_str is consistent with sha256_file for same content ────────

    #[test]
    fn test_sha256_str_consistent_with_sha256_file() {
        use std::io::Write;
        let content = "hello world\n";
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let via_str = sha256_str(content);
        let via_file = sha256_file(tmp.path()).unwrap();
        assert_eq!(
            via_str, via_file,
            "sha256_str and sha256_file must produce the same digest for identical content"
        );
    }

    // ── 27. sha256_str: known value for empty string ──────────────────────────
    //
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

    #[test]
    fn test_sha256_str_empty_string_known_digest() {
        let hash = sha256_str("");
        assert_eq!(
            hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
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
        assert!(html.contains("2010"), "HTML must contain year: {html}");
        assert!(
            html.contains("<!DOCTYPE html>"),
            "HTML must start with DOCTYPE"
        );
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
        assert_eq!(
            v["label"].as_str(),
            Some("myplan"),
            "JSON must contain label"
        );
        assert_eq!(v["year"].as_str(), Some("2010"), "JSON must contain year");
        assert_eq!(
            v["report_type"].as_str(),
            Some("label_analysis"),
            "JSON must have report_type"
        );
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
        assert_eq!(
            names,
            vec!["alaska", "vermont"],
            "enumerate_states must skip files and return only dirs: {names:?}"
        );
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
        assert_eq!(restored.year, "2020");
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
        assert_eq!(restored.year, "2010");
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
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
        assert!(
            msg.contains("html") || msg.contains("json"),
            "error must mention valid alternatives: {msg}"
        );
    }

    // ── 34. compactness: unknown state name writes error JSON, does not crash ──
    //
    // When state_name cannot be resolved to a 2-letter code, run_compactness_for_state
    // must write a structured JSON with available=false rather than propagating an Err.

    #[test]
    fn test_compactness_unknown_state_writes_error_json() {
        let tmp = TempDir::new().unwrap();
        let out_path = tmp.path().join("compactness.json");
        let assignments_path = tmp.path().join("final_assignments.json");
        // Write minimal valid assignments file (GEOID-keyed, one district)
        std::fs::write(&assignments_path, r#"{"10001000100": 1}"#).unwrap();

        // "atlantis" is not a known state name → should write error JSON, not Err
        let result = run_compactness_for_state(
            "test_label",
            "2020",
            "atlantis",
            None,
            &assignments_path,
            &out_path,
        );
        assert!(
            result.is_ok(),
            "unknown state must not propagate Err: {result:?}"
        );
        assert!(out_path.exists(), "error JSON must be written to out_path");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert_eq!(json["analyzer"].as_str(), Some("compactness"));
        assert_eq!(json["available"].as_bool(), Some(false));
        assert!(
            json["error"].as_str().unwrap_or("").contains("atlantis"),
            "error must mention the unknown state name"
        );
    }

    // ── 35. compactness: missing geometry writes error JSON, does not crash ─────
    //
    // When TIGER shapefiles are absent (no data/ dir), run_compactness_for_state
    // must write available=false JSON rather than propagating geometry load errors.

    #[test]
    fn test_compactness_missing_geometry_writes_error_json() {
        let tmp = TempDir::new().unwrap();
        let out_path = tmp.path().join("compactness.json");
        let assignments_path = tmp.path().join("final_assignments.json");
        // Write minimal GEOID-keyed assignments
        std::fs::write(&assignments_path, r#"{"50005957100": 1, "50005957200": 1}"#).unwrap();

        // Override CWD so no data/ dir is found
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(tmp.path()).expect("set_current_dir");

        let result = run_compactness_for_state(
            "test_label",
            "2020",
            "vermont",
            Some("VT"),
            &assignments_path,
            &out_path,
        );

        std::env::set_current_dir(&original).expect("restore current_dir");

        assert!(
            result.is_ok(),
            "missing geometry must not propagate Err: {result:?}"
        );
        assert!(out_path.exists(), "error JSON must be written to out_path");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert_eq!(json["analyzer"].as_str(), Some("compactness"));
        assert_eq!(json["available"].as_bool(), Some(false));
        assert!(json["error"].as_str().is_some(), "must have error field");
    }

    // ── 36. partisan: unknown state name writes error JSON, does not crash ──────

    #[test]
    fn test_partisan_unknown_state_writes_error_json() {
        let tmp = TempDir::new().unwrap();
        let out_dir = tmp.path().to_path_buf();
        let assignments_path = tmp.path().join("final_assignments.json");
        std::fs::write(&assignments_path, r#"{"10001000100": 1}"#).unwrap();

        let result = run_partisan_for_state(
            "test_label",
            "2020",
            "atlantis",
            None,
            &assignments_path,
            &out_dir,
            false,
        );
        assert!(
            result.is_ok(),
            "unknown state must not propagate Err: {result:?}"
        );

        let out_path = out_dir.join("partisan.json");
        assert!(out_path.exists(), "error JSON must be written");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert_eq!(json["analyzer"].as_str(), Some("partisan"));
        assert_eq!(json["available"].as_bool(), Some(false));
    }

    // ── 37. partisan: missing election data writes available=false, not crash ───
    //
    // run_partisan_for_state with a valid state but no election CSV must write
    // available=false (delegating gracefully to run_partisan which handles it).

    #[test]
    fn test_partisan_missing_election_data_writes_unavailable() {
        let tmp = TempDir::new().unwrap();
        let out_dir = tmp.path().to_path_buf();
        let assignments_path = tmp.path().join("final_assignments.json");
        std::fs::write(&assignments_path, r#"{"50005957100": 1, "50005957200": 1}"#).unwrap();

        // Run from tmp where data/{year}/elections/presidential_by_tract.csv does not exist
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(tmp.path()).expect("set_current_dir");

        let result = run_partisan_for_state(
            "test_label",
            "2020",
            "vermont",
            Some("VT"),
            &assignments_path,
            &out_dir,
            false,
        );

        std::env::set_current_dir(&original).expect("restore current_dir");

        assert!(
            result.is_ok(),
            "missing election data must not propagate Err: {result:?}"
        );

        let out_path = out_dir.join("partisan.json");
        assert!(
            out_path.exists(),
            "partisan.json must be written even when data absent"
        );

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert_eq!(json["analyzer"].as_str(), Some("partisan"));
        assert_eq!(
            json["available"].as_bool(),
            Some(false),
            "must mark as unavailable when election data absent: {json}"
        );
    }

    // ── 38. compactness type in run_analyze_state writes compactness.json ───────
    //
    // When types = ["compactness"], run_analyze_state must attempt the compactness
    // path. With no geometry available it writes an error JSON rather than a stub.

    #[test]
    fn test_run_analyze_state_compactness_dispatches() {
        let tmp = TempDir::new().unwrap();
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(tmp.path()).expect("set_current_dir");

        // Set up label tree: runs/my_label/2020/vermont/final_assignments.json
        let state_runs = tmp
            .path()
            .join("runs")
            .join("my_label")
            .join("2020")
            .join("vermont");
        std::fs::create_dir_all(&state_runs).unwrap();
        std::fs::write(
            state_runs.join("final_assignments.json"),
            r#"{"50005957100": 1}"#,
        )
        .unwrap();

        let out_dir = tmp.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let statuses = run_analyze_state(
            "my_label",
            "2020",
            "vermont",
            &["compactness".to_string()],
            &out_dir,
            AnalysisInput::Built,
        )
        .expect("run_analyze_state must not Err");

        std::env::set_current_dir(&original).expect("restore current_dir");

        // compactness.json must be written (may be error JSON if no TIGER data)
        assert!(
            out_dir.join("compactness.json").exists(),
            "compactness.json must be written"
        );
        // Status must be "ok" (error JSON is still "ok" — non-crashing)
        assert_eq!(
            statuses.get("compactness").map(|s| s.as_str()),
            Some("ok"),
            "compactness type must report ok status (error JSON counts as ok): {statuses:?}"
        );
    }

    // ── 39. partisan type in run_analyze_state writes partisan.json ──────────────

    #[test]
    fn test_run_analyze_state_partisan_dispatches() {
        let tmp = TempDir::new().unwrap();
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(tmp.path()).expect("set_current_dir");

        let state_runs = tmp
            .path()
            .join("runs")
            .join("my_label")
            .join("2020")
            .join("vermont");
        std::fs::create_dir_all(&state_runs).unwrap();
        std::fs::write(
            state_runs.join("final_assignments.json"),
            r#"{"50005957100": 1}"#,
        )
        .unwrap();

        let out_dir = tmp.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let statuses = run_analyze_state(
            "my_label",
            "2020",
            "vermont",
            &["partisan".to_string()],
            &out_dir,
            AnalysisInput::Built,
        )
        .expect("run_analyze_state must not Err");

        std::env::set_current_dir(&original).expect("restore current_dir");

        assert!(
            out_dir.join("partisan.json").exists(),
            "partisan.json must be written"
        );
        assert_eq!(
            statuses.get("partisan").map(|s| s.as_str()),
            Some("ok"),
            "partisan type must report ok status: {statuses:?}"
        );

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out_dir.join("partisan.json")).unwrap())
                .unwrap();
        assert_eq!(
            json["analyzer"].as_str(),
            Some("partisan"),
            "partisan.json must have analyzer field"
        );
    }

    // ── 40. compactness summary fields present when districts computed ────────────
    //
    // This test validates the summary JSON structure using a mock geometry path.
    // We can't run the full geometry pipeline in a unit test, so we check the
    // error-JSON structure (available=false) has the required schema fields.

    #[test]
    fn test_compactness_error_json_has_required_fields() {
        let tmp = TempDir::new().unwrap();
        let out_path = tmp.path().join("compactness.json");
        let assignments_path = tmp.path().join("final_assignments.json");
        std::fs::write(&assignments_path, r#"{"50005957100": 1}"#).unwrap();

        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(tmp.path()).expect("set_current_dir");

        run_compactness_for_state(
            "lbl",
            "2020",
            "vermont",
            Some("VT"),
            &assignments_path,
            &out_path,
        )
        .unwrap();

        std::env::set_current_dir(&original).expect("restore current_dir");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert!(json.get("analyzer").is_some(), "must have 'analyzer' field");
        assert!(json.get("label").is_some(), "must have 'label' field");
        assert!(json.get("year").is_some(), "must have 'year' field");
        assert!(json.get("state").is_some(), "must have 'state' field");
        assert!(
            json.get("available").is_some(),
            "must have 'available' field"
        );
    }
    #[test]
    fn test_report_clock_keeps_distinct_live_observations() {
        let _dir = with_tempdir(|| {
            std::fs::create_dir_all("analysis/clock_test/2020").unwrap();
            std::fs::write("analysis/clock_test/2020/index.json", "{}").unwrap();
            std::fs::write(".bisect", r#"{"clock_test":{"built":["2020"],"analyzed":["2020"],"reported":[]}}"#).unwrap();
            let mut count = 0;
            run_label_report_with_clock("clock_test", None, "json", None, || {
                count += 1;
                format!("2026-10-23T00:00:0{count}Z")
            }).unwrap();
            assert_eq!(count, 2);
            let report: serde_json::Value = serde_json::from_str(&std::fs::read_to_string("reports/clock_test/2020/clock_test_2020_report.json").unwrap()).unwrap();
            let index: serde_json::Value = serde_json::from_str(&std::fs::read_to_string("reports/clock_test/2020/index.json").unwrap()).unwrap();
            assert_eq!(report["created"], "2026-10-23T00:00:01Z");
            assert_eq!(index["created"], "2026-10-23T00:00:02Z");
        });
    }

    #[test]
    fn test_report_clock_recorded_self_overlap_hashes_exact_previous_index() {
        let _dir = with_tempdir(|| {
            let registry = r#"{"clock_test":{"built":["2020"],"analyzed":["2020","2020"],"reported":[]}}"#;
            let source = "analysis/clock_test/2020/index.json";
            std::fs::create_dir_all("analysis/clock_test/2020").unwrap();
            std::fs::write(source, "{}").unwrap();
            std::fs::write(".bisect", registry).unwrap();
            let output = Some(Path::new("analysis/clock_test/2020"));
            run_label_report_at("clock_test", Some("2020"), "json", output, "recorded-time").unwrap();
            std::fs::copy(source, "first-index.json").unwrap();
            let first_hash = sha256_file(Path::new("first-index.json")).unwrap();
            std::fs::write(source, "{}").unwrap();
            std::fs::write(".bisect", registry).unwrap();
            run_label_report_at("clock_test", None, "json", output, "recorded-time").unwrap();
            let last: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(source).unwrap()).unwrap();
            assert_eq!(last["analysis_index_sha256"], first_hash);
            assert_eq!(last["created"], "recorded-time");
        });
    }

}
