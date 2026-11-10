use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};
use bisect_report::tui_library::{SortColumn,SortDir};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct PlanInput {pub directory:String,pub folder_name:String,pub manifest_json:Option<String>,pub analysis_json:BTreeMap<String,String>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub plans:Vec<PlanInput>,pub filter:String,pub sort_column:SortColumn,pub sort_direction:SortDir}
pub(crate) fn inspect(input:Input)->Result<Value,String>{
 if input.plans.len()>100||input.filter.len()>200||input.filter.chars().any(char::is_control){return Err("Plan library supports 100 plans and a filter of at most 200 bytes without controls.".into());}
 let mut seen=BTreeSet::new();let mut bytes=0usize;
 for plan in &input.plans {
  if plan.directory.is_empty()||plan.directory.len()>240||plan.directory.split('/').any(|p|p.is_empty()||p=="."||p==".."||["__proto__","constructor","prototype"].contains(&p)||p.chars().any(|c|c.is_control()||c=='\\'||c==':'))||plan.folder_name.is_empty()||plan.folder_name.len()>200||plan.folder_name.chars().any(|c|c.is_control()||c=='/'||c=='\\'||c==':')||!seen.insert(plan.directory.clone())||plan.analysis_json.keys().any(|k|!["compactness.json","summary.json","splits.json","contiguity.json"].contains(&k.as_str())) {return Err("Invalid or duplicate plan library source path.".into());}
  for text in plan.manifest_json.iter().chain(plan.analysis_json.values()){if text.len()>1024*1024{return Err("Each library JSON source is limited to 1 MiB.".into());}bytes+=text.len();}
 }
 if bytes>16*1024*1024{return Err("Selected plan library exceeds 16 MiB.".into());}
 let policy:Value=serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
 let registry=bisect_analysis::location_registry::PolicyRegistry::new(&policy);
 let mut rows=Vec::new();let mut skipped=Vec::new();let mut sources=Vec::new();
 for plan in &input.plans {
  for (name,text) in plan.manifest_json.iter().map(|t|("manifest.json".to_string(),t)).chain(plan.analysis_json.iter().map(|(name,text)|(format!("analysis/{name}"),text))){sources.push(json!({"directory":plan.directory,"path":name,"bytes":text.len(),"sha256":format!("{:x}",Sha256::digest(text.as_bytes()))}));}
  let row=plan.manifest_json.as_ref().and_then(|text|bisect_report::tui_library::read_plan(&plan.directory,&plan.folder_name,text,&plan.analysis_json,|state|registry.state_name(state).unwrap_or_else(||state.to_string())));
  match row {Some(row)=>{if [&row.label,&row.state_code,&row.state_name,&row.chamber,&row.year].iter().any(|s|s.len()>200||s.chars().any(char::is_control)){return Err("Plan summary fields exceed display limits.".into());}rows.push(row);},None=>skipped.push(json!({"directory":plan.directory,"reason":"missing-or-invalid-manifest"}))}
 }
 // Native directory discovery sorts label first; preserve this base order when
 // metric ties are compared and when filter/sort is changed later.
 rows.sort_by(|a,b|a.label.cmp(&b.label));
 let visible=bisect_report::tui_library::visible_plans(&rows,&input.filter,&input.sort_column,&input.sort_direction);
 Ok(json!({"schema_version":"bisect-tui-plan-library-v1","status":"listed","plans":visible,"total_plans":rows.len(),"skipped":skipped,"source_hashes":sources,"filter":input.filter,"sort_column":input.sort_column,"sort_direction":input.sort_direction,"execution_performed":false,"scope":"Shared native TUI Home summary projection and stable filtering/sorting over explicitly selected local manifest/analysis sources. Missing or malformed manifests skip rows; absent/malformed/unrecognized metrics remain unknown, not recomputed. Counts are exact decimal strings. Sources and analysis declarations are unverified claims. Paths are identifiers only, never opened or fetched. Session/folder discovery, Run/Compare/Verify/Doctor navigation and full TUI parity remain incomplete. Opening a project does not inspect or execute it."}))
}
