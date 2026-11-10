use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {manifest_json:Option<String>,year:String,states:Vec<String>,#[serde(default)]completion:Option<Completion>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Completion {source_name:String,paths_json:String,reprocess:bool}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct Paths {paths:Vec<String>}
pub(crate) fn inspect(input:Input)->Result<Value,String>{
 if input.states.len()>100||input.states.iter().any(|s|s.len()!=2||!s.bytes().all(|b|b.is_ascii_alphabetic())){return Err("Select at most 100 two-letter state filters".into());}
 let builtin=input.manifest_json.is_none();let text=input.manifest_json.unwrap_or_else(||include_str!("../../../data/manifest.json").into());
 if text.len()>2*1024*1024{return Err("State manifest exceeds 2 MiB".into());}
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(&text).map_err(|e|format!("State manifest: {e}"))?;
 if manifest.states.len()>51||manifest.states.iter().any(|(code,state)|code.len()!=2||!code.bytes().all(|b|b.is_ascii_uppercase())||state.name.is_empty()||state.name.len()>200||state.fips.len()!=2||!state.fips.bytes().all(|b|b.is_ascii_digit())||state.districts.get(&input.year).is_some_and(|n|*n>500)){return Err("State manifest exceeds the supported catalog metadata bounds".into());}
 let all=bisect_core::state_catalog::congressional_states(&manifest,&input.year)?;
 let filter:BTreeSet<_>=input.states.into_iter().map(|s|s.to_uppercase()).collect();
 let known:BTreeSet<_>=all.iter().map(|r|r.0.clone()).collect();
 let ignored:Vec<_>=filter.difference(&known).cloned().collect();
 let mut selected:Vec<_>=all.into_iter().filter(|r|filter.is_empty()||filter.contains(&r.0)).collect();
 let mut completion_evidence=None;
 if let Some(c)=input.completion {
  if c.source_name.is_empty()||c.source_name.len()>200||c.source_name.chars().any(char::is_control)||c.paths_json.len()>2*1024*1024{return Err("Select a named completion path list up to 2 MiB.".into());}
  let p:Paths=serde_json::from_str(&c.paths_json).map_err(|e|format!("Completion paths: {e}"))?;
  if p.paths.len()>10000||p.paths.iter().any(|p|p.is_empty()||p.len()>300||p.split('/').any(|s|s.is_empty()||s=="."||s=="..")||!p.bytes().all(|b|b.is_ascii_alphanumeric()||b"/_-.".contains(&b))){return Err("Completion paths require at most 10,000 bounded relative portable paths.".into());}
  let paths:BTreeSet<_>=p.paths.into_iter().collect();let mut decisions=Vec::new();
  let mut queued=Vec::new();
  for r in selected {
   let decision=bisect_core::state_catalog::state_completion(&r.0,&r.1,&input.year,c.reprocess,|p|paths.contains(p))?;
   decisions.push(json!({"state_code":r.0,"marker_paths":decision.marker_paths,"observed_paths":decision.observed_paths,"incomplete_paths":decision.incomplete_paths,"reason":decision.reason,"status":if decision.complete{"skipped-already-complete"}else{"queued"}}));
   if !decision.complete {queued.push(r);}
  }
  selected=queued;
  completion_evidence=Some(json!({"schema_version":"bisect-native-state-completion-v1","source_name":c.source_name,"source_sha256":format!("{:x}",Sha256::digest(c.paths_json.as_bytes())),"reprocess":c.reprocess,"supplied_paths":paths.len(),"decisions":decisions,"scope":"Shared native State/States completion predicate: actual state-name folders first, historical lowercased-code folders second. Either final_assignments.json or .pkl presence skips a state unless an incomplete marker is observed in that namespace or reprocess is true. An incomplete canonical target prevents historical fallback. Reprocess bypasses observations and does not bypass the writer's separate incomplete-target guard. Paths are selected existence claims; contents and engine success are not validated. Labeled plan folders are not bulk completion markers. No filesystem was read, deleted or executed."}));
 }
 let selection_json=serde_json::to_string(&selected).map_err(|e|e.to_string())?;
 let rows:Vec<_>=selected.iter().map(|(code,name,districts)|json!({"state_code":code,"state_name":manifest.states[code].name,"state_slug":name,"fips":manifest.states[code].fips,"districts":districts})).collect();
 let mut output=json!({"schema_version":"bisect-state-batch-selection-v1","status":"selected","year":input.year,"manifest_source":if builtin{"builtin"}else{"selected"},"manifest_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"selection_json":selection_json,"selection_sha256":format!("{:x}",Sha256::digest(selection_json.as_bytes())),"states":rows,"ignored_states":ignored,"requested_states":filter,"execution_performed":false,"acquisition_performed":false,"scope":"Shared native bisect states congressional enumeration: supported Census year, nonzero/missing allocation filtering, state-name slugging, code sort, case-insensitive deduplicated state selection. Unknown or unavailable filters are disclosed. Empty filter selects all eligible states; a nonempty filter with no eligible matches selects none. Manifest metadata are source claims. This selects a batch universe, without observing completion markers, loading graphs, fetching data, running engines/analyses, writing outputs or certifying plans. Native full batch algorithm/options, scheduling and result collection remain separate."});
 if let Some(e)=completion_evidence {output["completion"]=e;output["scope"]=json!("Native state enumeration and selected completion-marker filtering. Inspect completion evidence; scheduling, fetching and engine execution remain separate.");}
 Ok(output)
}
