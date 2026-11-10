use base64::{engine::general_purpose::STANDARD,Engine};
use std::collections::{BTreeMap,BTreeSet};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(serde::Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct PlanInput{pub label:String,pub manifest_json:Option<String>,pub analysis_json:BTreeMap<String,Option<String>>,pub present:Vec<String>,#[serde(default)]pub manifest_base64:Option<String>,#[serde(default)]pub analysis_base64:BTreeMap<String,Option<String>>}
pub(crate) fn inspect(plans:Vec<PlanInput>)->Result<Value,String>{
 if plans.len()>100{return Err("Select up to 100 plan folders.".into());}
 let mut labels=BTreeSet::new();let mut bytes=0usize;let mut reports=Vec::new();
 for mut p in plans{
  if p.manifest_json.is_some()&&p.manifest_base64.is_some()||p.analysis_base64.keys().any(|n|p.analysis_json.contains_key(n)){return Err("Supply each artifact once, as text or raw bytes.".into());}
  let mut raw_hashes=BTreeMap::new();
  let mut raw_bytes=0usize;
  let mut decode=|name:&str,encoded:Option<String>|->Result<Option<String>,String>{
   let Some(encoded)=encoded else{return Ok(None);};
   if encoded.len()>1398104{return Err("Each selected JSON source must be at most 1 MiB.".into());}
   let b=STANDARD.decode(&encoded).map_err(|_|"Invalid Doctor Base64 bytes")?;
   if b.len()>1024*1024||STANDARD.encode(&b)!=encoded{return Err("Doctor bytes must be canonical Base64 and at most 1 MiB.".into());}
   raw_bytes+=b.len();raw_hashes.insert(name.to_owned(),format!("sha256:{:x}",Sha256::digest(&b)));
   Ok(String::from_utf8(b).ok())
  };
  if p.manifest_base64.is_some(){p.manifest_json=decode("manifest.json",p.manifest_base64.take())?;}
  for (name,encoded) in p.analysis_base64{let text=decode(&name,encoded)?;p.analysis_json.insert(name,text);}
  bytes+=raw_bytes;

  if p.label.is_empty()||p.label.len()>200||!labels.insert(p.label.clone())||p.label.contains(['/', '\\', ':','\0'])||matches!(p.label.as_str(),"."|".."|"__proto__"|"constructor"|"prototype"){return Err("Plan labels must be unique safe names up to 200 UTF-8 bytes.".into());}
  if p.analysis_json.keys().any(|n|!bisect_report::plan_doctor::REQUIRED.contains(&n.as_str())&&!bisect_report::plan_doctor::OPTIONAL.contains(&n.as_str()))||p.present.len()>5||p.present.iter().collect::<BTreeSet<_>>().len()!=p.present.len()||p.present.iter().any(|n|!["audit.json","maps/districts.png","maps/compactness.png","maps/political.png",&format!("{}.rplan",p.label)].contains(&n.as_str())){return Err("Only native Doctor analysis, map, audit and named RPLAN artifacts are accepted.".into());}
  let mut hashes=BTreeMap::new();
  for (name,text) in p.analysis_json.iter().map(|(n,t)|(n.as_str(),t.as_deref())).chain(std::iter::once(("manifest.json",p.manifest_json.as_deref()))){if let Some(t)=text{if t.len()>1024*1024{return Err("Each selected JSON source must be at most 1 MiB.".into());}if !raw_hashes.contains_key(name){bytes=bytes.checked_add(t.len()).ok_or("Source byte count overflow")?;}hashes.insert(name.to_owned(),format!("sha256:{:x}",Sha256::digest(t.as_bytes())));}}
  if bytes>16*1024*1024{return Err("Selected JSON sources exceed 16 MiB.".into());}
  let mut report=match p.manifest_json.as_deref(){None=>json!({"label":p.label,"status":"no-manifest","error":"No manifest.json selected for this plan."}),Some(t)=>match serde_json::from_str::<bisect_report::manifest_schema::ReadOnlyPlanManifest>(t){Err(e)=>json!({"label":p.label,"status":"invalid-manifest","error":e.to_string()}),Ok(m)=>{let mut r=bisect_report::plan_doctor::diagnose(&p.label,&m,&p.analysis_json,&p.present);r["status"]=json!(if r["required_files_present"]==true{"required-files-present"}else{"missing-required-files"});r}}};
  hashes.extend(raw_hashes);report["source_sha256"]=json!(hashes);report["unreadable_analysis"]=json!(p.analysis_json.iter().filter_map(|(n,s)|if s.is_none(){Some(n)}else{None}).collect::<Vec<_>>());reports.push(report);
 }
 reports.sort_by(|a,b|a["label"].as_str().cmp(&b["label"].as_str()));
 let rows:Vec<_>=reports.iter().map(|r|(r["label"].as_str().unwrap().to_owned(),r["districts"].as_str().map(|n|(n.to_owned(),r["scan"].clone())))).collect();
 Ok(json!({"all_plans_table":if rows.is_empty(){String::new()}else{bisect_report::plan_doctor::all_plans_table(&rows)},"schema_version":"bisect-plan-doctor-v1","status":"inventory-complete","plans":reports,"json_source_bytes":bytes,"scope":"Read-only selected-directory inventory, including empty plan directories. Raw source hashes cover readable selected bytes, including invalid UTF-8. No manifest includes unreadable UTF-8; analysis presence is retained even when unreadable or malformed. Shared native Doctor analysis summaries, compactness thresholds and all-plan table cells. Required files present is an inventory result, not validated analysis or report correctness. Native PASS may mean file presence even for malformed JSON; readable and parse_error are shown separately. Population balance and contiguity claims are read from supplied reports, never recomputed here. Map, audit and RPLAN presence does not validate their bytes. Selected manifests share the native schema with read-only counts in the native 64-bit domain; no count is used to allocate or execute a plan. Seeds and reported split counts remain exact decimal strings. No referenced paths, URLs or commands are opened or executed. Imported project inventories and results are unverified until explicitly rerun; this does not certify legal or court readiness."}))
}
