//! Native multi-year report orchestration with explicit shared registry snapshots.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Batch{pub analysis_index_json:String,pub files:BTreeMap<String,String>,pub file_sha256:BTreeMap<String,String>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input{pub batches:BTreeMap<String,Batch>,pub registry_json:Option<String>,pub generated_at:String,pub format:String,pub audit_with_report:bool}
fn sha(text:&str)->String{format!("{:x}",Sha256::digest(text.as_bytes()))}
pub(crate) fn run(input:Input)->Result<Value,String>{
 run_internal(input,None)
}
pub(crate) fn resume(input:Input,saved:Value)->Result<Value,String>{
 run_internal(input,Some(saved))
}
fn run_internal(input:Input,saved:Option<Value>)->Result<Value,String>{
 if input.batches.is_empty()||input.batches.len()>10||input.generated_at.trim().is_empty()||input.generated_at.len()>100||!["html","json"].contains(&input.format.as_str())||input.registry_json.as_ref().is_some_and(|s|s.len()>1024*1024){return Err("Select 1–10 report years, a native format, bounded timestamp and registry.".into());}
 let requested_years=input.batches.keys().cloned().collect::<Vec<_>>();let mut label=None;let mut bytes=0usize;
 for(year,batch)in &input.batches{
  if batch.analysis_index_json.len()>1024*1024||batch.files.len()>842||batch.files.contains_key(".bisect"){return Err("Each report year accepts a bounded analysis index/files and one separate shared registry.".into());}
  let index:bisect_report::label_indexes::AnalysisIndex=serde_json::from_str(&batch.analysis_index_json).map_err(|e|e.to_string())?;
  bisect_report::label_indexes::serialize_analysis_index(&index)?;
  if &index.year!=year||label.as_ref().is_some_and(|v|v!=&index.label){return Err("Report years must match their original analysis indexes and one label.".into());}label=Some(index.label);
  bytes+=batch.analysis_index_json.len()+batch.files.values().map(String::len).sum::<usize>();if bytes>25*1024*1024{return Err("Combined report sources exceed 25 MiB.".into());}
 }
 let label=label.expect("nonempty batches");let mut registry=input.registry_json;let mut years=Vec::new();let mut files=BTreeMap::<String,String>::new();
 let prefix=saved.as_ref().map(|s|crate::label_years_resume::prefix(s,"bisect-label-report-years-v1",&label,&requested_years)).transpose()?.unwrap_or(0);
 for(year,batch)in input.batches{
  let request=crate::label_report::Input{analysis_index_json:batch.analysis_index_json,files:batch.files,file_sha256:batch.file_sha256,registry_json:registry.clone(),generated_at:input.generated_at.clone(),format:input.format.clone(),audit_with_report:input.audit_with_report};
  match crate::label_report::run(request){
   Err(error)=>years.push(json!({"year":year,"status":"failed","error":error})),
   Ok(result)=>{for(path,text)in result["files"].as_object().ok_or("Missing native report files.")?{if path!=".bisect"{files.insert(path.clone(),text.as_str().ok_or("Native report file must be text.")?.into());}}if let Some(text)=result["registry_json"].as_str(){registry=Some(text.into());}years.push(json!({"year":year,"status":"completed","result":result}));}
  }
  let status=if years.len()<requested_years.len(){"in-progress"}else if years.iter().any(|y|y["status"]=="failed"){"partial-failure"}else{"completed"};
  let snapshot=checkpoint(&label,&input.generated_at,&input.format,input.audit_with_report,&requested_years,&years,&files,&registry,status)?;
  if years.len()==prefix{crate::label_years_resume::verify(saved.as_ref().expect("resume prefix"),&snapshot)?;}
  if years.len()>=prefix{crate::progress::emit(&json!({"schema_version":"bisect-label-report-years-progress-v1","sequence":years.len(),"requested_years":requested_years,"resume_verified":saved.is_some(),"snapshot":snapshot}))?;}
 }
 let status=if years.iter().any(|y|y["status"]=="failed"){"partial-failure"}else{"completed"};
 checkpoint(&label,&input.generated_at,&input.format,input.audit_with_report,&requested_years,&years,&files,&registry,status)
}
fn checkpoint(label:&str,generated_at:&str,format:&str,audit:bool,requested:&[String],years:&[Value],original_files:&BTreeMap<String,String>,registry:&Option<String>,status:&str)->Result<Value,String>{
 let mut files=original_files.clone();if let Some(text)=registry{files.insert(".bisect".into(),text.clone());}
 let record=json!({"schema_version":"bisect-label-report-years-record-v1","label":label,"generated_at":generated_at,"format":format,"audit_with_report":audit,"status":status,"requested_years":requested,"years":years.iter().map(|row|json!({"year":row["year"],"status":row["status"],"error":row["error"],"index_sha256":row["result"]["index_sha256"]})).collect::<Vec<_>>()});
 files.insert(format!("reports/{label}/years.json"),serde_json::to_string_pretty(&record).map_err(|e|e.to_string())?);
 if files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Combined report outputs exceed 25 MiB.".into());}
 let hashes:BTreeMap<_,_>=files.iter().map(|(p,t)|(p.clone(),sha(t))).collect();
 Ok(json!({"schema_version":"bisect-label-report-years-v1","label":label,"generated_at":generated_at,"format":format,"audit_with_report":audit,"status":status,"requested_years":requested,"years":years,"files":files,"file_sha256":hashes,"registry_json":registry,"scope":"Runs supplied label years through shared native commission report assembly in sorted order. Each year binds its exact raw analysis-index bytes and verifies every supplied file digest. Failed years retain errors and other completed outputs. Registry stages carry forward only from completed year requests, including native partial outcomes; individual state availability remains authoritative. Checkpoints expose only completed year requests and preserve requested/received years in the hashed receipt. No disk installation, automatic execution on Open, PDF generation or all-original-state capacity claim."}))
}
