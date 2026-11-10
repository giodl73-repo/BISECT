use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct Input{pub state:String,pub year:String,pub chamber:String,#[serde(default)]pub source_name:Option<String>,#[serde(default)]pub policy_json:Option<String>,#[serde(default)]pub manifest_json:Option<String>}
pub fn inspect(input:Input)->Result<Value,String>{
 if input.state.len()!=2||!input.state.bytes().all(|c|c.is_ascii_uppercase())||!["2000","2010","2020"].contains(&input.year.as_str())||!["congressional","house","senate"].contains(&input.chamber.as_str()){return Err("Select a two-letter state, Census year and congressional/house/senate chamber.".into());}
 let (policy,name,kind,policy_sha)=crate::location_policy::database(input.source_name.as_deref(),input.policy_json.as_deref())?;
 let manifest_text=input.manifest_json.as_deref().unwrap_or(include_str!("../../../data/manifest.json"));
 if manifest_text.len()>2*1024*1024{return Err("State manifest exceeds 2 MiB.".into());}
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(manifest_text).map_err(|e|format!("State manifest: {e}"))?;
 let fallback=manifest.states.get(&input.state).and_then(|state|state.districts.get(&input.year)).copied().unwrap_or(0);
 let (count,from_policy)=bisect_analysis::location_policy::state_chamber_count(&policy,&input.state,&input.chamber,fallback as u64)?;
 if !(1..=500).contains(&count){return Err("Resolved chamber count is unavailable or exceeds the browser engine's 1–500 district bound.".into());}
 Ok(json!({"schema_version":"bisect-native-chamber-count-v1","state":input.state,"year":input.year,"chamber":input.chamber,"districts":count,"allocation_source":if from_policy{"policy"}else{"congressional-fallback"},"congressional_fallback":fallback,"policy_source_name":name,"policy_source_kind":kind,"policy_sha256":policy_sha,"manifest_sha256":format!("{:x}",Sha256::digest(manifest_text.as_bytes())),"execution_performed":false,"scope":"Shared native single-State chamber count: exact policy-key lookup, positive House/Senate counts, absent-chamber error and congressional manifest fallback. Counts are policy/source claims, not verified current law. No template options, tolerance or graph were modified; no partition or certification occurred. Native bulk States remains congressional-only."}))
}
