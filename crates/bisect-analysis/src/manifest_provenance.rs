//! Native Doctor manifest compatibility rules. This is not authentication.
use serde_json::{json,Value};
pub fn version_matches(current:&str,recorded:&str)->Result<(),String>{
 if recorded==current{Ok(())}else{Err(format!("binary version mismatch: manifest records {recorded} but this binary is {current}"))}
}
pub fn assess(value:&Value,current_version:&str,current_commit:&str)->Value{
 let recorded=value.get("binary_version").and_then(Value::as_str).unwrap_or("");
 let mismatch=(!recorded.is_empty()).then(||version_matches(current_version,recorded).err()).flatten();
 let warning=if current_commit.starts_with("unknown"){Some("Running build commit is unknown; source provenance cannot be attested.")}else if current_commit.ends_with("-dirty"){Some("Running build came from a dirty working tree; source provenance is degraded.")}else{None};
 json!({"exit_code":if mismatch.is_some(){1}else{0},"hard_failures":if mismatch.is_some(){1}else{0},
  "version_status":if recorded.is_empty(){"missing"}else if mismatch.is_some(){"mismatch"}else{"matches"},"version_error":mismatch,
  "recorded":{"binary_version":recorded,"binary_sha256":value.get("binary_sha256").and_then(Value::as_str),"adjacency_sha256":value.get("adjacency_sha256").and_then(Value::as_str),"created_at":value.get("created_at").and_then(Value::as_str),"adjacency_file":value.get("adjacency_file").and_then(Value::as_str)},"build_warning":warning})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn native_missing_wrong_type_and_mismatch(){for v in [json!({}),json!({"binary_version":null}),json!([]),json!({"binary_version":123})]{assert_eq!(assess(&v,"1","unknown")["exit_code"],0);assert_eq!(assess(&v,"1","unknown")["version_status"],"missing");}assert_eq!(assess(&json!({"binary_version":"2"}),"1","abc")["exit_code"],1);}
 #[test]fn native_exact_version_and_commit_warnings(){assert_eq!(assess(&json!({"binary_version":"1"}),"1","abc-dirty")["version_status"],"matches");assert!(assess(&json!({}),"1","abc")["build_warning"].is_null());assert!(assess(&json!({}),"1","unknown")["build_warning"].is_string());}
}
