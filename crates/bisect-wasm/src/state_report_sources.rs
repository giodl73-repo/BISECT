//! Native report-source discovery from complete, explicitly supplied observations.
//! This module cannot read a path or confer filesystem authority.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

const FILE_LIMIT: usize = 8 * 1024 * 1024;
const TOTAL_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Kind { Missing, File, Directory }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Observation {
    pub relative_path: String,
    pub kind: Kind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_base64: Option<String>,
}

pub(crate) fn inspect(
    arguments: Vec<String>, observations: Option<Vec<Observation>>,
    unit_ids: Option<Vec<String>>, observation_origin: Option<String>,
) -> Result<Value, String> {
    let identity = crate::state_identity::inspect(arguments.clone())?;
    let get = |key: &str| identity[key].as_str().ok_or_else(|| format!("Missing native {key}"));
    if identity["native_year_valid"] != true { return Err("Resolve a valid native State/year before report-source preparation.".into()); }
    let name = get("effective_state_name")?;
    let state = get("state_code")?;
    let year = get("effective_year")?;
    if !crate::run_target::portable_state_component(name)
        || !crate::run_target::portable_state_component(state)
        || year.len() != 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Report-source snapshots require a portable native name/state and four-digit year; no path was read.".into());
    }
    let command = bisect_runner::state_command::inspect(arguments)?;
    let requested = command["fields"].as_array().and_then(|rows| rows.iter().find(|r| r["id"] == "population_source"))
        .and_then(|r| r["values"][0].as_str()).ok_or("Missing native population-source request")?;
    let prefix = format!("data/{year}/demographics/");
    let availability = format!("{prefix}{}", bisect_core::native_input_paths::cvap_availability_basename(name, year));
    let candidates = bisect_core::native_input_paths::report_demographic_basenames(name, state, year)
        .map(|basename| format!("{prefix}{basename}"));
    let mut paths = vec![availability.clone()];
    for path in &candidates { if !paths.contains(path) { paths.push(path.clone()); } }
    let mut result = json!({
        "schema_version":"bisect-native-state-report-sources-v1", "status":"selection-required",
        "identity":identity, "requested_population_source":requested,
        "availability_path":availability, "report_candidates":candidates, "required_observation_paths":paths,
        "observed":false, "filesystem_reads_performed":false, "execution_performed":false,
        "source_discovery_authorized":false, "path_authority_granted":false,
        "balance_population_source":"selected-graph-populations",
        "scope":"Native candidate order and compatibility CSV reader over supplied source snapshots. Report counts never select balancing weights. Missing observations are not native absence. Paths and source-origin labels grant no filesystem authority; source authenticity, graph binding and execution remain separate."
    });
    let Some(observations) = observations else {
        if unit_ids.is_some() || observation_origin.is_some() { return Err("Supply a complete explicit source snapshot before preparing report counts.".into()); }
        return Ok(result);
    };
    let origin = observation_origin.ok_or("Label the explicit source observation realm.")?;
    if !["selected-folder-snapshot", "explicit-file-bundle"].contains(&origin.as_str()) {
        return Err("Unknown report-source observation realm.".into());
    }
    if observations.len() != paths.len() { return Err("Observe every distinct requested native candidate, including missing paths.".into()); }
    let mut entries = BTreeMap::new();
    let mut total = 0usize;
    let mut source_metadata = Vec::new();
    for entry in observations {
        if !paths.contains(&entry.relative_path) || entries.contains_key(&entry.relative_path) {
            return Err("Unexpected or duplicate native report-source path.".into());
        }
        let bytes = match (&entry.kind, &entry.source_base64) {
            (Kind::File, Some(encoded)) => {
                if encoded.len() > FILE_LIMIT.div_ceil(3) * 4 { return Err("A native report CSV exceeds 8 MiB.".into()); }
                let bytes = STANDARD.decode(encoded).map_err(|_| "Invalid native report-source base64")?;
                total += bytes.len();
                if bytes.len() > FILE_LIMIT || total > TOTAL_LIMIT { return Err("Native report-source snapshot exceeds its 8 MiB/file or 16 MiB total limit.".into()); }
                Some(bytes)
            }
            (Kind::Missing | Kind::Directory, None) => None,
            _ => return Err("Files require original bytes; missing paths and directories cannot carry file bytes.".into()),
        };
        source_metadata.push(json!({"relative_path":entry.relative_path,"kind":entry.kind,
            "byte_length":bytes.as_ref().map(Vec::len),
            "sha256":bytes.as_ref().map(|b| format!("{:x}", Sha256::digest(b)))}));
        entries.insert(entry.relative_path.clone(), (entry, bytes));
    }
    let present = entries[&availability].0.kind != Kind::Missing;
    let warning = requested == "cvap" && !present;
    let selected = candidates.iter().find(|path| entries[*path].0.kind != Kind::Missing);
    let bundle = json!({"schema_version":"bisect-native-report-source-bundle-v1", "state":state,
        "state_name":name, "year":year, "requested_population_source":requested,
        "observation_origin":origin,
        "observations":paths.iter().map(|path| &entries[path].0).collect::<Vec<_>>()});
    result["status"] = json!("sources-observed");
    result["observed"] = json!(true);
    result["observation_origin"] = json!(origin);
    result["source_metadata"] = json!(source_metadata);
    result["source_bundle_sha256"] = json!(format!("{:x}", Sha256::digest(serde_json::to_vec(&bundle).map_err(|e| e.to_string())?)));
    result["source_bundle"] = bundle;
    result["cvap_presence_observed"] = json!(present);
    result["cvap_missing_warning"] = json!(warning);
    result["availability_result_unused_by_partition"] = json!(if warning { "total" } else { requested });
    result["selected_report_path"] = json!(selected);
    let Some(ids) = unit_ids else { return Ok(result); };
    let unique: HashSet<_> = ids.iter().collect();
    if ids.is_empty() || ids.len() > 100_000 || unique.len() != ids.len()
        || ids.iter().any(|id| id.is_empty() || id.len() > 200 || id.chars().any(char::is_control)) {
        return Err("Supply 1–100,000 unique ordered unit IDs of at most 200 bytes.".into());
    }
    let mut missing = 0;
    let demographics = if let Some(path) = selected {
        let (entry, bytes) = &entries[path];
        if entry.kind == Kind::Directory {
            result["status"] = json!("native-load-failure");
            result["report_load_failure_kind"] = json!("directory");
            result["report_load_error"] = json!("The first existing native report candidate is a directory; no lower-priority source is tried.");
            return Ok(result);
        }
        let bytes = bytes.as_ref().ok_or("Missing selected CSV bytes")?;
        let text = match std::str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                result["status"] = json!("native-load-failure");
                result["report_load_failure_kind"] = json!("utf8");
                result["report_load_error"] = json!("The first existing native report CSV is not UTF-8; no lower-priority source is tried.");
                return Ok(result);
            }
        };
        if text.lines().skip(1).take(100_001).count() > 100_000 { return Err("Native report CSV exceeds 100,000 rows.".into()); }
        let records = match bisect_data::native_demographics::parse_native_vap_demographics(text) {
            Ok(records) => records,
            Err(error) => {
                result["status"] = json!("native-load-failure");
                result["report_load_failure_kind"] = json!("csv");
                result["report_load_error"] = json!(error.chars().take(2000).collect::<String>());
                return Ok(result);
            }
        };
        let mut total_vap = Vec::with_capacity(ids.len());
        let mut minority_vap = Vec::with_capacity(ids.len());
        for id in &ids {
            if let Some(record) = records.get(id) {
                total_vap.push(record.total_vap);
                minority_vap.push(record.minority_vap);
            } else {
                missing += 1;
                total_vap.push(0.0);
                minority_vap.push(0.0);
            }
        }
        Some(rplan_core::DemographicContext { total_vap:Some(total_vap), minority_vap:Some(minority_vap) })
    } else { None };
    result["status"] = json!("prepared-report-counts");
    result["unit_ids"] = json!(ids);
    result["missing_units_defaulted_to_zero"] = json!(missing);
    result["demographic_context_json"] = json!(demographics.as_ref().map(serde_json::to_string).transpose().map_err(|e| e.to_string())?);
    result["graph_binding_verified"] = json!(false);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> Vec<String> { ["--state", "RI"].map(str::to_owned).to_vec() }

    fn missing() -> Vec<Observation> {
        let plan = inspect(args(), None, None, None).unwrap();
        plan["required_observation_paths"].as_array().unwrap().iter().map(|path| Observation {
            relative_path:path.as_str().unwrap().to_owned(), kind:Kind::Missing, source_base64:None,
        }).collect()
    }

    #[test]
    fn native_first_existing_failure_is_not_skipped() {
        let plan = inspect(args(), None, None, None).unwrap();
        let first = plan["report_candidates"][0].as_str().unwrap();
        let second = plan["report_candidates"][1].as_str().unwrap();
        let mut observations = missing();
        for entry in &mut observations {
            if entry.relative_path == first || entry.relative_path == second {
                entry.kind = Kind::File;
                entry.source_base64 = Some(STANDARD.encode(if entry.relative_path == first { "" } else {
                    "GEOID,total_vap,minority_vap\n44001000100,10,5\n"
                }));
            }
        }
        let result = inspect(args(), Some(observations), Some(vec!["44001000100".into()]), Some("explicit-file-bundle".into())).unwrap();
        assert_eq!(result["status"], "native-load-failure");
        assert_eq!(result["selected_report_path"], first);
        assert_eq!(result["report_load_failure_kind"], "csv");
        assert!(result.get("demographic_context_json").is_none());
    }

    #[test]
    fn absence_requires_complete_observations_and_never_grants_authority() {
        let plan = inspect(args(), None, None, None).unwrap();
        assert_eq!(plan["observed"], false);
        assert!(plan.get("cvap_presence_observed").is_none());
        let mut observations = missing();
        observations.pop();
        assert!(inspect(args(), Some(observations), None, Some("explicit-file-bundle".into())).is_err());
        let result = inspect(args(), Some(missing()), Some(vec!["44001000100".into()]), Some("selected-folder-snapshot".into())).unwrap();
        assert_eq!(result["demographic_context_json"], Value::Null);
        assert_eq!(result["filesystem_reads_performed"], false);
        assert_eq!(result["path_authority_granted"], false);
        assert_eq!(result["graph_binding_verified"], false);
    }
}
