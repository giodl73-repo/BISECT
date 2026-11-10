use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct Input{pub paths_json:String,pub year:String,pub states:Vec<String>,pub resolution:String,#[serde(default)]pub manifest_json:Option<String>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct Paths{paths:Vec<String>}
pub fn inspect(input:Input)->Result<Value,String>{
 if input.paths_json.len()>2*1024*1024||input.states.len()>100||input.states.iter().any(|s|s.len()!=2||!s.bytes().all(|c|c.is_ascii_alphabetic()))||!["tract","block_group","block-group"].contains(&input.resolution.as_str()){return Err("Select bounded native folder paths, state filters and tract/block-group resolution.".into());}
 let paths:Paths=serde_json::from_str(&input.paths_json).map_err(|e|format!("Folder paths: {e}"))?;
 if paths.paths.len()>10000||paths.paths.iter().any(|p|p.is_empty()||p.len()>300||p.split('/').any(|v|v.is_empty()||v=="."||v=="..")||!p.bytes().all(|c|c.is_ascii_alphanumeric()||b"/_-.".contains(&c))){return Err("Native folder paths require up to 10,000 portable relative entries.".into());}
 let observed:BTreeSet<_>=paths.paths.into_iter().collect();
 let text=input.manifest_json.as_deref().unwrap_or(include_str!("../../../data/manifest.json"));if text.len()>2*1024*1024{return Err("Manifest exceeds 2 MiB.".into());}
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(text).map_err(|e|e.to_string())?;
 if manifest.states.len()>51||manifest.states.iter().any(|(code,state)|code.len()!=2||!code.bytes().all(|c|c.is_ascii_uppercase())||state.districts.get(&input.year).is_some_and(|n|*n>500)){return Err("Native folder manifest exceeds catalog bounds.".into());}
 let all=bisect_core::state_catalog::congressional_states(&manifest,&input.year)?;let filters:BTreeSet<_>=input.states.iter().map(|s|s.to_uppercase()).collect();let known:BTreeSet<_>=all.iter().map(|s|s.0.clone()).collect();
 let mut rows=Vec::new();for (state,_,districts)in all.into_iter().filter(|s|filters.is_empty()||filters.contains(&s.0)){
  let selected=bisect_core::native_input_paths::select_adjacency_path(&state.to_lowercase(),&input.year,&input.resolution,|p|observed.contains(p));
  let binary=selected.path.as_ref().map(|p|format!("{}.adj.bin",p.strip_suffix(".pkl").unwrap()));
  let stem=binary.as_ref().map(|p|p.strip_suffix(".adj.bin").unwrap());
  let geoids=stem.map(|p|format!("{p}_geoids.json"));let centroids=stem.map(|p|format!("{p}_centroids.json"));
  let status=if selected.path.is_none(){"missing"}else if binary.as_ref().is_some_and(|p|observed.contains(p)){"binary-selected"}else{"requires-legacy-pickle-shim"};
  rows.push(json!({"state":state,"year":input.year,"districts":districts,"requested_resolution":input.resolution,"effective_resolution":selected.effective_resolution,"fallback_to_tract":selected.fallback_to_tract,"selected_path":selected.path,"binary_path":binary,"geoids_path":geoids.filter(|p|observed.contains(p)),"centroids_path":centroids.filter(|p|observed.contains(p)),"status":status}));
 }
 Ok(json!({"schema_version":"bisect-native-input-folder-v1","year":input.year,"resolution":input.resolution,"states":rows,"requested_states":&filters,"ignored_states":filters.difference(&known).collect::<Vec<_>>(),"paths_sha256":format!("{:x}",Sha256::digest(input.paths_json.as_bytes())),"manifest_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"execution_performed":false,"scope":"Shared native State path selection from supplied existence observations: requested V3 then V4, then tract V3/V4 for missing block groups. The native loader prefers the selected path's binary sibling; it does not switch to a different store merely because that store has a binary. Pickle-only selections require the external native shim and cannot be queued as browser binary jobs. Observations do not verify file contents, unreadable/empty directories or filesystem freshness. No source file, graph or pickle was read and no engine ran."}))
}
