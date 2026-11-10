//! Native `bisect validate` result, without filesystem or console I/O.
use serde_json::{json,Value};
pub fn assess(text:&str,strict:bool)->Value{
 let validation=match crate::validate_rplan_str(text){Ok(v)=>v,Err(e)=>return json!({"valid":false,"strict":strict,"warnings":[],"native_exit_code":1,"error":format!("FAIL: {e}")})};
 let raw:Value=serde_json::from_str(text).expect("successful validator parsed JSON");
 let n=raw["assignments"].as_object().map(|m|m.len()).unwrap_or(0);
 let k=raw["metadata"]["num_districts"].as_u64().unwrap_or(0);
 let state=raw["metadata"]["state_code"].as_str().unwrap_or("?");
 let year=raw["metadata"]["year"].as_str().unwrap_or("?");
 let chamber=raw["metadata"]["chamber"].as_str().unwrap_or("?");
 let failure=strict&&!validation.warnings.is_empty();
 json!({"valid":true,"strict":strict,"warnings":validation.warnings,"native_exit_code":if failure{1}else{0},"error":if failure{Some(format!("FAIL (strict): {} warning(s) treated as errors",validation.warnings.len()))}else{None},"native_summary":if failure{None}else{Some(format!("PASS: valid RPLAN v0.1 ({n} tracts, {k} districts, {state} {year} {chamber})"))},"source_version":raw["rplan_version"],"legacy_metadata":{"tracts":n.to_string(),"districts":k.to_string(),"state":state,"year":year,"chamber":chamber}})
}
