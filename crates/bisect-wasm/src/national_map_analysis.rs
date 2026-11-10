//! Explicit native national-map analysis semantics. Raw source bytes are retained
//! and hashed; normalization affects renderer input only, never source authority.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use base64::{engine::general_purpose::STANDARD,Engine};
use std::collections::BTreeMap;
const CAP:usize=3*1024*1024;
fn statistics(bytes:Option<&[u8]>,map_type:&str)->(BTreeMap<u64,f64>,Value){
 let field=match map_type{"political"=>"dem_pct","demographic"=>"pct_minority",_=>"polsby_popper"};
 let (value,status)=match bytes {
  None=>(Value::Null,"missing"),
  Some(bytes)=>match std::str::from_utf8(bytes){
   Err(_)=>(Value::Null,"unreadable-utf8"),
   Ok(text)=>match serde_json::from_str::<Value>(text){Ok(value)=>(value,"parsed"),Err(_)=>(Value::Null,"invalid-json")},
  },
 };
 let mut stats=BTreeMap::new();let mut usable=0usize;
 let rows=value.get("districts").and_then(Value::as_array);
 if let Some(rows)=rows {for row in rows{
  // Native filter_map ignores invalid rows before collecting a last-valid-wins map.
  if let (Some(id),Some(stat))=(row.get("district").and_then(Value::as_u64),row.get(field).and_then(Value::as_f64)){stats.insert(id,stat);usable+=1;}
 }}
 let ignored_metadata=["analyzer","state","year","plan_hash"].into_iter().filter(|key|value.get(*key).is_some()).collect::<Vec<_>>();
 let report=json!({"status":if map_type=="districts"{"not-used"}else{status},"statistic_field":if map_type=="districts"{None}else{Some(field)},"rows_seen":rows.map_or(0,Vec::len),"usable_rows":usable,"metadata_validation":"native-cli-unchecked","ignored_metadata_fields":ignored_metadata,"duplicate_district_policy":"last valid numeric row wins; unusable rows do not erase prior statistics"});
 (stats,report)
}
pub(crate) fn statistics_for_labels(bytes:Option<&[u8]>,map_type:&str,labels:&std::collections::BTreeSet<u64>)->(Value,Vec<u64>,Value){
 let (stats,report)=statistics(bytes,map_type);let field=match map_type{"political"=>"dem_pct","demographic"=>"pct_minority",_=>"polsby_popper"};
 let rows=stats.iter().filter(|(id,_)|labels.contains(id)).map(|(id,stat)|{let mut row=json!({"district":id});row[field]=json!(stat);row}).collect::<Vec<_>>();
 let fallback=labels.iter().copied().filter(|id|!stats.contains_key(id)).collect();(json!({"districts":rows}),fallback,report)
}
pub(crate) fn prepare(input:&crate::map_export::Input)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{
 if input.analysis_json.is_some()&&input.analysis_base64.is_some(){return Err("Select one national analysis text or byte source".into());}
 let raw=if let Some(encoded)=&input.analysis_base64{
  if encoded.len()>4*1024*1024{return Err("Native national analysis exceeds 3 MiB".into());}
  Some(STANDARD.decode(encoded).map_err(|_|"Invalid native national analysis Base64")?)
 }else{input.analysis_json.as_ref().map(|text|text.as_bytes().to_vec())};
 if raw.as_ref().is_some_and(|bytes|bytes.len()>CAP){return Err("Native national analysis exceeds 3 MiB".into());}
 let mut geometry_input=input.clone();geometry_input.map_type="districts".into();geometry_input.analysis_json=None;geometry_input.analysis_base64=None;
 let (districts,k,_,mut metadata)=crate::map_export::prepare(&geometry_input)?;
 let (stats,report)=statistics(raw.as_deref(),&input.map_type);
 let field=match input.map_type.as_str(){"political"=>"dem_pct","demographic"=>"pct_minority",_=>"polsby_popper"};
 // Only actual one-based final-plan district IDs can be rendered. Keeping u64
 // keys until this filter avoids wasm32 wraparound on unrelated huge IDs.
 let rows=stats.iter().filter(|(id,_)|**id>=1&&**id<=k as u64).map(|(id,value)|{let mut row=json!({"district":id});row[field]=json!(value);row}).collect::<Vec<_>>();
 let fallback=if input.map_type=="districts"{vec![]}else{(1..=k as u64).filter(|id|!stats.contains_key(id)).collect::<Vec<_>>()};
 metadata["map_type"]=json!(input.map_type);metadata["fallback_districts"]=json!(fallback);metadata["analysis_policy"]=json!("native-defaults");metadata["analysis_observation"]=report;
 metadata["analysis_source_sha256"]=json!(raw.as_ref().map(|bytes|format!("{:x}",Sha256::digest(bytes))));
 metadata["analysis_scope"]=json!("Explicit native national analysis policy: missing, unreadable UTF-8 or invalid JSON analysis uses native defaults. Any JSON shape is accepted; only rows with unsigned district IDs and numeric statistics contribute, with last-valid-row wins. Declared analyzer/state/year/plan_hash metadata is ignored, not authenticated. Original raw source bytes are hashed; normalized renderer rows are not replacement source data. Defaults are political 0.5, demographic 0.0 and compactness 0.3.");
 Ok((districts,k,if input.map_type=="districts"{Value::Null}else{json!({"districts":rows})},metadata))
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn last_valid_numeric_row_matches_native_filter_map(){
  let (stats,report)=statistics(Some(br#"{"state":"wrong","districts":[{"district":1,"dem_pct":0.7},{"district":1,"dem_pct":null},{"district":2,"dem_pct":0.4},{"district":2,"dem_pct":0.8},{"district":4294967297,"dem_pct":0.9},{"district":1.0,"dem_pct":0.1},null]}"#),"political");
  assert_eq!(stats[&1],0.7);assert_eq!(stats[&2],0.8);assert_eq!(stats[&4294967297],0.9);assert_eq!(report["rows_seen"],7);assert_eq!(report["usable_rows"],4);
 }
 #[test]fn source_failures_and_arbitrary_shapes_use_empty_statistics(){
  for (bytes,status)in [(None,"missing"),(Some(&[255u8][..]),"unreadable-utf8"),(Some(&b"{"[..]),"invalid-json"),(Some(&b"null"[..]),"parsed"),(Some(&b"[]"[..]),"parsed")]{let (stats,report)=statistics(bytes,"demographic");assert!(stats.is_empty());assert_eq!(report["status"],status);}
 }
 #[test]fn duplicate_json_keys_follow_native_value_parser(){
  let (stats,_)=statistics(Some(br#"{"districts":[],"districts":[{"district":1,"polsby_popper":0.6}]}"#),"compactness");assert_eq!(stats[&1],0.6);
 }
}
