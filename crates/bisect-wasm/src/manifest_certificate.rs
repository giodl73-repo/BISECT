//! Native manifest certificate bindings over explicitly selected bytes.
use crate::package_verification::Input;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
pub(crate) fn verify(input:Input)->Result<Value,String>{
    if input.manifest_json.is_empty()||input.manifest_json.len()>1024*1024{return Err("Select a nonempty manifest up to 1 MiB.".into());}
    let files=crate::package_verification::selected_bytes(input.files_base64)?;
    let result=(||{
        // Match native typed PlanManifest parsing before extracting shared expectations.
        let manifest:bisect_report::manifest_schema::ReadOnlyPlanManifest=serde_json::from_str(&input.manifest_json).map_err(|e|format!("invalid manifest JSON: {e}"))?;
        let bindings=serde_json::from_value(serde_json::to_value(manifest).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        rplan_audit::manifest_verification::verify_manifest_certificate(&bindings,
            |path|{if !crate::package_verification::safe(path){return Err(format!("Unsafe selected manifest reference: {path}"));}files.get(path).map(|b|format!("{:x}",Sha256::digest(b))).ok_or_else(||format!("Manifest file not selected: {path}"))},
            |path|{if !crate::package_verification::safe(path){return Err(format!("Unsafe selected manifest reference: {path}"));}files.get(path).cloned().ok_or_else(||format!("Manifest file not selected: {path}"))})
    })();
    let (status,receipt,error)=match result{Ok(Some(receipt))=>("verified",serde_json::to_value(receipt).map_err(|e|e.to_string())?,None),Ok(None)=>("not-recorded",Value::Null,None),Err(e)=>("failed",Value::Null,Some(e))};
    let hashes:std::collections::BTreeMap<_,_>=files.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect();
    Ok(json!({"schema_version":"bisect-manifest-certificate-verification-v1","status":status,"native_exit_code":if status=="failed"{1}else{0},"verification":receipt,"error":error,"manifest_sha256":format!("{:x}",Sha256::digest(input.manifest_json.as_bytes())),"selected_file_sha256":hashes,"scope":"Shared native PlanManifest certificate checks: certificate file digest, manifest content/result/profile/context expectations, ILP algorithm lineage, context adjacency/geometry source-hash claims, and native certificate content/plan/context bindings. No certificate path yields not-recorded, not a verified certificate. Selected byte map only; no referenced paths or URLs are opened. Stored audit results and lineage remain claims. This operation does not verify the separate ILP audit summary, compare assignments, replay the engine, authenticate the author or establish legal validity."}))
}
