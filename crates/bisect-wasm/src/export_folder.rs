use base64::{engine::general_purpose::STANDARD,Engine};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct Input{pub label:String,pub files:BTreeMap<String,String>,pub formats:Vec<String>}
fn sha(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
pub fn export(input:Input)->Result<Value,String>{
 if input.label.is_empty()||input.label.len()>100||!input.label.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||input.files.len()>504||input.formats.is_empty()||input.formats.len()>8||input.formats.iter().any(|f|!["csv","geojson","gerrychain","reproducibility-package"].contains(&f.as_str())){return Err("Select a portable label, 1–8 native export formats and up to 504 source files.".into());}
 let mut sources=BTreeMap::new();let mut hashes=BTreeMap::new();let mut total=0;
 for(path,encoded)in &input.files{
  let analysis=path.strip_prefix("analysis/").is_some_and(|p|!p.starts_with('.')&&p.len()<=120&&p.ends_with(".json")&&p.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-'||b==b'.'));
  if path!=&format!("{}.rplan",input.label)&&!["final_assignments.json","data/final_assignments.json","manifest.json"].contains(&path.as_str())&&!analysis{return Err("Select native labeled RPLAN, assignments, manifest or plain analysis JSON sources.".into());}
  if encoded.len()>(3*1024*1024usize).div_ceil(3)*4{return Err("Each selected native export source is limited to 3 MiB.".into());}
  let bytes=STANDARD.decode(encoded).map_err(|_|"Invalid export source base64")?;
  if bytes.len()>3*1024*1024||STANDARD.encode(&bytes)!=*encoded{return Err("Invalid canonical export source bytes.".into());}
  total+=bytes.len();if total>25*1024*1024{return Err("Native export sources exceed 25 MiB.".into());}
  hashes.insert(path.clone(),sha(&bytes));sources.insert(path.clone(),bytes);
 }
 let selected=bisect_report::export_source::source_path(&input.label,|p|sources.contains_key(p)).ok_or("No labeled .rplan or final_assignments.json selected")?;
 let text=|path:&str|->Result<Option<&str>,String>{sources.get(path).map(|b|std::str::from_utf8(b).map_err(|_|format!("Selected export source {path} is not UTF-8"))).transpose()};
 let rplan_selected=selected==format!("{}.rplan",input.label);
 let source=text(&selected)?.ok_or("Selected export source is unavailable")?;
 if rplan_selected&&serde_json::from_str::<Value>(source).map_err(|e|e.to_string())?["rplan_version"]=="0.2" {crate::plan_audit_text::plan(source)?;}
 let manifest=if rplan_selected{None}else{text("manifest.json")?};
 let plan=bisect_report::export_source::load_portable(&input.label,if rplan_selected{Some(source)}else{None},if rplan_selected{None}else{Some(source)},manifest)?;
 if plan.assignments.len()>100000{return Err("Native folder export accepts up to 100,000 assignments.".into());}
 let document_json=serde_json::to_string_pretty(&plan).map_err(|e|e.to_string())?;
 let mut files=BTreeMap::<String,String>::new();let mut stages=Vec::new();let mut status="completed";
 for format in &input.formats{
  let next=(||->Result<BTreeMap<String,String>,String>{
   let result=match format.as_str(){
    "csv"=>BTreeMap::from([(format!("{}_tracts.csv",input.label),bisect_report::export_tracts_csv(&plan).map_err(|e|e.to_string())?)]),
    "gerrychain"=>BTreeMap::from([(format!("{}_gerrychain.json",input.label),bisect_report::export_gerrychain_v23(&plan).map_err(|e|e.to_string())?)]),
    "geojson"=>BTreeMap::from([(format!("{}.geojson",input.label),bisect_report::export_geojson(&plan).map_err(|e|e.to_string())?)]),
    "reproducibility-package"=>{
     let analysis=sources.keys().filter(|p|p.starts_with("analysis/")).map(|p|Ok((p[9..].to_string(),text(p)?.ok_or("Missing analysis bytes")?.to_string()))).collect::<Result<BTreeMap<_,_>,String>>()?;
     bisect_report::reproducibility::build_reproducibility_package(&input.label,&plan,text("manifest.json")?,&analysis).map_err(|e|e.to_string())?.into_iter().map(|(path,text)|(format!("{}_reproducibility/{path}",input.label),text)).collect()
    },_=>unreachable!()
   };Ok(result)
  })();
  match next{
   Ok(next)=>{let mut staged=files.clone();staged.extend(next);if staged.values().map(String::len).sum::<usize>()>64*1024*1024{status="partial";stages.push(json!({"format":format,"status":"failed","error":"Export outputs exceed 64 MiB; earlier formats retained"}));break;}files=staged;stages.push(json!({"format":format,"status":"completed"}));},
   Err(error)=>{status="partial";stages.push(json!({"format":format,"status":"failed","error":error}));break;}
  }
 }
 // Keep the existing number for safe values; larger counts cross JavaScript as decimal text.
 let count=plan.metadata.num_districts;
 let declared_districts=if count <= 9_007_199_254_740_991 {json!(count)} else {json!(count.to_string())};
 let file_sha256:BTreeMap<_,_>=files.iter().map(|(p,t)|(p.clone(),sha(t.as_bytes()))).collect();
 Ok(json!({"schema_version":"bisect-native-folder-export-v1","status":status,"label":input.label,"selected_source":selected,"manifest_used_for_reconstruction":manifest.is_some(),"document_json":document_json,"document_sha256":sha(document_json.as_bytes()),"source_sha256":hashes,"files":files,"file_sha256":file_sha256,"stages":stages,"units":plan.assignments.len(),"declared_districts":declared_districts,"geometry_present":plan.geometry.is_some(),"execution_performed":false,"scope":"Shared native Export source precedence: labeled RPLAN, root assignments, then data assignments. Assignment fallback uses supplied native manifest metadata or native empty defaults; no district count, geometry or provenance is inferred. Native CSV/GerryChain/GeoJSON/reproducibility builders preserve legacy export semantics, including null geometry and metadata claims. Formats execute in requested order; first failure retains earlier completed files. This is interchange export, not plan validation, engine replay, source authentication or certification. No filesystem discovery, subprocess or verification script execution. Browser file and record bounds apply. Legacy count and district ID values are preserved in the native unsigned 64-bit range."}))
}
