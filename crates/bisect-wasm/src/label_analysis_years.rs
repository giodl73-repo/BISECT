//! Bounded label-year orchestration over the same native single-year analyzer.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub batches:BTreeMap<String,crate::label_analysis::Input>,
 pub created:String,
 pub types:Option<Vec<String>>,
 pub registry_json:Option<String>,
}
pub(crate) fn run(input:Input)->Result<Value,String>{
 run_internal(input,None)
}
pub(crate) fn resume(input:Input,saved:Value)->Result<Value,String>{
 run_internal(input,Some(saved))
}
fn run_internal(input:Input,saved:Option<Value>)->Result<Value,String>{
 if input.batches.is_empty()||input.batches.len()>10||input.created.trim().is_empty()||input.created.len()>100{return Err("Select 1–10 supplied years and record a bounded analysis timestamp.".into());}
 let types=crate::analysis_package::selected_types(input.types)?;
 let mut label=None;let mut source_bytes=0usize;
 // Check all year/label identities and source bounds before executing any year.
 for(year,batch)in &input.batches{
  if batch.build_index_json.len()>1024*1024||batch.states.len()>60{return Err("Each analysis year accepts a build index up to 1 MiB and at most 60 states.".into());}
  let build=crate::label_analysis::source_index(&batch.build_index_json,batch.external_import.is_some())?;
  bisect_report::label_rename::validate_name(&build.label)?;
  if year.len()!=4||!year.bytes().all(|c|c.is_ascii_digit())||&build.year!=year{return Err("Supplied analysis year does not match its original build index.".into());}
  if label.as_ref().is_some_and(|v|v!=&build.label){return Err("All analysis years must belong to the same label.".into());}
  label=Some(build.label);
  if batch.registry_json.is_some(){return Err("Supply one shared registry for multi-year analysis.".into());}
  source_bytes+=batch.build_index_json.len();
  if let Some(proof)=&batch.external_import {source_bytes+=serde_json::to_string(&proof.input).map_err(|e|e.to_string())?.len();}
  for source in batch.states.values(){source_bytes+=source.document_json.len();for text in [&source.context_json,&source.geography_json,&source.demographic_csv,&source.urban_csv,&source.election_csv].into_iter().flatten(){source_bytes+=text.len();}if let Some(g)=&source.compactness{source_bytes+=g.unit_geometry_json.len()+g.crs_label.len()+g.population_centroids_csv.as_ref().map_or(0,String::len);}}
  if source_bytes>25*1024*1024{return Err("Combined multi-year analysis sources exceed 25 MiB.".into());}
 }
 if input.registry_json.as_ref().is_some_and(|s|s.len()>1024*1024){return Err("Shared analysis registry exceeds 1 MiB.".into());}
 let requested_years=input.batches.keys().cloned().collect::<Vec<_>>();
 let prefix=saved.as_ref().map(|s|crate::label_years_resume::prefix(s,"bisect-label-analysis-years-v1",label.as_deref().expect("nonempty batches"),&requested_years)).transpose()?.unwrap_or(0);
 let mut registry=input.registry_json;let mut years=Vec::new();let mut files=BTreeMap::<String,String>::new();let mut output_bytes=0usize;
 for(year,mut batch)in input.batches{
  batch.created=input.created.clone();batch.types=Some(types.clone());batch.registry_json=registry.clone();
  match crate::label_analysis::run_for_years(batch){
   Err(error)=>years.push(json!({"year":year,"status":"failed","error":error})),
   Ok(result)=>{
    let mut next=Vec::new();let mut bytes=0usize;
    for(path,text)in result["files"].as_object().ok_or("Missing native analysis files.")?{if path==".bisect"{continue;}let text=text.as_str().ok_or("Native analysis file must be text.")?;bytes+=text.len();next.push((path.clone(),text.to_string()));}
    if output_bytes+bytes>25*1024*1024{return Err("Combined multi-year analysis outputs exceed 25 MiB.".into());}
    output_bytes+=bytes;files.extend(next);
    if let Some(text)=result["registry_json"].as_str(){registry=Some(text.to_string());}
    years.push(json!({"year":year,"status":"completed","result":result}));
   }
  }
  let status=if years.len()<requested_years.len(){"in-progress"}else if years.iter().any(|y|y["status"]=="failed"){"partial-failure"}else{"completed"};
  let snapshot=checkpoint(label.as_deref().expect("nonempty batches"),&input.created,&types,&years,&registry,&files,&requested_years,status)?;
  if years.len()==prefix{crate::label_years_resume::verify(saved.as_ref().expect("resume prefix"),&snapshot)?;}
  if years.len()>=prefix{crate::progress::emit(&json!({"schema_version":"bisect-label-analysis-years-progress-v1","sequence":years.len(),"requested_years":requested_years,"resume_verified":saved.is_some(),"snapshot":snapshot}))?;}
 }
 let status=if years.iter().any(|y|y["status"]=="failed"){"partial-failure"}else{"completed"};
 checkpoint(label.as_deref().expect("nonempty batches"),&input.created,&types,&years,&registry,&files,&requested_years,status)
}
fn checkpoint(label:&str,created:&str,types:&[String],years:&[Value],registry:&Option<String>,original_files:&BTreeMap<String,String>,requested_years:&[String],status:&str)->Result<Value,String>{
 let mut files=original_files.clone();
 if let Some(text)=registry{if files.values().map(String::len).sum::<usize>()+text.len()>25*1024*1024{return Err("Multi-year analysis outputs and registry exceed 25 MiB.".into());}files.insert(".bisect".into(),text.clone());}
 let receipt=json!({"schema_version":"bisect-label-analysis-years-record-v1","label":label,"created":created,"status":status,"requested_years":requested_years,"types":types,"years":years.iter().map(|y|json!({"year":y["year"],"status":y["status"],"error":y["error"],"index_sha256":y["result"]["index_sha256"]})).collect::<Vec<_>>()});
 let receipt=serde_json::to_string_pretty(&receipt).map_err(|e|e.to_string())?;
 if files.values().map(String::len).sum::<usize>()+receipt.len()>25*1024*1024{return Err("Multi-year analysis outputs and receipt exceed 25 MiB.".into());}
 files.insert(format!("analysis/{}/years.json",label),receipt);
 let hashes:BTreeMap<_,_>=files.iter().map(|(p,t)|(p.clone(),format!("{:x}",Sha256::digest(t.as_bytes())))).collect();
 Ok(json!({"schema_version":"bisect-label-analysis-years-v1","label":label,"status":status,"created":created,"requested_years":requested_years,"types":types,"years":years,"files":files,"file_sha256":hashes,"registry_json":registry,"scope":"Executes supplied label years in sorted order through the native single-year analyzers. Every original build index keeps its own exact-byte chain hash. Failed years retain errors and earlier completed output; individual state/analyzer outcomes remain authoritative. Shared registry stages carry forward only from completed native year requests. No filesystem installation, automatic execution on Open or original national capacity claim."}))
}
