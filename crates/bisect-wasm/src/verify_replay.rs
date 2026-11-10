//! Compose the existing native manifest checks and selected-artifact replay.
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub prechecks: crate::manifest_prechecks::Input,
    pub replay: crate::engine_replay::Input,
    #[serde(default)]
    pub source_graph_json: Option<String>,
    #[serde(default)]
    pub inspect_only: bool,
}

fn source_graph_binding(run_json:&str,text:&str)->Result<Value,String>{
    use sha2::{Digest,Sha256};
    if text.len()>16*1024*1024{return Err("Selected source graph exceeds 16 MiB.".into());}
    let graph:crate::engine::PreparedGraph=serde_json::from_str(text).map_err(|e|format!("Selected source graph: {e}"))?;
    let source=serde_json::to_value(graph).map_err(|e|e.to_string())?;
    let selected=crate::engine_replay::binding_inputs(run_json)?;
    if source!=selected["request"]["graph"]{return Err("Regenerated original-source graph differs from the actual selected replay graph.".into());}
    Ok(json!({"status":"verified","source_graph_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"canonical_prepared_graph_sha256":rplan_core::canonical_sha256(&source).map_err(|e|e.to_string())?,"scope":"Typed native graph equality against the actual selected replay input; no JavaScript numeric normalization or partition. This does not authenticate upstream source bytes."}))
}

#[cfg(test)]
mod source_tests {
    use super::*;
    #[test]
    fn source_graph_equality_preserves_integer_bits_without_partition() {
        let graph=r#"{"schema_version":1,"state":"RI","year":"2020","geoids":["44001000100"],"adjacency":[[]],"population":[9007199254740993],"edges":[],"areas":[],"exterior_perimeters":[],"centroids":[]}"#;
        let run=format!(r#"{{"kind":"engine","request":{{"graph":{graph},"options":{{"structure":"standard-bisect","weights":"geographic","search":"single","districts":1,"seed":"18446744073709551615","seeds":1,"steps":10,"percentile":0,"alpha_county":0,"balance_tolerance":5,"area_swing":1.1,"iterations":10}}}}}}"#);
        let checked=source_graph_binding(&run,graph).unwrap();
        assert_eq!(checked["status"],"verified");
        assert_eq!(source_graph_binding(&run,&format!(" \n{graph}\n")).unwrap()["canonical_prepared_graph_sha256"],checked["canonical_prepared_graph_sha256"]);
        let changed=graph.replace("9007199254740993","9007199254740992");
        assert!(source_graph_binding(&run,&changed).unwrap_err().contains("differs from the actual"));
    }
}

pub(crate) fn verify(input: Input, runtime:Option<crate::manifest::RuntimeArtifact>, skip_binary_check:bool) -> Result<Value, String> {
    let decoded =
        crate::package_verification::selected_bytes(input.prechecks.files_base64.clone())?;
    let total = decoded.values().map(Vec::len).sum::<usize>()
        + input.prechecks.manifest_json.len()
        + input.replay.run_json.len()
        + input.replay.original_assignments_json.len()
        + input.source_graph_json.as_ref().map_or(0,String::len);
    if total > 16 * 1024 * 1024 {
        return Err("Combined verification sources exceed 16 MiB.".into());
    }
    let manifest_json = input.prechecks.manifest_json.clone();
    let manifest_path = input.prechecks.manifest_path.clone();
    let prechecks = crate::manifest_prechecks::verify(input.prechecks)?;
    let mut binary_check = json!({"status":"not-run","advisory":true});
    let mut replay = Value::Null;
    let mut binding = json!({"status":"not-run"});
    let mut source_binding=json!({"status":"not-run"});
    let mut error = prechecks.get("error").cloned().unwrap_or(Value::Null);
    let status = if prechecks["native_exit_code"] != 0 {
        "precheck-failed"
    } else {
        let manifest:bisect_report::PlanManifest=serde_json::from_str(&manifest_json).map_err(|e|e.to_string())?;
        binary_check=crate::manifest::binary_check(&manifest.binary_sha256,runtime,skip_binary_check)?;
        match crate::replay_bindings::check(
            &manifest_json,
            &manifest_path,
            &decoded,
            &input.replay.run_json,
            &input.replay.original_assignments_json,
        ) {
            Err(message) => {
                error = Value::String(message.clone());
                binding = json!({"status":"failed","error":message});
                "binding-failed"
            }
            Ok(receipt) => {
                binding = receipt;
                let source_check=input.source_graph_json.as_deref().map(|text|source_graph_binding(&input.replay.run_json,text)).transpose();
                match source_check {
                    Err(message)=>{source_binding=json!({"status":"failed","error":message});error=source_binding["error"].clone();"binding-failed"},
                    Ok(checked)=>{
                if let Some(checked)=checked{source_binding=checked;}
                if input.inspect_only {"binding-checked-replay-not-run"}else{match crate::engine_replay::replay(input.replay) {
                    Ok(receipt) => {
                        replay = receipt;
                        error = replay.get("error").cloned().unwrap_or(Value::Null);
                        match replay["status"].as_str() {
                            Some("match") => "match",
                            Some("different") => "different",
                            Some("unavailable") => "unavailable",
                            _ => "replay-failed",
                        }
                    }
                    Err(message) => {
                        error = Value::String(message);
                        "replay-failed"
                    }
                }}
                    }
                }
            }
        }
    };
    let mut result=json!({
        "schema_version":"bisect-manifest-replay-verification-v1",
        "status":status,"verification_exit_code":if status=="match"{0}else{1},
        "prechecks":prechecks,"binary_check":binary_check,"skip_binary_check":skip_binary_check,"replay":replay,"error":error,
        "execution_performed":replay.get("execution_performed").and_then(Value::as_bool).unwrap_or(false),
        "execution_completed":replay.get("execution_completed").and_then(Value::as_bool).unwrap_or(false),
        "manifest_to_run_binding":binding["status"],"binding":binding,
        "scope":"Checks the selected native manifest ILP/certificate sidecars first, then records the shared native advisory binary hash check before binding and explicitly executing the selected portable run artifact and compares supplied original assignments. Precheck failures stop before parsing or executing replay. Absent recorded sidecars remain not-recorded and permit replay. Stage receipts and exact selected source hashes are retained. A match establishes only that these selected prechecks passed and replay assignments met the selected native threshold. Selected manifest/run identity and original assignment correspondence are checked before execution. Recorded plan/options/prepared-graph evidence is checked when present; missing evidence yields partial binding. Auxiliary inputs bind through recorded native source hashes/context and available RPLAN provenance; missing evidence remains partial. This is not complete native CLI Verify orchestration: filesystem discovery, fetching and remaining native manifest-to-engine orchestration remain pending. The browser worker supplies the measured loaded WASM fingerprint; direct ABI callers supply it as a claim. Native executable and WASM artifact formats differ; mismatch or an unavailable fingerprint never blocks replay. Explicit skip uses native sentinel-before-skip precedence. Failed prechecks leave binary comparison not-run. No author authenticity, legal validity or optimality is asserted."
    });
    if input.source_graph_json.is_some(){result["source_graph_binding"]=source_binding;}
    if input.inspect_only{result["inspect_only"]=json!(true);result["inspection_exit_code"]=json!(if status=="binding-checked-replay-not-run"{0}else{1});}
    Ok(result)
}
