use base64::{engine::general_purpose::STANDARD,Engine};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Deserialize)]#[serde(deny_unknown_fields)]
pub struct Selection{pub label:String,pub state_identifier:String,pub force:bool,pub observation_json:String,pub manifest_json:Option<String>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct Input{pub selection:Selection,pub files_base64:BTreeMap<String,String>,pub created_at:String}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct Observation{directories:Vec<String>,files:Vec<String>}
fn sha(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn safe(p:&str)->bool{!p.is_empty()&&p.len()<=200&&p.bytes().all(|b|b.is_ascii_alphanumeric()||b"_./ -".contains(&b))&&p.split('/').all(|s|!s.is_empty()&&![".","..","__proto__","constructor","prototype"].contains(&s))}
struct Chosen{observation:Observation,source:String,target:String,source_files:Vec<String>,target_files:Vec<String>,manifest_exists:bool}
fn choose(s:&Selection)->Result<Chosen,String>{
 if !safe(&s.state_identifier)||s.state_identifier.contains('/')||s.label.is_empty()||s.label.len()>100||!s.label.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||["__proto__","constructor","prototype"].contains(&s.label.as_str())||s.observation_json.len()>2*1024*1024{return Err("Select a portable state identifier, label and bounded base observation.".into());}
 let observation:Observation=serde_json::from_str(&s.observation_json).map_err(|e|format!("Migration observations: {e}"))?;
 if observation.files.len()+observation.directories.len()>10000{return Err("Migration base inventory exceeds 10,000 entries.".into());}
 let mut all=BTreeMap::new();for(path,kind)in observation.directories.iter().map(|p|(p,true)).chain(observation.files.iter().map(|p|(p,false))){if !safe(path)||all.insert(path.to_ascii_lowercase(),kind).is_some(){return Err("Unsafe, duplicate or case-colliding base observation.".into());}}
 for path in all.keys(){let mut parts=path.split('/').collect::<Vec<_>>();parts.pop();while !parts.is_empty(){if all.get(&parts.join("/"))!=Some(&true){return Err("Base observations require every parent directory and no file/directory conflicts.".into());}parts.pop();}}
 let manifest=s.manifest_json.as_deref().unwrap_or(include_str!("../../../data/manifest.json"));if manifest.len()>2*1024*1024{return Err("Migration resolution manifest exceeds 2 MiB.".into());}
 let resolve=||serde_json::from_str::<bisect_core::fetch_plan::ManifestWithDistricts<u64>>(manifest).ok().and_then(|m|m.states.get(&s.state_identifier.to_uppercase()).map(|state|state.name.to_lowercase().replace(' ',"_")));
 let source=bisect_report::legacy_migration::select_source(&s.state_identifier,|p|all.contains_key(&p.to_ascii_lowercase()),resolve).ok_or("Legacy state directory not found in selected base observations")?;
 if !safe(&source)||all.get(&source.to_ascii_lowercase())!=Some(&true){return Err("Selected legacy source is not a portable directory.".into());}
 let target=format!("plans/{}",s.label);if all.get("plans")==Some(&false)||all.get(&target.to_ascii_lowercase())==Some(&false){return Err("Native migration target or plans parent is a file, not a directory.".into());}
 let below=|p:&str,root:&str|p.to_ascii_lowercase().starts_with(&(root.to_ascii_lowercase()+"/"));
 let source_files=observation.files.iter().filter(|p|below(p,&source)).cloned().collect::<Vec<_>>();let target_files=observation.files.iter().filter(|p|below(p,&target)).cloned().collect::<Vec<_>>();
 if source_files.len()+target_files.len()>512{return Err("Selected migration source and target exceed 512 files.".into());}
 let manifest_exists=all.contains_key(&(target.to_ascii_lowercase()+"/manifest.json"));
 Ok(Chosen{observation,source,target,source_files,target_files,manifest_exists})
}
pub fn inspect(s:Selection)->Result<Value,String>{let c=choose(&s)?;Ok(json!({"schema_version":"bisect-native-migration-selection-v1","label":s.label,"state_identifier":s.state_identifier,"force":s.force,"source_path":c.source,"target_path":c.target,"source_files":c.source_files,"target_files":c.target_files,"target_manifest_exists":c.manifest_exists,"observation_sha256":sha(s.observation_json.as_bytes()),"manifest_sha256":sha(s.manifest_json.as_deref().unwrap_or(include_str!("../../../data/manifest.json")).as_bytes()),"execution_performed":false,"scope":"Shared native direct legacy folder then manifest state-name resolution, against selected portable observations. Directory entries include empty folders. No file bytes read, migration staged or filesystem modified."}))}
pub fn stage(input:Input)->Result<Value,String>{
 let s=&input.selection;let c=choose(s)?;if input.created_at.trim().is_empty()||input.created_at.len()>100||input.files_base64.len()>512{return Err("Supply a bounded recorded migration timestamp and selected bytes.".into());}
 let mut decoded=BTreeMap::new();let mut total=0usize;
 for(path,b64)in &input.files_base64{
  if !c.source_files.contains(path)&&!c.target_files.contains(path)||b64.len()>((32*1024*1024usize).div_ceil(3)*4){return Err("Migration bytes must name selected source/target files only.".into());}
  let bytes=STANDARD.decode(b64).map_err(|_|"Invalid migration source base64")?;total+=bytes.len();if total>32*1024*1024||STANDARD.encode(&bytes)!=*b64{return Err("Migration bytes exceed 32 MiB or are not canonical base64.".into());}decoded.insert(path.clone(),bytes);
 }
 let target_manifest=c.target_files.iter().find(|p|p.to_ascii_lowercase()==c.target.to_ascii_lowercase()+"/manifest.json");
 let collision_text=target_manifest.and_then(|p|decoded.get(p)).and_then(|b|std::str::from_utf8(b).ok());
 bisect_report::run_target::check_plan_collision_evidence(std::path::Path::new(&c.target),s.force,c.manifest_exists,collision_text).map_err(|e|e.to_string())?;
 if decoded.len()!=c.source_files.len()+c.target_files.len(){return Err("Supply every selected migration source and existing target file.".into());}
 for(path,root)in c.source_files.iter().map(|p|(p,&c.source)).chain(c.target_files.iter().map(|p|(p,&c.target))){let relative=&path[root.len()+1..];let first=relative.split('/').next().unwrap();if ["manifest.json","manifest.tmp","final_assignments.json","analysis","maps","intermediate"].contains(&first.to_ascii_lowercase().as_str())&&first!=first.to_ascii_lowercase(){return Err("Native reserved migration paths require canonical case.".into());}}
 let mut files=BTreeMap::<String,Vec<u8>>::new();
 for path in &c.target_files{files.insert(format!("{}/{}",c.target,&path[c.target.len()+1..]),decoded[path].clone());}
 for path in &c.source_files{files.insert(format!("{}/{}",c.target,&path[c.source.len()+1..]),decoded[path].clone());}
 let mut folded=BTreeSet::new();for path in files.keys(){if !folded.insert(path.to_ascii_lowercase()){return Err("Migration output has case-colliding file names.".into());}}
 let mut dirs=BTreeSet::new();dirs.insert("plans".into());dirs.insert(c.target.clone());
 for path in &c.observation.directories{
  let relative=if path.to_ascii_lowercase().starts_with(&(c.source.to_ascii_lowercase()+"/")){Some(&path[c.source.len()+1..])}else if path.to_ascii_lowercase().starts_with(&(c.target.to_ascii_lowercase()+"/")){Some(&path[c.target.len()+1..])}else{None};
  if let Some(relative)=relative{let first=relative.split('/').next().unwrap();if ["manifest.json","manifest.tmp"].contains(&first.to_ascii_lowercase().as_str())||(["analysis","maps","intermediate","final_assignments.json"].contains(&first.to_ascii_lowercase().as_str())&&first!=first.to_ascii_lowercase()){return Err("Migration reserved manifest directory or noncanonical reserved directory case.".into());}dirs.insert(format!("{}/{relative}",c.target));}
 }
 for name in ["analysis","maps","intermediate"]{dirs.insert(format!("{}/{name}",c.target));}
 for path in files.keys(){let mut parts=path.split('/').collect::<Vec<_>>();parts.pop();while !parts.is_empty(){dirs.insert(parts.join("/"));parts.pop();}}
 let mut directory_case=BTreeSet::new();for path in &dirs{if !directory_case.insert(path.to_ascii_lowercase())||folded.contains(&path.to_ascii_lowercase()){return Err("Migration output has a directory case collision or file/directory conflict; no package staged.".into());}}
 let assignments=files.get(&(c.target.clone()+"/final_assignments.json")).map(Vec::as_slice);
 let source_name=c.source.rsplit('/').next().unwrap();let (_,assignment_status)=bisect_report::legacy_migration::district_count(assignments);
 let manifest=bisect_report::legacy_migration::build_manifest(&s.label,&s.state_identifier,source_name,assignments,input.created_at);
 let manifest_json=serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?;
 files.remove(&(c.target.clone()+"/manifest.tmp"));files.insert(c.target.clone()+"/manifest.json",manifest_json.as_bytes().to_vec());
 if files.len()+dirs.len()>2000||files.keys().chain(dirs.iter()).any(|p|p.len()>240){return Err("Migration ZIP inventory exceeds 2,000 entries or 240-byte paths.".into());}
 let files_base64:BTreeMap<_,_>=files.iter().map(|(p,b)|(p.clone(),STANDARD.encode(b))).collect();let file_sha256:BTreeMap<_,_>=files.iter().map(|(p,b)|(p.clone(),sha(b))).collect();let source_sha256:BTreeMap<_,_>=decoded.iter().map(|(p,b)|(p.clone(),sha(b))).collect();
 Ok(json!({"schema_version":"bisect-native-migration-package-v1","status":"staged","source_path":c.source,"target_path":c.target,"force":s.force,"manifest":manifest,"manifest_json":manifest_json,"assignment_status":assignment_status,"files_base64":files_base64,"file_sha256":file_sha256,"source_sha256":source_sha256,"directories":dirs,"observation_sha256":sha(s.observation_json.as_bytes()),"execution_performed":false,"scope":"Shared native legacy resolution, completed-manifest collision policy and migration metadata. Copies source over supplied target bytes, retaining unrelated target files and empty directories; replaces root manifest and removes native temporary manifest. Distinct u64 assignment IDs retain native one-district fallback semantics. Snapshot paths and provenance are owner observations, not authentication. Immutable staged package only: no local file modified, no engine run, audit or certification. Portable names, counts, 32 MiB bytes and file/directory conflict guards apply; native partial filesystem failure installation is not emulated."}))
}
