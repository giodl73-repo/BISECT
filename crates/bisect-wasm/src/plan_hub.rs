//! Portable native hub dispatch, with explicit browser navigation only.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {registry_json:Option<String>,label:Option<String>,configure:bool,config_path:Option<String>,library:Option<crate::tui_library::Input>}
pub(crate) fn inspect(input:Input)->Result<Value,String>{
    for value in [&input.label,&input.config_path].into_iter().flatten(){if value.is_empty()||value.len()>200||value.chars().any(char::is_control){return Err("Hub identifiers must be nonempty text up to 200 bytes without control characters.".into());}}
    if input.registry_json.as_ref().is_some_and(|s|s.len()>1024*1024){return Err("Hub registry exceeds 1 MiB.".into());}
    let registry=bisect_report::label_registry::parse(input.registry_json.as_deref().unwrap_or(""));
    if registry.as_ref().is_ok_and(|r|r.len()>500||r.iter().any(|(n,e)|n.len()>200||n.chars().any(char::is_control)||["__proto__","constructor","prototype"].contains(&n.as_str())||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100)){return Err("Hub registry supports 500 safe labels and 100 recorded years per stage.".into());}
    let compact=registry.as_ref().map(|r|serde_json::to_string(r)).ok().transpose().map_err(|e|e.to_string())?.unwrap_or("{}".into());
    let args=bisect_report::plan_hub::launch_arguments(input.label.as_deref(),input.configure,input.config_path.as_deref().map(std::ffi::OsStr::new)).iter().map(|s|s.to_string_lossy().into_owned()).collect::<Vec<_>>();
    let mode=if input.configure{"configure"}else if input.label.is_some(){"label"}else{"picker"};
    let fallback=format!("{}{}",bisect_report::plan_hub::MISSING_NOTICE,bisect_report::plan_hub::fallback(input.label.as_deref(),input.configure,&registry));
    let labels=registry.as_ref().map(|r|r.iter().map(|(name,entry)|json!({"label":name,"built":entry.built,"analyzed":entry.analyzed,"reported":entry.reported})).collect::<Vec<_>>()).unwrap_or_default();
    let library=input.library.map(crate::tui_library::inspect).transpose()?;
    let library_plans=library.as_ref().and_then(|v|v["plans"].as_array());
    let matched=library_plans.and_then(|rows|bisect_report::plan_hub::label_index(rows.iter().map(|row|row["label"].as_str().unwrap_or("")),input.label.as_deref()));
    let selected=library_plans.and_then(|rows|rows.get(matched.unwrap_or(0))).map(|row|row["plan_dir"].clone());
    Ok(json!({"schema_version":"bisect-plan-hub-v1","mode":mode,"selected_label":input.label,"config_path":input.config_path,
        "native_launch_arguments":args,"native_registry_json":compact,"native_fallback_text":fallback,
        "registry_error":registry.as_ref().err(),"registry_source_sha256":input.registry_json.as_ref().map(|s|format!("{:x}",Sha256::digest(s.as_bytes()))),
        "label_found":input.label.as_ref().map(|name|registry.as_ref().is_ok_and(|r|r.contains_key(name))),"labels":labels,
        "selected_directory":selected,"label_selection_matched":matched.is_some(),"library_source_hashes":library.as_ref().map(|v|v["source_hashes"].clone()),
        "execution_performed":false,"scope":"Shared native plan command argument order, fallback precedence/text and exact-label Home preselection. Optional selected library sources use the existing native reader/filter/sort; a missing label keeps the first visible plan, matching default selection. Registry stages and library summaries are supplied claims, not file verification. Missing/malformed registry forwards an empty desktop registry; malformed show fallback records the native parse error. Paths are informational text and are never opened. Browser navigation to existing library/configuration tools is explicit and performs no engine execution. Desktop binary discovery, subprocess spawning and terminal interaction stay native. Native TUI currently stores --configure/--config flags without wiring a compositor startup screen; the browser configuration handoff is an explicit browser workflow, not a claim that desktop startup is implemented."}))
}
