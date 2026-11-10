use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Mode {
    Prechecks,
    Replay,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub mode: Mode,
    pub prechecks: crate::manifest_prechecks::Input,
    pub replay: Option<crate::engine_replay::Input>,
}
pub(crate) fn verify(
    input: Input,
    runtime: Option<crate::manifest::RuntimeArtifact>,
) -> Result<Value, String> {
    let path = input.prechecks.manifest_path.clone();
    let text = input.prechecks.manifest_json.clone();
    let binary = crate::manifest::inspect(&path, &text, runtime.clone())?;
    let manifest = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    let (mode,receipt)=match (input.mode,input.replay){
        (Mode::Prechecks,None)=>("prechecks",crate::manifest_prechecks::verify(input.prechecks)?),
        (Mode::Replay,Some(replay))=>("replay",crate::verify_replay::verify(crate::verify_replay::Input{prechecks:input.prechecks,replay,source_graph_json:None,inspect_only:false},runtime,false)?),
        _=>return Err("Prechecks mode takes no replay request; replay mode requires original assignments and a run artifact.".into())
    };
    let prechecks = if mode == "prechecks" {
        &receipt
    } else {
        &receipt["prechecks"]
    };
    let prechecks_ok = prechecks["native_exit_code"] == 0;
    let completed = mode == "replay" && receipt["execution_completed"] == true;
    let matched = mode == "replay" && completed && receipt["status"] == "match";
    let status = if mode == "prechecks" {
        if prechecks_ok {
            "prechecks-passed-replay-not-run"
        } else {
            "prechecks-failed"
        }
    } else {
        receipt["status"].as_str().unwrap_or("replay-failed")
    };
    let recorded = |key: &str| {
        manifest.get(key).filter(|v| !v.is_null()).map(|v| match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        })
    };
    let view = json!({"label":recorded("label"),"state_code":recorded("state_code"),"year":recorded("year"),"manifest_path_used":path,"replay_matched":matched,"assignment_similarity":if completed{receipt["replay"]["agreement"]["similarity"].clone()}else{Value::Null},"seed_recorded":manifest.get("seed").is_some_and(|v|!v.is_null())||manifest.get("engine_seed").is_some_and(|v|!v.is_null()),"recorded_seed":recorded("engine_seed").or_else(||recorded("seed")),"metis_version_recorded":recorded("metis_version"),"binary_check":binary["binary_check"],"adjacency_check":"not-independently-rehashed","tiger_check":"not-independently-rehashed","manifest_run_binding":if mode=="replay"{receipt["manifest_to_run_binding"].clone()}else{json!("not-run")}});
    Ok(
        json!({"schema_version":"bisect-tui-verification-v1","status":status,"mode":mode,"view":view,"prechecks":prechecks,"verification":receipt,"binary_provenance":binary,"execution_performed":mode=="replay"&&receipt["execution_performed"]==true,"execution_completed":completed,"scope":"Native TUI Verify workflow with explicit native ILP/certificate prechecks and optional manifest-bound portable engine replay. Native TUI currently calls CLI --dry-run and parses console PASS/FAIL strings; browser decisions use typed stage receipts, never console substrings. Passing prechecks does not imply an engine ran or assignments matched. Binary identity is advisory and compares the loaded WASM artifact literally with a recorded native hash; artifact kinds differ. Seed/METIS fields are recorded claims. Raw adjacency/TIGER bytes are not independently rehashed by this operation; manifest-to-run evidence retains its own verified/partial status. Source paths are selected inventory identifiers, never opened or fetched. No legal compliance, authenticity or optimality is certified. Open is inert and replay requires explicit Run."}),
    )
}
