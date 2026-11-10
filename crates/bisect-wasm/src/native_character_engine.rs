//! Explicit selected-file run envelope. Ordinary/build/multiscale requests keep
//! their native option semantics; the selected CSV is the only attached source.
use serde::Deserialize;
use serde_json::Value;
use crate::native_character_source;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input{#[serde(default)]pub inspect_only:bool,pub source:native_character_source::Input,pub request_json:String,pub label:String,pub chamber:String,pub created_at:String}
pub fn run(input:Input)->Result<Value,String>{
 crate::prepared_engine::run_with_source(crate::prepared_engine::Input{context_source:None,inspect_only:input.inspect_only,request_json:input.request_json,label:input.label,chamber:input.chamber,created_at:input.created_at},Some(input.source))
}
