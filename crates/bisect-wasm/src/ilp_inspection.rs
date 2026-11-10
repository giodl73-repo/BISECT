//! Retain actual native node reports without native temporary-file access.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
use std::sync::Mutex;
use bisect_runner::bisection_runner::IlpNodeReport;
pub(crate) fn inspect(text:&str)->Result<Value,String>{
 if text.len()>8*1024*1024{return Err("ILP inspection source exceeds 8 MiB".into());}
 let request:crate::engine::Request=serde_json::from_str(text).map_err(|e|e.to_string())?;
 inspect_request(request,text)
}
fn inspect_request(request:crate::engine::Request,text:&str)->Result<Value,String>{
 if request.options.structure!="ilp"{return Err("ILP inspection requires native ILP structure".into());}
 let unit_ids=request.graph.geoids.clone();
 let records=Mutex::new((Vec::<IlpNodeReport>::new(),0usize,0usize,None::<String>));
 let observer=|node:IlpNodeReport|->Result<(),String>{
  let mut state=records.lock().map_err(|_|"ILP observation lock failed")?;
  if let Some(error)=&state.3{return Err(error.clone());}
  let bytes=node.report_json.as_ref().map_or(0,String::len)+node.lp_text.as_ref().map_or(0,String::len);
  if state.0.len()>=512||state.1.saturating_add(node.unit_indices.len())>2000000||state.2.saturating_add(bytes)>16*1024*1024{let error="ILP observation exceeds 512 nodes, two million unit references or 16 MiB of artifacts".to_string();state.3=Some(error.clone());return Err(error);}
  state.1+=node.unit_indices.len();state.2+=bytes;state.0.push(node);Ok(())
 };
 let outcome=crate::engine::execute_observed(request,Some(&observer));
 let (mut nodes,_,_,failure)=records.into_inner().map_err(|_|"ILP observation lock failed")?;
 if let Some(error)=failure{return Err(error);}
 if outcome.as_ref().err().is_some_and(|e|e.contains("ILP observation")){return Err(outcome.err().unwrap());}
 nodes.sort_by(|a,b|(a.depth,&a.path).cmp(&(b.depth,&b.path)));
 let mut files=BTreeMap::new();
 for node in &nodes{let name=if node.path.is_empty(){"root"}else{node.path.as_str()};let parent=format!("depth_{:02}/node_{name}",node.depth);if let Some(text)=&node.report_json{files.insert(format!("{parent}.json"),text);}if let Some(text)=&node.lp_text{files.insert(format!("{parent}.lp"),text);}}
 let file_hashes:BTreeMap<_,_>=files.iter().map(|(path,text)|(path,format!("{:x}",Sha256::digest(text.as_bytes())))).collect();
 let (result,error)=match outcome{Ok(mut v)=>{if v["metrics"]["structure_evidence"].is_object(){v["metrics"]["structure_evidence"]["report_delivery"]=json!("captured-in-inspection-packet");v["metrics"]["structure_evidence"]["native_solver_behavior"]=json!("shared-native-solver; per-node reports captured");}(Some(v),None)},Err(e)=>(None,Some(e))};
 Ok(json!({"schema_version":"bisect-native-ilp-inspection-v1","status":if error.is_some(){"failed"}else{"completed"},"error":error,"result":result,"source_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"unit_ids":unit_ids,"nodes":nodes,"files":files,"file_sha256":file_hashes,"scope":"Actual shared native dispatcher node observations, with local variable order mapped to original input indices. LP/report bytes and model hashes are retained without filesystem extraction. Failed dispatch can retain reports already produced; bounds reject the inspection instead of exporting a truncated packet. Local solver/proof status remains a native report claim and does not certify the whole plan, execution authenticity or global optimality. Skipped nodes have no fabricated solve report. Native node half-split scheduling and population clamping retain their existing semantics."}))
}

/// Capture the very dispatch returned to the caller, without executing it twice.
/// Population-retry callers retain only the packet belonging to their accepted result.
pub(crate) fn execute_captured(request:crate::engine::Request)->Result<Value,String>{
 let text=serde_json::to_string(&request).map_err(|e|e.to_string())?;
 if text.len()>8*1024*1024{return Err("ILP report capture request exceeds 8 MiB".into());}
 let mut packet=inspect_request(request,&text)?;
 if packet["status"]!="completed"{return Err(packet["error"].as_str().unwrap_or("Native ILP dispatch failed").to_string());}
 let mut result=packet.as_object_mut().ok_or("Invalid ILP inspection packet")?.remove("result").ok_or("Missing ILP inspection result")?;
 let assignments=serde_json::to_string(&result["assignments"]).map_err(|e|e.to_string())?;
 packet["schema_version"]=json!("bisect-native-ilp-run-reports-v1");
 packet["source_json"]=json!(text);
 packet["returned_assignments_json"]=json!(assignments);
 packet["returned_assignments_sha256"]=json!(format!("{:x}",Sha256::digest(assignments.as_bytes())));
 packet["accepted_attempt"]=Value::Null;
 result["ilp_report_capture"]=packet;
 Ok(result)
}
