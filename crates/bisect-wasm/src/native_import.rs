use base64::{engine::general_purpose::STANDARD,Engine};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct Input{pub source_base64:String,pub settings:bisect_report::import_source::Settings}
fn sha(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
pub fn import(input:Input)->Result<Value,String>{
 let s=input.settings;
 if s.label.is_empty()||s.label.len()>100||!s.label.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||s.state.len()!=2||!s.state.bytes().all(|b|b.is_ascii_alphabetic())||s.year.len()!=4||!s.year.bytes().all(|b|b.is_ascii_digit())||s.format.len()>50||s.source_name.is_empty()||s.source_name.len()>200||s.source_name.contains(['/', '\\'])||s.source_name.chars().any(char::is_control)||s.created_at.trim().is_empty()||s.created_at.len()>200||s.submitted_by.as_ref().is_some_and(|v|v.len()>200)||s.submitted_at.as_ref().is_some_and(|v|v.len()>200){return Err("Select a portable label, two-letter state, four-digit year, named source and bounded import metadata.".into());}
 let extension=std::path::Path::new(&s.source_name).extension().and_then(|e|e.to_str());
 if extension.is_some_and(|e|["shp","shx","dbf","prj","zip"].contains(&e)){return Err("Native plan import does not directly support shapefile-family or ZIP input. Convert the plan to GeoJSON first.".into());}
 if input.source_base64.len()>4*1024*1024{return Err("Native import source exceeds 3 MiB.".into());}
 let bytes=STANDARD.decode(&input.source_base64).map_err(|_|"Invalid native import base64")?;
 if bytes.len()>3*1024*1024||STANDARD.encode(&bytes)!=input.source_base64{return Err("Invalid canonical native import source.".into());}
 let text=std::str::from_utf8(&bytes).map_err(|_|"Native import source is not UTF-8")?;
 let assignments=bisect_report::import_source::parse(&s.format,&s.source_name,text).map_err(|e|e.to_string())?;
 if assignments.len()>100000{return Err("Native import accepts up to 100,000 tract assignments.".into());}
 if assignments.values().any(|&district|district>u32::MAX as usize){return Err("Native import district IDs must fit the wasm32 unsigned range.".into());}
 bisect_report::validate_geoid_format_batch(&assignments).map_err(|e|format!("GEOID validation failed: {e}"))?;
 let plan=bisect_report::import_source::legacy(&s,assignments);
 let mut files=bisect_report::import_source::legacy_files(&s.label,&plan).map_err(|e|e.to_string())?;
 let (status,error)=match bisect_report::import_source::manifest(&s,plan.metadata.num_districts,&s.created_at,&s.created_at){
  Ok(manifest)=>{files.insert("manifest.json".into(),serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?);("completed",None)},
  Err(error)=>("partial",Some(error.to_string()))
 };
 let hashes:std::collections::BTreeMap<_,_>=files.iter().map(|(p,t)|(p.clone(),sha(t.as_bytes()))).collect();
 Ok(json!({"schema_version":"bisect-native-plan-import-v1","status":status,"manifest_error":error,"label":s.label,"state":plan.metadata.state_code,"year":s.year,"parser_used":bisect_report::import_source::parser_kind(&s.format,&s.source_name),"declared_format":s.format,"source_name":s.source_name,"source_sha256":sha(&bytes),"files":files,"file_sha256":hashes,"units":plan.assignments.len(),"districts":plan.metadata.num_districts,"execution_performed":false,"scope":"Shared native CLI plan import: explicit GerryChain/DRA dispatch, otherwise lowercase .csv filename dispatch and GeoJSON-property fallback. Native legacy RPLAN, sorted compatibility assignments and typed manifest retain native FIPS, empty/default metadata, source-tool declaration and civic timestamp default. Missing civic submitter retains earlier RPLAN/assignments without a manifest, matching the native failure boundary. Host-supplied recorded time replaces the native clock. Native import's authoritative submission_type is a source declaration, not authenticated official authority. No modern context alignment, polygon/graph inference, engine replay, population/contiguity/legal audit, filesystem write or automatic execution on Open."}))
}
