use serde_json::{json,Value};
use std::collections::BTreeMap;
use sha2::{Digest,Sha256};
#[derive(serde::Deserialize)]#[serde(deny_unknown_fields)]pub(crate) struct FileDigest{pub sha256:Option<String>,pub read_error:Option<String>}
fn safe_path(p:&str)->bool{!p.is_empty()&&p.len()<=1024&&!p.contains(['\\',':','\0'])&&!p.starts_with('/')&&p.split('/').all(|s|!s.is_empty()&&!matches!(s,"."|".."|"__proto__"|"constructor"|"prototype"))}
pub(crate) fn inspect(tutorial:&str,source:&str,text:&str,files:BTreeMap<String,FileDigest>)->Result<Value,String>{
 if tutorial.trim().is_empty()||tutorial.len()>200||source.is_empty()||source.len()>200||text.len()>512*1024||files.len()>2000{return Err("Supply tutorial/name up to 200 UTF-8 bytes, checksum JSON up to 512 KiB and at most 2,000 file digests.".into());}
 for (p,d) in &files{if !safe_path(p)||d.sha256.is_some()==d.read_error.is_some()||d.sha256.as_ref().is_some_and(|s|s.len()!=64||!s.bytes().all(|b|b.is_ascii_hexdigit()))||d.read_error.as_ref().is_some_and(|e|e.len()>2000){return Err("Invalid local file path or digest/read-error record.".into());}}
 let parsed:Result<bisect_analysis::tutorial_checksums::TutorialChecksums,_>=serde_json::from_str(text);
 let mut report=match parsed{Err(e)=>json!({"native_exit_code":2,"status":"invalid-checksums","error":e.to_string()}),Ok(c)=>{
  if c.schema_version!="tutorial-checksums v1"{json!({"native_exit_code":2,"status":"invalid-schema","error":format!("Unexpected schema_version: '{}'",c.schema_version)})}
  else{if c.pinned_inputs.len()+c.expected_outputs.len()>2000||c.pinned_inputs.iter().any(|i|!safe_path(&i.local_path))||c.expected_outputs.iter().any(|o|!safe_path(&o.path)){return Err("Checksum document needs at most 2,000 safe relative file references.".into());}
   let mut r=bisect_analysis::tutorial_checksums::assess(&c,tutorial,|p|files.get(p).map(|d|match &d.sha256{Some(s)=>Ok(s.clone()),None=>Err(d.read_error.clone().unwrap_or_default())}));
   r["status"]=json!(if r["fail"].as_u64().unwrap_or(0)>0{"drift-detected"}else if r["missing"].as_u64().unwrap_or(0)>0{"files-missing"}else{"hashes-match"});r
  }
 }};
 report["schema_version"]=json!("bisect-tutorial-checks-v1");report["tutorial"]=json!(tutorial);report["source_name"]=json!(source);report["source_sha256"]=json!(format!("sha256:{:x}",Sha256::digest(text.as_bytes())));report["supplied_paths"]=json!(files.keys().collect::<Vec<_>>());
 report["scope"]=json!("Native tutorial checksum schema and case-insensitive SHA-256 comparison. The browser worker streams each explicitly selected file through Rust SHA-256 using bounded chunks. Missing files retain native exit code 0 but remain visibly missing. URLs and checksum paths are reference labels only; nothing is fetched or opened by path. Tract counts are informational as in the CLI and are not recomputed. Hash matching checks supplied bytes against supplied pins; it does not authenticate the pins, execution, data source or legal compliance. Imported digest/report claims are unverified; reopening never rehashes files. Select local files and run again for fresh verification.");Ok(report)
}
