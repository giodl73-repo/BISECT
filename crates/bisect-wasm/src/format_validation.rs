use serde_json::{json,Value};
use sha2::{Digest,Sha256};
pub(crate) fn inspect(source:&str,text:&str,strict:bool)->Result<Value,String>{
 if source.is_empty()||source.len()>200||text.is_empty()||text.len()>3*1024*1024{return Err("Select nonempty RPLAN text up to 3 MiB with a source name up to 200 UTF-8 bytes.".into());}
 // Bound native legacy warning enumeration and modern district allocation before validation.
 if let Ok(raw)=serde_json::from_str::<Value>(text){
  let modern=raw["rplan_version"]=="0.2";
  let k=if modern{raw["plan"]["k"].as_u64()}else{raw["metadata"]["num_districts"].as_u64()};
  let n=if modern{raw["plan"]["units"]["unit_ids"].as_array().map(Vec::len)}else{raw["assignments"].as_object().map(|a|a.len())};
  if k.is_some_and(|k|k>100_000)||n.is_some_and(|n|n>100_000){return Err("Native format validation supports up to 100,000 units and 100,000 declared districts in the browser.".into());}
 }
 let mut r=bisect_report::format_validation::assess(text,strict);
 r["schema_version"]=json!("bisect-native-format-validation-v1");r["status"]=json!(if r["native_exit_code"]==0{if r["warnings"].as_array().is_some_and(|a|!a.is_empty()){"pass-with-warnings"}else{"pass"}}else if r["valid"]==true{"strict-warning-failure"}else{"invalid-format"});
 r["source_name"]=json!(source);r["source_sha256"]=json!(format!("sha256:{:x}",Sha256::digest(text.as_bytes())));
 r["scope"]=json!("Shared native bisect validate parser, legacy GEOID/metadata/district-range/geometry-type checks, modern v0.2 reader and strict warning policy. A format pass does not verify populations, contiguity, complete state coverage, topology, provenance or legal compliance. Empty legacy assignments are accepted by the native validator. Native v0.2 validation has no legacy warnings; the historical native console summary still says v0.1 and reads legacy metadata, so its version/count text is not a modern plan summary. Source version is shown separately; use Inspect exact RPLAN for modern plan measurements. No conversion or plan execution occurs. Imported results remain unverified until an explicit Run.");Ok(r)
}
