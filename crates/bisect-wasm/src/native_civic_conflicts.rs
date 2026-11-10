use base64::{engine::general_purpose::STANDARD, Engine};
use bisect_analysis::civic::{detect_conflicts, parse_and_canonicalize_csv};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const MAX_INPUTS: usize = 20;
const MAX_BYTES: usize = 3 * 1024 * 1024;
const MAX_ROWS: usize = 50_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NormalizedInput {
    pub label: String,
    pub source_name: String,
    pub csv_base64: String,
}

pub(crate) fn inspect(inputs: Vec<NormalizedInput>) -> Result<Value, String> {
    if !(2..=MAX_INPUTS).contains(&inputs.len()) {
        return Err("Select 2–20 normalized civic CSV inputs.".into());
    }
    let mut total_bytes = 0;
    let mut total_rows = 0;
    let mut labels = BTreeMap::new();
    let mut sources = Vec::with_capacity(inputs.len());
    let mut parsed = Vec::with_capacity(inputs.len());
    for input in &inputs {
        if input.label.is_empty()
            || input.label.len() > 100
            || input.label.chars().any(char::is_control)
            || input.source_name.is_empty()
            || input.source_name.len() > 200
            || input.source_name.chars().any(char::is_control)
        {
            return Err("Use a nonempty desktop label (100 bytes) and source name (200 bytes) without control characters.".into());
        }
        if input.csv_base64.len() > 4 * MAX_BYTES / 3 {
            return Err("Normalized civic inputs total at most 3 MiB.".into());
        }
        let raw = STANDARD
            .decode(&input.csv_base64)
            .map_err(|_| "Invalid normalized civic CSV base64.")?;
        total_bytes += raw.len();
        if raw.is_empty() || total_bytes > MAX_BYTES {
            return Err("Select nonempty normalized civic inputs totaling at most 3 MiB.".into());
        }
        let hash = Sha256::digest(&raw);
        if let Some(prior) = labels.get(&input.label) {
            if prior != &raw {
                return Err("Repeated desktop labels must use identical normalized CSV bytes.".into());
            }
        }
        let (rows, _, _, _) = parse_and_canonicalize_csv(&raw)
            .map_err(|err| format!("Civic input '{}': {err}", input.label))?;
        total_rows += rows.len();
        if total_rows > MAX_ROWS {
            return Err("Normalized civic inputs are limited to 50,000 rows in total.".into());
        }
        sources.push(json!({
            "label": input.label,
            "source_name": input.source_name,
            "sha256": format!("sha256:{hash:x}"),
            "bytes": raw.len(),
            "row_count": rows.len(),
        }));
        labels.entry(input.label.clone()).or_insert(raw);
        parsed.push((input.label.clone(), rows));
    }
    let conflicts = detect_conflicts(&parsed);
    let conflicts_json = serde_json::to_string_pretty(&conflicts).map_err(|err| err.to_string())?;
    Ok(json!({
        "schema_version": "bisect-native-civic-conflicts-v1",
        "status": "compared",
        "sources": sources,
        "conflicts": conflicts,
        "conflicts_json": conflicts_json,
        "scope": "Native Conflicts parser and overlap report on explicitly selected normalized CSV bytes. Input order and repeated identical labels are retained. No ingestion validation, file discovery, network access, candidate-race analysis, or package authentication."
    }))
}
