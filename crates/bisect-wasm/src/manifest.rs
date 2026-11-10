use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeArtifact { pub sha256:String, pub byte_length:u64 }
pub(crate) fn binary_check(recorded:&str,runtime_artifact:Option<RuntimeArtifact>,skip:bool)->Result<Value,String>{
 if runtime_artifact.as_ref().is_some_and(|a|a.sha256.len()!=64||!a.sha256.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))||a.byte_length==0||a.byte_length>256*1024*1024){return Err("Invalid bounded loaded WASM artifact identity.".into());}
 Ok(json!({"status":bisect_report::binary_provenance::assess(recorded,skip,runtime_artifact.as_ref().map(|a|a.sha256.as_str())),"recorded_sha256":recorded,"loaded_wasm":runtime_artifact,"advisory":true,"scope":"Literal native Verify hash comparison against host-supplied loaded WASM bytes. The browser worker measures the instantiated module; direct ABI callers supply this identity as a claim. Native executable and WASM module are different artifact kinds. A mismatch is advisory; equality does not prove native executable identity, build equivalence, source authenticity or original execution."}))
}
pub(crate) fn inspect(source_name:&str,text:&str,runtime_artifact:Option<RuntimeArtifact>)->Result<Value,String>{
 if source_name.is_empty()||source_name.len()>200||text.len()>1024*1024{return Err("Supply a source name of 1–200 UTF-8 bytes and manifest JSON up to 1 MiB.".into());}
 let current=json!({"version":env!("CARGO_PKG_VERSION"),"build_commit":env!("BISECT_BUILD_COMMIT"),"build_unix":env!("BISECT_BUILD_UNIX"),"rustc_version":env!("BISECT_RUSTC_VERSION"),"target":"wasm32-unknown-unknown"});
 let parsed:Result<Value,_>=serde_json::from_str(text);
 let recorded=parsed.as_ref().ok().and_then(|v|v.get("binary_sha256")).and_then(Value::as_str).unwrap_or("");
 let binary_check=binary_check(recorded,runtime_artifact,false)?;
 let (assessment,error)=match parsed{Ok(v)=>(Some(bisect_analysis::manifest_provenance::assess(&v,env!("CARGO_PKG_VERSION"),env!("BISECT_BUILD_COMMIT"))),None),Err(e)=>(None,Some(e.to_string()))};
 let exit=assessment.as_ref().map_or(2,|v|v["exit_code"].as_u64().unwrap_or(2));
 Ok(json!({"schema_version":"bisect-manifest-compatibility-v1","binary_check":binary_check,"status":if exit==2{"invalid-json"}else if exit==1{"version-mismatch"}else if assessment.as_ref().is_some_and(|v|v["version_status"]=="missing"){"version-unavailable"}else{"version-compatible"},"native_exit_code":exit,"assessment":assessment,"parse_error":error,"current_build":current,"source_name":source_name,"source_sha256":format!("sha256:{:x}",Sha256::digest(text.as_bytes())),"scope":"Native Doctor manifest binary-version comparison against this WASM build. Missing or non-string versions are warnings and retain native exit code 0; malformed JSON has code 2. Recorded timestamps and filenames are informational. A supplied loaded WASM artifact identity adds an advisory literal native hash comparison; native executable and WASM are different artifact kinds. No referenced path is opened, fetched or rehashed. Equal versions do not prove equal binaries, source commits, authentic execution, trustworthy inputs or legal compliance. Build metadata describes this compilation; it is not an authenticated attestation. The browser cannot attest that a native CLI binary and this WASM module are identical."}))
}
