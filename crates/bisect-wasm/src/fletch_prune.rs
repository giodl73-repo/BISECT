use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Serialize)]
struct Candidate {relative_path:String,absolute_path:String,bytes:u64,reason:String}
#[derive(Serialize)]
struct Plan {schema_version:String,generated_by:String,cache_root:String,object_root:String,destructive:bool,keep_count:usize,keep_bytes:u64,prune_count:usize,prune_bytes:u64,candidates:Vec<Candidate>}
fn join(root:&str,path:&str)->String {let root=root.replace('\\',"/");let absolute=root.starts_with('/');let root=root.trim_end_matches('/');if root.is_empty(){if absolute{format!("/{path}")}else{path.into()}}else{format!("{root}/{path}")}}
/// Pinned upstream plan_cache_prune projection over caller-selected regular
/// object inventory. Portable path spellings never become disk destinations.
pub(crate) fn plan(text:String,files:BTreeMap<String,String>)->Result<Value,String>{
 if text.len()>16*1024*1024||files.len()>10000{return Err("Cache prune input exceeds manifest/inventory bounds".into());}
 let manifest=bisect_core::fletch_cache::read_cache_manifest_text(&text)?;
 if manifest.entries.len()>2000{return Err("Cache prune supports up to 2000 manifest entries".into());}
 let mut referenced=BTreeSet::new();let mut keep_bytes=0u64;
 for entry in &manifest.entries {referenced.insert(crate::fletch_verify::relative_path(&entry.relative_path)?);keep_bytes=keep_bytes.checked_add(entry.bytes).ok_or("Cache keep-byte total exceeds u64")?;}
 let mut seen=BTreeSet::new();let mut candidates=Vec::new();let mut prune_bytes=0u64;
 for (path,size) in files {
  let path=crate::fletch_verify::relative_path(&path)?;if !seen.insert(path.clone()){return Err("Duplicate normalized inventory path".into());}
  if size.is_empty()||size.len()>20||(size.len()>1&&size.starts_with('0'))||!size.bytes().all(|b|b.is_ascii_digit()){return Err("Inventory byte count must be an exact canonical u64 string".into());}
  let bytes=size.parse::<u64>().map_err(|_|"Inventory byte count exceeds u64")?;
  if !path.starts_with("objects/")||referenced.contains(&path){continue;}
  prune_bytes=prune_bytes.checked_add(bytes).ok_or("Cache prune-byte total exceeds u64")?;
  candidates.push(Candidate{absolute_path:join(&manifest.cache_root,&path),relative_path:path,bytes,reason:"unreferenced-cache-object".into()});
 }
 candidates.sort_by(|a,b|a.relative_path.cmp(&b.relative_path));
 let rows=candidates.iter().map(|c|json!({"relative_path":c.relative_path,"byte_count_text":c.bytes.to_string(),"reason":c.reason})).collect::<Vec<_>>();
 let plan=Plan{schema_version:"fletch.cache-prune.v1".into(),generated_by:"fletch-core/0.1.0".into(),object_root:join(&manifest.cache_root,"objects"),cache_root:manifest.cache_root,destructive:false,keep_count:referenced.len(),keep_bytes,prune_count:candidates.len(),prune_bytes,candidates};
 Ok(json!({"schema_version":"bisect-fletch-cache-prune-v1","manifest_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"plan_json":serde_json::to_string_pretty(&plan).map_err(|e|e.to_string())?,"prune_count":plan.prune_count,"prune_bytes_text":prune_bytes.to_string(),"keep_count":plan.keep_count,"keep_bytes_text":keep_bytes.to_string(),"candidates":rows,"scope":"Native orphan-object prune planning over selected regular-file inventory. Paths use portable slash spellings and remain inert identifiers; byte counts are observed inventory sizes, not fresh hashes. This preview does not delete files or manifest entries."}))
}
