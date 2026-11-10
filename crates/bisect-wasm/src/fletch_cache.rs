use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub selection: crate::fetch_plan::Input,
    pub cache_manifest_json: String,
}
pub(crate) fn merge(base: String, updates: String) -> Result<Value, String> {
    if base.len() > 16 * 1024 * 1024 || updates.len() > 16 * 1024 * 1024 {
        return Err("FLETCH merge input exceeds 16 MiB per source".into());
    }
    // Bound the native linear key searches before performing any merge.
    let base_entries: bisect_core::fletch_cache::Manifest = serde_json::from_str(&base).map_err(|e|e.to_string())?;
    let updates_entries: Vec<bisect_core::fletch_cache::ManifestEntry> = serde_json::from_str(&updates).map_err(|e|e.to_string())?;
    if base_entries.entries.len() > 2000 || updates_entries.len() > 2000 {
        return Err("FLETCH merge supports up to 2000 entries per input".into());
    }
    drop((base_entries, updates_entries));
    let manifest = bisect_core::fletch_cache::merge_cache_manifest_text(&base, &updates)?;
    if manifest.entries.len() > 2000 {
        return Err("FLETCH merged manifest exceeds 2000 entries".into());
    }
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    if manifest_json.len() > 16 * 1024 * 1024 {
        return Err("FLETCH merged manifest exceeds 16 MiB".into());
    }
    Ok(json!({"schema_version":"bisect-fletch-cache-merge-v1",
        "manifest_json":manifest_json,"entry_count":manifest.entries.len(),
        "base_sha256":format!("{:x}",Sha256::digest(base.as_bytes())),
        "updates_sha256":format!("{:x}",Sha256::digest(updates.as_bytes())),
        "scope":"Native FLETCH cache-key metadata merge only. Saved verification flags remain claims. No object is read, acquired, verified, installed or activated. Manifest paths and URLs remain inert text; exact u64 values are preserved in manifest_json."}))
}
pub(crate) fn inspect(input: Input) -> Result<Value, String> {
    if input.cache_manifest_json.len() > 8 * 1024 * 1024 {
        return Err("FLETCH cache manifest exceeds 8 MiB".into());
    }
    let plan = crate::fetch_plan::plan(input.selection)?;
    let mut expected = BTreeSet::new();
    for row in plan["items"].as_array().ok_or("Missing Fetch items")? {
        if row["url"].is_string() {
            expected.insert(format!(
                "bisect.{}.{}.{}",
                row["year"].as_str().ok_or("Missing year")?,
                row["state_code"]
                    .as_str()
                    .ok_or("Missing state")?
                    .to_lowercase(),
                row["kind"].as_str().ok_or("Missing type")?
            ));
        }
    }
    let report =
        bisect_core::fletch_cache::report_from_manifest(&input.cache_manifest_json, &expected)?;
    let passed = report.unverified_index_count == 0 && report.unexpected_index_count == 0;
    // Native JSON remains text: converting u64 report values to JavaScript
    // numbers would silently round valid manifest evidence.
    let report_json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    Ok(
        json!({"schema_version":"bisect-fletch-cache-inspection-v1", "report_json":report_json,
        "byte_count_text":report.byte_count.to_string(), "gate_passed":passed,
        "manifest_sha256":format!("{:x}",Sha256::digest(input.cache_manifest_json.as_bytes())),
        "source_plan":plan, "execution_performed":false,
        "scope":"Selected manifest evidence only. Saved verification flags are not fresh hash checks. No cache root or relative path is opened, no URL is requested, and no files are installed or activated. Missing expected sources are permitted by the native cache-index gate."}),
    )
}
