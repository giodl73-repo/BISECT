//! Native target guards over selected local-folder evidence; no filesystem writes.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub source_name:String,pub folder_json:String,pub force:bool}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct Folder {version:String,paths:Vec<String>,manifests:BTreeMap<String,String>}
pub(crate) fn check(input:Input,version:&str,year:&str,label:&str)->Result<Value,String>{
 let relative=format!("{year}/plans/{label}");check_relative(input,version,&relative,label,true)
}
fn check_relative(input:Input,version:&str,relative:&str,label:&str,check_collision:bool)->Result<Value,String>{
 if input.source_name.is_empty()||input.source_name.len()>200||input.source_name.chars().any(char::is_control)||input.folder_json.len()>2*1024*1024{return Err("Select named run-target folder evidence up to 2 MiB.".into());}
 let folder:Folder=serde_json::from_str(&input.folder_json).map_err(|e|format!("Run target folder: {e}"))?;
 if folder.version!=version{return Err("Selected target folder version differs from the Run version. Select the intended output folder again.".into());}
 if folder.paths.len()>10000||folder.paths.iter().any(|p|p.is_empty()||p.len()>300||p.split('/').any(|s|s.is_empty()||s=="."||s=="..")||!p.bytes().all(|b|b.is_ascii_alphanumeric()||b"/_-. ".contains(&b))){return Err("Run target requires at most 10,000 bounded relative portable paths.".into());}
 let paths:BTreeSet<_>=folder.paths.into_iter().collect();
 if folder.manifests.len()>1000||folder.manifests.iter().any(|(p,t)|!paths.contains(p)||!p.ends_with("/manifest.json")||t.len()>1024*1024){return Err("Run target manifest text must match selected paths and stay within 1 MiB per manifest.".into());}
 let manifest=format!("{relative}/manifest.json");let incomplete=format!("{relative}/manifest.tmp");
 // Same ordering as native State: force never bypasses an incomplete manifest.
 bisect_report::run_target::check_incomplete_plan_evidence(label,paths.contains(&incomplete)).map_err(|e|e.to_string())?;
 let target=format!("outputs/{version}/{relative}");
 if check_collision {bisect_report::run_target::check_plan_collision_evidence(std::path::Path::new(&target),input.force,paths.contains(&manifest),folder.manifests.get(&manifest).map(String::as_str)).map_err(|e|e.to_string())?;}
 Ok(json!({"schema_version":"bisect-native-run-target-v1","status":"ready","target_namespace":target,"source_name":input.source_name,"source_sha256":format!("{:x}",Sha256::digest(input.folder_json.as_bytes())),"force":input.force,"existing_manifest":paths.contains(&manifest),"manifest_sha256":folder.manifests.get(&manifest).map(|t|format!("{:x}",Sha256::digest(t.as_bytes()))),"scope":"Shared native State incomplete-manifest and label-collision guards, in native order, on explicitly selected local-folder evidence. Force allows a completed-manifest collision but never an incomplete manifest.tmp. Recorded folder presence is a snapshot/user-supplied claim; no filesystem was written or deleted. Output download is a newly generated package, not installation or atomic folder promotion. Open is inert; changing the target requires another explicit Run."}))
}

pub(crate) fn inspect(input:Input,version:String,year:String,label:String)->Result<Value,String>{
 if version.is_empty()||version.len()>100||!version.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))||year.len()!=4||!year.bytes().all(|b|b.is_ascii_digit())||label.len()>100{return Err("Choose a portable version/label up to 100 bytes and a four-digit year.".into());}
 bisect_report::label_rename::validate_name(&label)?;
 check(input,&version,&year,&label)
}

/// Native unlabeled State writer namespace, also used by bulk completion checks.
pub(crate) fn inspect_state(input:Input,version:String,year:String,state:String)->Result<Value,String>{
 if version.is_empty()||version.len()>100||!version.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))||state.len()!=2||!state.bytes().all(|b|b.is_ascii_uppercase()){return Err("Choose a portable output version and uppercase state code.".into());}
 let text=include_str!("../../../data/manifest.json");let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(text).map_err(|e|e.to_string())?;
 let rows=bisect_core::state_catalog::congressional_states(&manifest,&year)?;
 let (_,slug,_)=rows.into_iter().find(|r|r.0==state).ok_or("State is unavailable in the native bulk catalog for this year.")?;
 bisect_report::label_rename::validate_name(&slug)?;
 let relative=format!("{year}/states/{slug}");let mut output=check_relative(input,&version,&relative,&slug,false)?;
 output["layout"]=json!("state");output["state_code"]=json!(state);output["state_slug"]=json!(slug);
 output["catalog_sha256"]=json!(format!("{:x}",Sha256::digest(text.as_bytes())));
 output["completion_paths"]=json!(bisect_core::state_catalog::state_completion(&state,&slug,&year,true,|_|false)?.marker_paths);
 output["historical_completion_paths"]=json!(bisect_core::state_catalog::legacy_completion_paths(&state,&year));
 output["scope"]=json!("Shared native bulk state slug and unlabeled writer layout from the embedded manifest. Native incomplete-manifest guard is applied. The native unlabeled writer does not apply labeled manifest-collision checks; the browser separately requires explicit reviewed overwrite for any existing target entries. Bulk completion recognizes this state-name folder before historical lowercased-code folders; incomplete targets remain queued for recovery. Supplied/live folder observations do not authenticate origin; no filesystem is written here.");Ok(output)
}

pub(crate) fn portable_state_component(name:&str)->bool{
 if name.is_empty()||name.len()>100||name.trim()!=name||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b"_- ".contains(&b)){return false;}
 let lower=name.to_ascii_lowercase();
 !["__proto__","constructor","prototype","con","prn","aux","nul"].contains(&lower.as_str())
  &&!(lower.len()==4&&(lower.starts_with("com")||lower.starts_with("lpt"))&&matches!(lower.as_bytes()[3],b'1'..=b'9'))
}

/// Resolve a custom State target from the original flags, never a supplied slug.
pub(crate) fn inspect_named_state(input:Input,version:String,arguments:Vec<String>)->Result<Value,String>{
 if version.is_empty()||version.len()>100||!version.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b)){return Err("Choose a portable output version.".into());}
 let identity=crate::state_identity::inspect(arguments)?;
 let year=identity["effective_year"].as_str().ok_or("Native State location/year preflight failed")?;
 if year.len()!=4||!year.bytes().all(|b|b.is_ascii_digit()){return Err("Named State output requires a four-digit native year.".into());}
 let state=identity["state_code"].as_str().ok_or("Missing native State code")?;
 let name=identity["effective_state_name"].as_str().ok_or("Missing native State name")?;
 let label=identity["effective_label"].as_str().ok_or("Missing native State label")?;
 let layout=identity["layout"].as_str().ok_or("Missing native State layout")?;
 let mut output=if layout=="labeled"{
  if !portable_state_component(label){return Err("Choose a portable explicit native label before output review.".into());}
  inspect(input,version.clone(),year.to_owned(),label.to_owned())?
 }else{
  if !portable_state_component(name){return Err("Native state name is not a portable single output component; the original name was not changed.".into());}
  let relative=format!("{year}/states/{name}");
  let mut result=check_relative(input,&version,&relative,label,false)?;
  result["state_slug"]=json!(name);result
 };
 output["layout"]=json!(layout);output["state_code"]=json!(state);output["identity"]=identity;
 output["scope"]=json!("Original native State flags resolve the name, year, label and writer namespace. Portable names retain case and internal spaces; unsafe names are refused without sanitization. Existing incomplete/collision guards use selected folder observations. No path, overwrite, deletion or execution authority is granted.");
 Ok(output)
}

#[cfg(test)]
mod named_tests{
 use super::*;
 fn input(paths:Vec<&str>,force:bool)->Input{Input{source_name:"selected.json".into(),folder_json:json!({"version":"v1","paths":paths,"manifests":{}}).to_string(),force}}
 fn args(alias:&str)->Vec<String>{["--state","RI","--year","2020","--adjacency","not-opened.adj.bin","--state-name",alias].iter().map(|v|v.to_string()).collect()}
 #[test]
 fn exact_raw_namespace_and_incomplete_guard(){
  let result=inspect_named_state(input(vec![],false),"v1".into(),args("Research Area")).unwrap();
  assert_eq!(result["target_namespace"],"outputs/v1/2020/states/Research Area");
  assert_eq!(result["identity"]["effective_label"],"research_area_congressional_2020");
  assert!(inspect_named_state(input(vec!["2020/states/Research Area/manifest.tmp"],true),"v1".into(),args("Research Area")).is_err());
  for bad in ["../outside","CON","__proto__","trailing ",""] {assert!(inspect_named_state(input(vec![],false),"v1".into(),args(bad)).is_err(),"{bad}");}
 }
 #[test]
 fn explicit_label_retains_native_precedence(){
  let mut a=args("../raw-observation");a.extend(["--label".into(),"chosen_plan".into()]);
  let result=inspect_named_state(input(vec![],false),"v1".into(),a).unwrap();
  assert_eq!(result["target_namespace"],"outputs/v1/2020/plans/chosen_plan");
  assert_eq!(result["identity"]["effective_state_name"],"../raw-observation");
 }
}
