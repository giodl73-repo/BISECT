use base64::{Engine,engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{json,Value};
use std::collections::BTreeMap;
use sha2::{Digest,Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {pub label:String,pub state_identifier:String,pub source_dir_name:String,pub created_at:String}

pub(crate) fn migrate(files:BTreeMap<String,String>,s:Settings)->Result<Value,String>{
 let safe=|p:&str| !p.is_empty()&&p.len()<=200&&p.bytes().all(|b|b.is_ascii_alphanumeric()||b"_./ -".contains(&b))&&!p.split('/').any(|x|x.is_empty()||[".","..","__proto__","constructor","prototype"].contains(&x));
 if s.label.is_empty()||s.label.len()>100||!s.label.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))||["__proto__","constructor","prototype"].contains(&s.label.as_str())||!safe(&s.source_dir_name)||s.source_dir_name.contains('/')||!safe(&s.state_identifier)||s.state_identifier.contains('/')||s.created_at.is_empty()||s.created_at.len()>100{return Err("Supply a safe label, state identifier, source directory name and recorded timestamp.".into());}
 if files.len()>256||files.values().map(String::len).sum::<usize>()>24*1024*1024{return Err("Migration supports up to 256 files totaling 16 MiB.".into());}
 let mut decoded=BTreeMap::new();let mut folded=std::collections::BTreeSet::new();let mut total=0;
 for(p,b)in &files{if !safe(p)||p.len()+s.label.len()+7>240||!folded.insert(p.to_ascii_lowercase())||(["manifest.json","manifest.tmp","final_assignments.json","analysis","maps","intermediate"].contains(&p.to_ascii_lowercase().as_str())&&!["manifest.json","manifest.tmp","final_assignments.json"].contains(&p.as_str()))||p.split('/').next().is_some_and(|x|["manifest.json","manifest.tmp","final_assignments.json"].contains(&x.to_ascii_lowercase().as_str())&&p.contains('/')){return Err("Unsafe, case-colliding or reserved migration file path.".into());}
 let bytes=STANDARD.decode(b).map_err(|_|"Invalid migration file base64.")?;total+=bytes.len();if total>16*1024*1024{return Err("Migration files exceed 16 MiB.".into());}decoded.insert(p.clone(),bytes);}
 for p in decoded.keys(){let mut parts=p.split('/').collect::<Vec<_>>();parts.pop();while !parts.is_empty(){if folded.contains(&parts.join("/").to_ascii_lowercase()){return Err("Migration path is both a file and a directory.".into());}parts.pop();}}
 let assignments=decoded.get("final_assignments.json").map(Vec::as_slice);
 let (_,assignment_status)=bisect_report::legacy_migration::district_count(assignments);
 let manifest=bisect_report::legacy_migration::build_manifest(&s.label,&s.state_identifier,&s.source_dir_name,assignments,s.created_at);
 let manifest_json=serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?;
 let hashes:BTreeMap<_,_>=decoded.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect();
 let replaced:Vec<_>=["manifest.json","manifest.tmp"].into_iter().filter(|p|decoded.contains_key(*p)).collect();
 Ok(json!({"schema_version":"bisect-legacy-migration-v1","status":"packaged","manifest":manifest,"manifest_json":manifest_json,"assignment_status":assignment_status,"source_dir_name":s.source_dir_name,"source_sha256":hashes,"replaced_sources":replaced,"required_directories":["analysis/","maps/","intermediate/"],"scope":"Shared native migration manifest builder. Copies supplied file bytes into plans/LABEL and replaces root manifest.json/manifest.tmp with a minimal native manifest. District count uses distinct unsigned 64-bit assignment IDs; missing, empty or invalid assignments produce the native one-district fallback. State code is the supplied identifier uppercased; year, provenance hashes and legal/balance findings are not inferred. Native placeholder release and TIGER URLs are retained as metadata and never fetched. This packages a selected legacy directory; it does not discover state folders, run the engine, validate a plan, certify provenance or overwrite local files. Browser inputs and results remain inert until an explicit Run."}))
}
