//! Bounded selected-byte COI input; shared native adjustment remains in bisect-runner.
use serde::{Deserialize,Serialize};use sha2::{Digest,Sha256};use std::collections::{BTreeMap,HashMap};use serde_json::{Value,json};
#[derive(Clone,Deserialize,Serialize)]#[serde(deny_unknown_fields)]
pub struct Input{pub source_name:String,pub weights_json:String}
pub(crate) fn validate(input:&Input)->Result<BTreeMap<String,f64>,String>{
 if input.source_name.is_empty()||input.source_name.len()>200||input.source_name.chars().any(char::is_control)||input.weights_json.len()>3*1024*1024{return Err("Select named UTF-8 COI JSON up to 3 MiB.".into());}
 let values=bisect_runner::coi_weights::parse(&input.weights_json)?;
 if values.len()>100_000||values.keys().any(|s|s.is_empty()||s.len()>200||s.chars().any(char::is_control)||["__proto__","constructor","prototype"].contains(&s.as_str())){return Err("COI source requires at most 100,000 bounded safe unit IDs.".into());}Ok(values)
}
pub(crate) fn hash(input:&Input)->String{format!("{:x}",Sha256::digest(input.weights_json.as_bytes()))}
pub(crate) fn apply(input:&Input,geoids:&[String],base:HashMap<(usize,usize),f64>,active:bool)->Result<(HashMap<(usize,usize),f64>,Value),String>{
 let values=validate(input)?;let ids:HashMap<_,_>=geoids.iter().cloned().enumerate().collect();let matched=geoids.iter().filter(|id|values.contains_key(*id)).count();let adjusted=if active{bisect_runner::coi_weights::apply(base.clone(),&values,&ids)?}else{base.clone()};
 let changed=base.iter().filter(|(key,value)|adjusted[key]!=**value).count();
 let mut total=0.0;for value in adjusted.values(){total+=value;if !total.is_finite(){return Err("COI adjusted weight total is nonfinite.".into());}}
 Ok((adjusted,json!({"schema_version":"bisect-coi-weighting-v1","source_name":input.source_name,"source_sha256":hash(input),"supplied_units":values.len(),"matched_units":matched,"defaulted_units":geoids.len()-matched,"ignored_units":values.len()-matched,"applied_edges":if active{base.len()}else{0},"changed_edges":changed,"status":if !active{"no-effect-unweighted-native-empty-map"}else if changed==0{"applied-no-edge-change"}else{"applied"},"formula":"resolved_edge_weight * sqrt(unit_weight_u * unit_weight_v)","missing_unit_weight":1,"scope":"Shared native COI post-weighter on resolved algorithm edge weights. Missing units default to one; unknown IDs are counted and ignored. Physical boundary lengths and census populations are retained. Native unweighted mode has an empty computed weight map and COI has no effect. Invalid, duplicate or overflowing selected source weights reject the browser Run; the native filesystem State wrapper retains its warning/fallback policy. Source hashes establish selected-byte integrity, not community provenance or legal status."})))
}
