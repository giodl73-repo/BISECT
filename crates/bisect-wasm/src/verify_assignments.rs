//! Native assignments-only branch over two explicitly selected inventories.
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub prechecks: crate::manifest_prechecks::Input,
    pub output_files_base64: BTreeMap<String, String>,
    pub explicit_label: Option<String>,
    pub reference_json: Option<String>,
    pub min_similarity: f64,
}

pub(crate) fn verify(input: Input) -> Result<Value, String> {
    let sidecars = crate::package_verification::selected_bytes(input.prechecks.files_base64.clone())?;
    let output = crate::package_verification::selected_bytes(input.output_files_base64.clone())?;
    let total = sidecars.values().chain(output.values()).map(Vec::len).sum::<usize>()
        + input.prechecks.manifest_json.len() + input.reference_json.as_ref().map_or(0, String::len);
    if total > 16 * 1024 * 1024 || !input.min_similarity.is_finite() {
        return Err("Combined assignments-only sources exceed 16 MiB or threshold is not finite.".into());
    }
    let manifest_json = input.prechecks.manifest_json.clone();
    let prechecks = crate::manifest_prechecks::verify(input.prechecks)?;
    let mut discovery = Value::Null;
    let mut comparison = Value::Null;
    let mut error = prechecks["error"].clone();
    let status = if prechecks["native_exit_code"] != 0 { "precheck-failed" } else {
        match crate::assignment_discovery::discover(crate::assignment_discovery::Input {
            manifest_json, explicit_label: input.explicit_label, files_base64: input.output_files_base64,
        }) {
            Err(message) => { error = json!(message); "discovery-failed" }
            Ok(receipt) => {
                discovery = receipt;
                match discovery["status"].as_str() {
                    Some("failed") => { error = discovery["error"].clone(); "discovery-failed" }
                    Some("not-found" | "empty") => "unavailable",
                    Some("found") => match input.reference_json {
                        None => { error = json!("Assignments-only verification requires a selected reference assignment file."); "reference-required" }
                        Some(reference_json) => match crate::assignment_verification::compare(crate::assignment_verification::Input {
                            original_json: discovery["source_json"].as_str().unwrap().to_owned(),
                            reference_json, min_similarity: input.min_similarity,
                        }) {
                            Ok(receipt) => { comparison = receipt; if comparison["status"] == "match" {"match"} else {"different"} }
                            Err(message) => { error = json!(message); "reference-failed" }
                        },
                    },
                    _ => { error = json!("Unexpected discovery status."); "discovery-failed" }
                }
            }
        }
    };
    Ok(json!({"schema_version":"bisect-assignments-only-verification-v1",
        "status":status,"native_exit_code":if status=="match"{0}else{1},
        "prechecks":prechecks,"discovery":discovery,"comparison":comparison,"error":error,
        "execution_performed":false,
        "scope":"Native PlanManifest assignments-only branch in native order: ILP/certificate prechecks, original assignment discovery, empty/missing-original stop, then reference parsing and literal agreement comparison. Selected output paths are relative to the output root; sidecar paths are relative to the selected manifest. Failed earlier stages prevent later parsing and comparison. No engine runs and no native executable hash check occurs in this branch. Selected inventories do not prove external directory completeness or author authenticity. Comparison does not establish population validity, legal validity or optimality. Public-example/benchmark package manifests use the separate package verifier."}))
}
