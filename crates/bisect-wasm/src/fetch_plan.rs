use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub manifest_json:Option<String>,pub states:Vec<String>,pub year:String,
    pub data_types:Vec<bisect_core::fetch_plan::DataType>,pub selected_paths:Vec<String>,
}
pub(crate) fn plan(input:Input)->Result<Value,String>{
    if input.states.len()>51||input.states.iter().any(|s|s.len()!=2||!s.bytes().all(|b|b.is_ascii_uppercase()))||!["2000","2010","2020","all"].contains(&input.year.as_str())||input.data_types.len()>12{return Err("Select up to 51 uppercase state codes, a Census vintage (or all) and native Fetch types.".into());}
    if input.selected_paths.len()>1000{return Err("Selected Fetch inventory exceeds 1000 paths".into());}
    let mut seen=BTreeSet::new();for path in &input.selected_paths{if !crate::package_verification::safe(path)||!seen.insert(path.to_ascii_lowercase()){return Err("Selected Fetch paths must be unique portable relative paths".into());}}
    let builtin=input.manifest_json.is_none();let text=input.manifest_json.unwrap_or_else(||include_str!("../../../data/manifest.json").into());
    if text.len()>2*1024*1024{return Err("Fetch manifest exceeds 2 MiB".into());}
    let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(&text).map_err(|e|format!("Fetch manifest: {e}"))?;
    if manifest.states.len()>51||!crate::package_verification::safe(&manifest.local_data_dir)||!crate::package_verification::safe(&manifest.local_outputs_dir){return Err("Fetch manifest requires portable relative data/output roots and up to 51 states".into());}
    let inventory:BTreeSet<_>=input.selected_paths.into_iter().collect();let mut warnings=vec![];
    let years:Vec<&str>=if input.year=="all"{vec!["2020","2010","2000"]}else{vec![input.year.as_str()]};
    let mut items=Vec::new();
    for year in years {items.extend(bisect_core::fetch_plan::build_fetch_list(&manifest,&input.states,year,&input.data_types,|p|inventory.contains(&p.to_string_lossy().replace('\\',"/")),|w|warnings.push(w.to_string())));}

    let mut rows=vec![];
    for item in items {
        let path=item.local_path.to_string_lossy().replace('\\',"/");let marker=item.done_marker.to_string_lossy().replace('\\',"/");
        if !crate::package_verification::safe(&path)||!crate::package_verification::safe(&marker){return Err("Fetch item produces an unsafe portable path".into());}
        let done=inventory.contains(&marker)&&item.available_locally;
        rows.push(json!({"state_code":item.state_code,"year":item.year,"kind":item.kind,"url":item.url,"local_path":path,"done_marker":marker,"available_locally":item.available_locally,"done":done,"check_status":if done{"done"}else if item.available_locally{"file"}else{"need"}}));
    }
    rows.sort_by_key(|r|(r["state_code"].as_str().unwrap_or("").to_owned(),r["kind"].as_str().unwrap_or("").to_owned(),r["local_path"].as_str().unwrap_or("").to_owned()));
    let ignored:Vec<_>=input.states.iter().filter(|s|!manifest.states.contains_key(*s)).collect();
    Ok(json!({"schema_version":"bisect-fetch-source-plan-v1","status":"planned","year":input.year,"manifest_source":if builtin{"builtin"}else{"selected"},"manifest_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"items":rows,"warnings":warnings,"ignored_states":ignored,"execution_performed":false,"acquisition_performed":false,
        "scope":"Shared native Fetch source selection and URL/path construction. Native all selects TIGER, PL94 and adjacency; optional sources require explicit types. Unknown selected states are ignored and disclosed. Rows are sorted for display. File/done statuses observe selected paths only, not disk existence or content validity. A marker alone is not completion. Manifest overrides are selected explicitly; native environment/home lookup and GitHub authentication are host responsibilities. URLs are source claims; no request, download, extraction, installation, activation or engine run occurs."}))
}
