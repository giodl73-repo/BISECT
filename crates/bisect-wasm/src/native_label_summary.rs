//! Native label/year summary preparation; filesystem application is separate.
use std::collections::BTreeMap;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use bisect_report::label_indexes::{ReportIndex, serialize_report_index};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub label: String,
    pub year: String,
    pub registry_json: String,
    pub analysis_directory_present: bool,
    pub analysis_index_json: Option<String>,
    pub generated_at: String,
    pub format: String,
}

fn sha(text: &str) -> String { format!("{:x}", Sha256::digest(text.as_bytes())) }

pub(crate) fn run(input: Input) -> Result<Value, String> {
    bisect_report::label_rename::validate_name(&input.label)?;
    if input.registry_json.len() > 1024*1024 || input.analysis_index_json.as_ref().is_some_and(|text|text.len()>1024*1024) {
        return Err("Select registry and analysis-index sources up to 1 MiB each.".into());
    }
    let registry: BTreeMap<String, bisect_report::label_registry::LabelEntry> = bisect_report::label_registry::parse(&input.registry_json)?;
    let label = &input.label;
    let year = &input.year;
    let entry = registry.get(label).ok_or_else(|| format!("[CONFIG] report: label '{label}' not found in registry.\nRun: bisect build {label} --year <YEAR> and bisect analyze {label} --year <YEAR> first."))?;
    if entry.analyzed.is_empty() {
        return Err(format!("[CONFIG] report: label '{label}' has no analyzed years.\nRun: bisect analyze {label} --year <YEAR> first."));
    }
    if !entry.analyzed.contains(year) {
        return Err(format!("[CONFIG] report: '{label}' has not been analyzed for year {year}.\nRun: bisect analyze {label} --year {year}"));
    }
    if !input.analysis_directory_present {
        return Err(format!("[CONFIG] report: analysis directory not found: 'analysis/{label}/{year}'\nRun: bisect analyze {label} --year {year}"));
    }
    let (filename, text) = bisect_report::label_summary::render_label_summary(label, year, input.analysis_index_json.as_deref(), &input.format, &input.generated_at)?;
    let upstream = input.analysis_index_json.as_deref().map(sha).unwrap_or_default();
    let report_index = ReportIndex {label:label.clone(), year:year.clone(), analysis_index_sha256:upstream.clone(), format:input.format.clone(), created:input.generated_at.clone(), files:vec![filename.clone()]};
    let index_json = serialize_report_index(&report_index)?;
    let promoted = crate::label_stage::promote(Some(&input.registry_json), label, year, true)?.ok_or("Registry output is unavailable.")?;
    let root = format!("reports/{label}/{year}/");
    let index_path = root.clone()+"index.json";
    let files = BTreeMap::from([(root+&filename, text), (index_path.clone(), index_json.clone()), (".bisect".into(), promoted.clone())]);
    let hashes: BTreeMap<_,_> = files.iter().map(|(path,text)|(path.clone(),sha(text))).collect();
    Ok(json!({"schema_version":"bisect-label-report-v1", "report_variant":"native-summary", "label":label, "year":year, "status":"generated", "states":[], "files":files, "file_sha256":hashes, "index_path":index_path, "index_json":index_json, "index_sha256":sha(&index_json), "analysis_index_present":input.analysis_index_json.is_some(), "analysis_index_sha256":upstream, "registry_promoted":true, "registry_json":promoted, "registry_source_sha256":sha(&input.registry_json), "scope":"Native label/year summary from the raw analysis index, including missing-index fallback. Completion does not imply successful state analysis or source authenticity. Supplied directory presence is a selected-snapshot claim, never filesystem authority. Safe bounded default-layout package only; no files are written, no OS-wide discovery or output override is implied. Shared HTML text is escaped; imported content still requires a restricted script-free preview."}))
}
