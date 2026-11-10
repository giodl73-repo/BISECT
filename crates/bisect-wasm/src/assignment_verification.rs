//! Selected assignment comparison using native Verify semantics; no engine replay.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::HashMap;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub original_json:String,pub reference_json:String,pub min_similarity:f64}
pub(crate) fn assignments(text:&str)->Result<HashMap<String,usize>,String>{
    let values:HashMap<String,u64>=serde_json::from_str(text).map_err(|e|format!("Invalid assignment JSON: {e}"))?;
    if values.len()>1_000_000||values.iter().any(|(u,d)|u.len()>240||*d>u32::MAX as u64){return Err("Assignment sources exceed one million units, 240-byte identifiers or wasm32 district range.".into());}
    Ok(values.into_iter().map(|(u,d)|(u,d as usize)).collect())
}
pub(crate) fn compare(input:Input)->Result<Value,String>{
    if input.original_json.is_empty()||input.reference_json.is_empty()||input.original_json.len()>8*1024*1024||input.reference_json.len()>8*1024*1024||!input.min_similarity.is_finite(){return Err("Select two nonempty assignment JSON sources up to 8 MiB each and a finite threshold.".into());}
    let original=assignments(&input.original_json)?;let reference=assignments(&input.reference_json)?;
    let agreement=bisect_report::assignment_verification::assignment_agreement(&original,&reference);
    let status=if original.is_empty(){"unavailable"}else if agreement.similarity>=input.min_similarity{"match"}else{"different"};
    Ok(json!({"schema_version":"bisect-assignment-verification-v1","status":status,"native_exit_code":if status=="match"{0}else{1},"agreement":agreement,"min_similarity":input.min_similarity,"original_sha256":format!("{:x}",Sha256::digest(input.original_json.as_bytes())),"reference_sha256":format!("{:x}",Sha256::digest(input.reference_json.as_bytes())),"scope":"Shared native Verify assignment agreement: exact unit identifier and district-number equality, divided by the larger map size. Native CLI labels this Jaccard similarity, but the denominator is not a set union. District numbers are compared literally without relabeling. An empty original is unavailable as in assignments-only Verify; an empty reference has zero similarity. Finite thresholds retain native comparison behavior, including values outside 0–1. Only selected JSON bytes are compared; no plan discovery, manifest/ILP/certificate verification, population/connectivity validation or engine replay is performed."}))
}
